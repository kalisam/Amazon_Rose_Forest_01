use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::darwin::exploration::ExplorationStrategy;
use amazon_rose_forest::darwin::ritual::{RitualManager, RitualStage, RitualStageStatus};
use amazon_rose_forest::darwin::self_improvement::{Modification, ModificationStatus};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_add_to_archive() {
    let metrics = Arc::new(MetricsCollector::new());
    let strategy = ExplorationStrategy::new(metrics.clone());
    let modification = Modification {
        id: Uuid::new_v4(),
        name: "test".into(),
        description: "desc".into(),
        code_changes: Vec::new(),
        validation_metrics: HashMap::new(),
        created_at: chrono::Utc::now(),
        status: ModificationStatus::Proposed,
    };
    let mut metric_map = HashMap::new();
    metric_map.insert("score".to_string(), 1.0);
    strategy
        .add_to_archive(modification, metric_map)
        .await
        .unwrap();
    let stats = strategy.get_archive_stats().await;
    assert_eq!(stats.total_entries, 1);
}

#[tokio::test]
async fn test_ritual_stage_dependencies() {
    let metrics = Arc::new(MetricsCollector::new());
    let manager = RitualManager::new(metrics);
    let stages = vec![
        RitualStage {
            name: "A".into(),
            description: "A".into(),
            status: RitualStageStatus::Pending,
            depends_on: vec![],
            artifacts: vec![],
            started_at: None,
            completed_at: None,
        },
        RitualStage {
            name: "B".into(),
            description: "B".into(),
            status: RitualStageStatus::Pending,
            depends_on: vec!["A".into()],
            artifacts: vec![],
            started_at: None,
            completed_at: None,
        },
    ];
    let ritual_id = manager.create_ritual("r", "d", stages).await.unwrap();
    manager.start_stage(ritual_id, "A").await.unwrap();
    // Starting B should fail because A not completed
    assert!(manager.start_stage(ritual_id, "B").await.is_err());
    manager
        .complete_stage(ritual_id, "A", vec![])
        .await
        .unwrap();
    assert!(manager.start_stage(ritual_id, "B").await.is_ok());
}
