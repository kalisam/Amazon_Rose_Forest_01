use amazon_rose_forest::{core::metrics::MetricsCollector, nerv::runtime::Runtime};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn runtime_start_initializes_shard_manager() {
    let metrics = Arc::new(MetricsCollector::new());
    let mut runtime = Runtime::new(metrics.clone());
    runtime.start().await.unwrap();
    assert!(runtime.shard_manager().is_some());
    runtime.stop().await.unwrap();
}

#[tokio::test]
async fn runtime_stop_sends_shutdown_signal() {
    let metrics = Arc::new(MetricsCollector::new());
    let mut runtime = Runtime::new(metrics);
    runtime.start().await.unwrap();

    runtime.stop().await.unwrap();

    if let Some(tx) = runtime.shutdown_sender() {
        // The channel should close once the background task handles the shutdown message
        let closed = timeout(Duration::from_secs(1), tx.closed()).await;
        assert!(closed.is_ok(), "shutdown signal not processed in time");
    } else {
        panic!("shutdown channel missing");
    }
}
