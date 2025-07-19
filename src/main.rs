use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::core::vector::Vector;
use amazon_rose_forest::darwin::agent::CodingAgent;
use amazon_rose_forest::darwin::exploration::ExplorationStrategy;
use amazon_rose_forest::darwin::ritual::RitualManager;
use amazon_rose_forest::darwin::self_improvement::SelfImprovementEngine;
use amazon_rose_forest::darwin::validation::{
    PerformanceBenchmarkStage, SecurityValidationStage, UnitTestStage, ValidationPipeline,
};
use amazon_rose_forest::nerv::runtime::Runtime;
use amazon_rose_forest::sharding::manager::ShardManager;
use amazon_rose_forest::sharding::vector_index::DistanceMetric;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!(
        "Starting Amazon Rose Forest v{}",
        amazon_rose_forest::VERSION
    );

    // Initialize metrics
    let metrics =
        Arc::new(MetricsCollector::new().with_report_interval(std::time::Duration::from_secs(30)));

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

    // Initialize Darwin Gödel Machine components
    info!("Initializing Darwin Gödel Machine components");

    // Create validation pipeline
    let mut validation_pipeline = ValidationPipeline::new(metrics.clone());
    validation_pipeline.add_stage(UnitTestStage);
    validation_pipeline.add_stage(PerformanceBenchmarkStage);
    validation_pipeline.add_stage(SecurityValidationStage);

    // Set validation thresholds
    validation_pipeline.set_threshold("unit_tests.pass_rate", 0.9);
    validation_pipeline.set_threshold("performance.vector_search_latency_ms", 10.0);
    validation_pipeline.set_threshold("security.vulnerability_score", 0.2);

    let validation_pipeline = Arc::new(validation_pipeline);

    // Create exploration strategy
    let exploration_strategy = Arc::new(ExplorationStrategy::new(metrics.clone()));

    // Create self-improvement engine
    let self_improvement_engine = Arc::new(SelfImprovementEngine::new(
        metrics.clone(),
        validation_pipeline.clone(),
        exploration_strategy.clone(),
    ));

    // Create coding agent
    let coding_agent = Arc::new(CodingAgent::new(metrics.clone()));

    // Create ritual manager
    let ritual_manager = Arc::new(RitualManager::new(metrics.clone()));

    info!("Darwin Gödel Machine components initialized");

    // Create a demo shard
    let shard_id = shard_manager.create_shard("demo_shard").await?;

    // Create a vector index
    let dimensions = 60;
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
            debug!("Added vector {}/{}: {}", i + 1, 100, vector_id);
        }
    }

    // Search for similar vectors
    let query = Vector::random(dimensions);
    let results = shard_manager.search_vectors(shard_id, &query, 5).await?;

    info!("Search results:");
    for (i, result) in results.iter().enumerate() {
        info!("  {}: ID={}, score={:.4}", i + 1, result.id, result.score);
        if let Some(metadata) = &result.metadata {
            if let Some(idx) = metadata.get("index") {
                debug!("    index={}", idx);
            }
        }
    }

    // Get index statistics
    let stats = index.stats().await;
    info!(
        "Index statistics: {} vectors, {} buckets, avg bucket size: {:.2}",
        stats.vector_count, stats.bucket_count, stats.avg_bucket_size
    );

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

    // Start self-improvement loop
    let self_improvement_clone = self_improvement_engine.clone();
    tokio::spawn(async move {
        loop {
            // Generate new improvement proposals
            match self_improvement_clone.generate_modifications().await {
                Ok(ids) => {
                    if !ids.is_empty() {
                        info!("Generated {} new improvement proposals", ids.len());
                    }
                }
                Err(e) => {
                    error!("Failed to generate improvements: {}", e);
                }
            }

            // Wait before next iteration
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        }
    });

    // Create an initial learning ritual
    let ritual_manager_clone = ritual_manager.clone();
    tokio::spawn(async move {
        use amazon_rose_forest::darwin::ritual::RitualStage;
        use amazon_rose_forest::darwin::ritual::RitualStageStatus;

        // Define ritual stages
        let stages = vec![
            RitualStage {
                name: "exploration".to_string(),
                description: "Explore potential improvements".to_string(),
                status: RitualStageStatus::Pending,
                depends_on: Vec::new(),
                artifacts: Vec::new(),
                started_at: None,
                completed_at: None,
            },
            RitualStage {
                name: "validation".to_string(),
                description: "Validate proposed improvements".to_string(),
                status: RitualStageStatus::Pending,
                depends_on: vec!["exploration".to_string()],
                artifacts: Vec::new(),
                started_at: None,
                completed_at: None,
            },
            RitualStage {
                name: "deployment".to_string(),
                description: "Deploy approved improvements".to_string(),
                status: RitualStageStatus::Pending,
                depends_on: vec!["validation".to_string()],
                artifacts: Vec::new(),
                started_at: None,
                completed_at: None,
            },
            RitualStage {
                name: "reflection".to_string(),
                description: "Learn from the process".to_string(),
                status: RitualStageStatus::Pending,
                depends_on: vec!["deployment".to_string()],
                artifacts: Vec::new(),
                started_at: None,
                completed_at: None,
            },
        ];

        // Create ritual
        match ritual_manager_clone
            .create_ritual(
                "Initial Self-Improvement Cycle",
                "First cycle of self-improvement for the system",
                stages,
            )
            .await
        {
            Ok(ritual_id) => {
                info!("Created initial learning ritual with ID: {}", ritual_id);

                // Start the first stage
                if let Err(e) = ritual_manager_clone
                    .start_stage(ritual_id, "exploration")
                    .await
                {
                    error!("Failed to start exploration stage: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to create learning ritual: {}", e);
            }
        }
    });

    info!("Amazon Rose Forest started successfully with Darwin Gödel Machine integration");

    // Wait for ctrl+c signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
