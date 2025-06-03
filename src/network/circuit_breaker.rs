use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use tracing::{info, warn};

#[derive(Debug)]
pub enum CircuitState {
    Closed,    // Normal operation, requests pass through
    Open,      // Circuit is open, requests are blocked
    HalfOpen,  // Testing if the service is healthy again
}

#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    state: AtomicU64,  // 0 = Closed, 1 = Open, 2 = HalfOpen
    failure_threshold: u64,
    reset_timeout: Duration,
    request_timeout: Duration,
    
    failure_count: AtomicU64,
    last_failure: Mutex<Option<Instant>>,
    last_success: Mutex<Option<Instant>>,
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
            state: AtomicU64::new(0),  // Start in Closed state
            failure_threshold,
            reset_timeout,
            request_timeout,
            failure_count: AtomicU64::new(0),
            last_failure: Mutex::new(None),
            last_success: Mutex::new(None),
        }
    }
    
    pub fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,  // Default to closed for unknown states
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
                        self.state.store(2, Ordering::Relaxed);
                        info!("Circuit '{}' transitioning from Open to Half-Open state", self.name);
                        true
                    } else {
                        false
                    }
                } else {
                    // No recorded failure, default to allowing execution
                    true
                }
            },
            CircuitState::HalfOpen => {
                // In half-open state, only allow one request to test the service
                true
            },
        }
    }
    
    pub async fn on_success(&self) {
        let mut last_success = self.last_success.lock().await;
        *last_success = Some(Instant::now());
        
        match self.get_state() {
            CircuitState::HalfOpen => {
                // On success in half-open state, transition to closed
                self.state.store(0, Ordering::Relaxed);
                self.failure_count.store(0, Ordering::Relaxed);
                info!("Circuit '{}' transitioning from Half-Open to Closed state", self.name);
            },
            CircuitState::Closed => {
                // In closed state, reset failure count after success
                self.failure_count.store(0, Ordering::Relaxed);
            },
            _ => {},
        }
    }
    
    pub async fn on_failure(&self) {
        let mut last_failure = self.last_failure.lock().await;
        *last_failure = Some(Instant::now());
        
        match self.get_state() {
            CircuitState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failures >= self.failure_threshold {
                    // Transition to open
                    self.state.store(1, Ordering::Relaxed);
                    warn!("Circuit '{}' transitioning from Closed to Open state after {} failures", self.name, failures);
                }
            },
            CircuitState::HalfOpen => {
                // On failure in half-open state, transition back to open
                self.state.store(1, Ordering::Relaxed);
                warn!("Circuit '{}' transitioning from Half-Open back to Open state after test failure", self.name);
            },
            _ => {},
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
        let operation_future = operation();
        let timeout_future = tokio::time::sleep(self.request_timeout);
        
        let result = tokio::select! {
            result = operation_future => result,
            _ = timeout_future => Err(format!("Operation timed out after {:?}", self.request_timeout)),
        };
        
        match result {
            Ok(value) => {
                self.on_success().await;
                Ok(value)
            },
            Err(error) => {
                self.on_failure().await;
                Err(error)
            },
        }
    }
}