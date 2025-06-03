use std::sync::LazyLock;
use once_cell::sync::Lazy;
use prometheus::{
    Registry, Counter, Gauge, Histogram, HistogramOpts, 
    CounterVec, GaugeVec, HistogramVec, Opts, register_counter_vec, 
    register_gauge_vec, register_histogram_vec,
};

// Global Prometheus registry
pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

// Define metrics

// Vector operations
pub static VECTOR_OPS_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "vector_operations_total",
        "Number of vector operations performed",
        &["operation"],
        REGISTRY.clone(),
    )
    .expect("Failed to create vector operations counter")
});

// Vector search metrics
pub static VECTOR_SEARCH_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        HistogramOpts::new(
            "vector_search_duration_seconds",
            "Duration of vector search operations"
        ).buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        &["index", "dimensions"],
        REGISTRY.clone(),
    )
    .expect("Failed to create vector search duration histogram")
});

// Shard metrics
pub static SHARD_COUNT: Lazy<Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "shard_count",
        "Number of active shards",
        REGISTRY.clone(),
    )
    .expect("Failed to create shard count gauge")
});

pub static SHARD_VECTORS: Lazy<GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "shard_vectors",
        "Number of vectors in each shard",
        &["shard_id"],
        REGISTRY.clone(),
    )
    .expect("Failed to create shard vectors gauge")
});

// Circuit breaker metrics
pub static CIRCUIT_BREAKER_STATE: Lazy<GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "circuit_breaker_state",
        "Circuit breaker state (0=closed, 1=open, 2=half-open)",
        &["name"],
        REGISTRY.clone(),
    )
    .expect("Failed to create circuit breaker state gauge")
});

pub static CIRCUIT_BREAKER_FAILURES: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "circuit_breaker_failures_total",
        "Number of circuit breaker failures",
        &["name"],
        REGISTRY.clone(),
    )
    .expect("Failed to create circuit breaker failures counter")
});

// System metrics
pub static SYSTEM_MEMORY_BYTES: Lazy<Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "system_memory_bytes",
        "Current memory usage in bytes",
        REGISTRY.clone(),
    )
    .expect("Failed to create memory usage gauge")
});

pub static SYSTEM_CPU_USAGE: Lazy<Gauge> = Lazy::new(|| {
    prometheus::register_gauge!(
        "system_cpu_usage",
        "Current CPU usage percentage",
        REGISTRY.clone(),
    )
    .expect("Failed to create CPU usage gauge")
});

// Helper functions

/// Record a vector operation
pub fn record_vector_operation(operation: &str) {
    VECTOR_OPS_COUNTER.with_label_values(&[operation]).inc();
}

/// Record a vector search duration
pub fn record_search_duration(index: &str, dimensions: usize, duration_secs: f64) {
    VECTOR_SEARCH_DURATION
        .with_label_values(&[index, &dimensions.to_string()])
        .observe(duration_secs);
}

/// Update shard count
pub fn update_shard_count(count: usize) {
    SHARD_COUNT.set(count as f64);
}

/// Update vectors in a shard
pub fn update_shard_vectors(shard_id: &str, count: usize) {
    SHARD_VECTORS.with_label_values(&[shard_id]).set(count as f64);
}

/// Update circuit breaker state
pub fn update_circuit_breaker_state(name: &str, state: crate::network::circuit_breaker::CircuitState) {
    let state_value = match state {
        crate::network::circuit_breaker::CircuitState::Closed => 0.0,
        crate::network::circuit_breaker::CircuitState::Open => 1.0,
        crate::network::circuit_breaker::CircuitState::HalfOpen => 2.0,
    };
    CIRCUIT_BREAKER_STATE.with_label_values(&[name]).set(state_value);
}

/// Record a circuit breaker failure
pub fn record_circuit_breaker_failure(name: &str) {
    CIRCUIT_BREAKER_FAILURES.with_label_values(&[name]).inc();
}