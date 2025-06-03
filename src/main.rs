use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::nerv::runtime::Runtime;
use amazon_rose_forest::sharding::manager::ShardManager;
use amazon_rose_forest::sharding::vector_index::DistanceMetric;
use amazon_rose_forest::core::vector::Vector;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, error, debug, warn};
use anyhow::Result;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Amazon Rose Forest v{}", amazon_rose_forest::VERSION);
    
    // Initialize metrics
    let metrics = Arc::new(MetricsCollector::new().with_report_interval(std::time::Duration::from_secs(30)));
    
    // Start the runtime
    let mut runtime = Runtime::new(metrics.clone());
    runtime.start().await?;
    
    // Initialize shard manager
    let shard_manager = match runtime.shard_manager() {
        Some(manager) => manager,
        None => {
            error!("Failed to get shard manager from runtime");
            return Err(anyhow::anyhow!("Shard manager not initialized"));
        }
    };
    
    // Create a demo shard
    let shard_id = shard_manager.create_shard("demo_shard").await?;
    
    // Create a vector index
    let dimensions = 128;
    let index = shard_manager
        .create_vector_index(shard_id, "demo_index", dimensions, DistanceMetric::Cosine)
        .await?;
        
    info!("Created vector index with {} dimensions", dimensions);
    
    // Add some test vectors
    for i in 0..100 {
        let vector = Vector::random(dimensions);
        
        let mut metadata = HashMap::new();
        metadata.insert("index".to_string(), i.to_string());
        metadata.insert("created".to_string(), chrono::Utc::now().to_rfc3339());
        
        let vector_id = shard_manager
            .add_vector(shard_id, vector, Some(metadata))
            .await?;
            
        if i % 10 == 0 {
            debug!("Added vector {}/{}: {}", i+1, 100, vector_id);
        }
    }
    
    // Search for similar vectors
    let query = Vector::random(dimensions);
    let results = shard_manager
        .search_vectors(shard_id, &query, 5)
        .await?;
        
    info!("Search results:");
    for (i, result) in results.iter().enumerate() {
        info!("  {}: ID={}, score={:.4}", i+1, result.id, result.score);
        if let Some(metadata) = &result.metadata {
            if let Some(idx) = metadata.get("index") {
                debug!("    index={}", idx);
            }
        }
    }
    
    // Get index statistics
    let stats = index.stats().await;
    info!("Index statistics: {} vectors, {} buckets, avg bucket size: {:.2}", 
          stats.vector_count, stats.bucket_count, stats.avg_bucket_size);
    
    // Start metrics reporting
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            if metrics_clone.report().await {
                debug!("Metrics reported");
            }
        }
    });
    
    info!("Amazon Rose Forest started successfully");
    
    // Wait for ctrl+c signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    
    Ok(())
}