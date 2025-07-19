use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation, requests pass through
    Open,     // Circuit is open, requests are blocked
    HalfOpen, // Testing if the service is healthy again
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub rejected_calls: u64,
    pub state_transitions: Vec<(CircuitState, CircuitState, chrono::DateTime<chrono::Utc>)>,
    pub current_state: CircuitState,
    pub current_failure_count: u64,
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    pub last_state_change: Option<chrono::DateTime<chrono::Utc>>,
    pub avg_response_time_ms: f64,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    state: AtomicU64, // 0 = Closed, 1 = Open, 2 = HalfOpen
    failure_threshold: u64,
    reset_timeout: Duration,
    request_timeout: Duration,

    failure_count: AtomicU64,
    successful_calls: AtomicU64,
    failed_calls: AtomicU64,
    rejected_calls: AtomicU64,
    state_transitions: Mutex<Vec<(CircuitState, CircuitState, chrono::DateTime<chrono::Utc>)>>,
    last_failure: Mutex<Option<Instant>>,
    last_success: Mutex<Option<Instant>>,
    last_state_change: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
    response_times: Mutex<Vec<Duration>>,
}

impl CircuitBreaker {
    pub fn new(
        name: &str,
        failure_threshold: u64,
        reset_timeout: Duration,
        request_timeout: Duration,
    ) -> Self {
        Self {
            name: name.to_string(),
            state: AtomicU64::new(0), // Start in Closed state
            failure_threshold,
            reset_timeout,
            request_timeout,
            failure_count: AtomicU64::new(0),
            successful_calls: AtomicU64::new(0),
            failed_calls: AtomicU64::new(0),
            rejected_calls: AtomicU64::new(0),
            state_transitions: Mutex::new(Vec::new()),
            last_failure: Mutex::new(None),
            last_success: Mutex::new(None),
            last_state_change: Mutex::new(None),
            response_times: Mutex::new(Vec::new()),
        }
    }

    pub fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed, // Default to closed for unknown states
        }
    }

    async fn transition_state(&self, new_state: CircuitState) {
        let current_state = self.get_state();

        if current_state != new_state {
            self.state.store(
                match new_state {
                    CircuitState::Closed => 0,
                    CircuitState::Open => 1,
                    CircuitState::HalfOpen => 2,
                },
                Ordering::Relaxed,
            );

            let now = chrono::Utc::now();

            // Record the state transition
            {
                let mut transitions = self.state_transitions.lock().await;
                transitions.push((current_state, new_state, now));
            }

            // Update last state change
            {
                let mut last_change = self.last_state_change.lock().await;
                *last_change = Some(now);
            }

            info!(
                "Circuit '{}' transitioning from {} to {} state",
                self.name, current_state, new_state
            );
        }
    }

    pub async fn can_execute(&self) -> bool {
        match self.get_state() {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if it's time to transition to half-open
                let last_failure = self.last_failure.lock().await;
                if let Some(time) = *last_failure {
                    if time.elapsed() >= self.reset_timeout {
                        // Transition to half-open
                        drop(last_failure); // Release the mutex before the state transition
                        self.transition_state(CircuitState::HalfOpen).await;
                        true
                    } else {
                        self.rejected_calls.fetch_add(1, Ordering::Relaxed);
                        false
                    }
                } else {
                    // No recorded failure, default to allowing execution
                    true
                }
            }
            CircuitState::HalfOpen => {
                // In half-open state, only allow one request to test the service
                true
            }
        }
    }

    pub async fn on_success(&self) {
        let now = Instant::now();
        {
            let mut last_success = self.last_success.lock().await;
            *last_success = Some(now);
        }

        self.successful_calls.fetch_add(1, Ordering::Relaxed);

        match self.get_state() {
            CircuitState::HalfOpen => {
                // On success in half-open state, transition to closed
                self.transition_state(CircuitState::Closed).await;
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::Closed => {
                // In closed state, reset failure count after success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub async fn on_failure(&self) {
        let now = Instant::now();
        {
            let mut last_failure = self.last_failure.lock().await;
            *last_failure = Some(now);
        }

        self.failed_calls.fetch_add(1, Ordering::Relaxed);

        match self.get_state() {
            CircuitState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failures >= self.failure_threshold {
                    // Transition to open
                    self.transition_state(CircuitState::Open).await;
                }
            }
            CircuitState::HalfOpen => {
                // On failure in half-open state, transition back to open
                self.transition_state(CircuitState::Open).await;
            }
            _ => {}
        }
    }

    pub async fn record_response_time(&self, duration: Duration) {
        let mut times = self.response_times.lock().await;
        times.push(duration);

        // Keep only the last 100 response times to avoid unbounded growth
        if times.len() > 100 {
            times.remove(0);
        }
    }

    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        let transitions = self.state_transitions.lock().await.clone();
        let last_failure = self.last_failure.lock().await;
        let last_success = self.last_success.lock().await;
        let last_state_change = self.last_state_change.lock().await;
        let response_times = self.response_times.lock().await;

        let avg_response_time = if !response_times.is_empty() {
            let sum: u128 = response_times.iter().map(|d| d.as_millis()).sum();
            sum as f64 / response_times.len() as f64
        } else {
            0.0
        };

        CircuitBreakerMetrics {
            successful_calls: self.successful_calls.load(Ordering::Relaxed),
            failed_calls: self.failed_calls.load(Ordering::Relaxed),
            rejected_calls: self.rejected_calls.load(Ordering::Relaxed),
            state_transitions: transitions,
            current_state: self.get_state(),
            current_failure_count: self.failure_count.load(Ordering::Relaxed),
            last_failure: last_failure.map(|t| {
                let elapsed = t.elapsed();
                chrono::Utc::now() - chrono::Duration::from_std(elapsed).unwrap()
            }),
            last_success: last_success.map(|t| {
                let elapsed = t.elapsed();
                chrono::Utc::now() - chrono::Duration::from_std(elapsed).unwrap()
            }),
            last_state_change: *last_state_change,
            avg_response_time_ms: avg_response_time,
        }
    }

    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        if !self.can_execute().await {
            return Err(format!("Circuit '{}' is open", self.name));
        }

        // Setup timeout for the operation
        let start = Instant::now();
        let operation_future = operation();
        let timeout_future = tokio::time::sleep(self.request_timeout);

        let result = tokio::select! {
            result = operation_future => result,
            _ = timeout_future => Err(format!("Operation timed out after {:?}", self.request_timeout)),
        };

        let duration = start.elapsed();
        self.record_response_time(duration).await;

        match &result {
            Ok(_) => {
                debug!(
                    "Circuit '{}' operation succeeded in {:?}",
                    self.name, duration
                );
                self.on_success().await;
            }
            Err(error) => {
                warn!(
                    "Circuit '{}' operation failed with error: {}",
                    self.name, error
                );
                self.on_failure().await;
            }
        }

        result
    }
}
