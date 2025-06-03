use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::nerv::runtime::Runtime;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Amazon Rose Forest...");
    
    // Initialize metrics
    let metrics = Arc::new(MetricsCollector::new());
    
    // Start the runtime
    let runtime = Runtime::new(metrics.clone());
    runtime.start().await?;
    
    info!("Amazon Rose Forest started successfully");
    
    // Wait for ctrl+c signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    
    Ok(())
}