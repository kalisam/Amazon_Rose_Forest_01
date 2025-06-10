use amazon_rose_forest::{
    core::metrics::MetricsCollector,
    sharding::{manager::ShardManager, vector_index::DistanceMetric},
    Vector,
};
use std::sync::Arc;

#[tokio::test]
async fn test_shard_creation_and_search() {
    let metrics = Arc::new(MetricsCollector::new());
    let manager = ShardManager::new(metrics);

    let shard_id = manager.create_shard("test_shard").await.unwrap();
    manager
        .create_vector_index(shard_id, "main", 3, DistanceMetric::Euclidean)
        .await
        .unwrap();

    for _ in 0..5 {
        let v = Vector::random(3);
        manager.add_vector(shard_id, v, None).await.unwrap();
    }

    let results = manager
        .search_vectors(shard_id, &Vector::random(3), 3)
        .await
        .unwrap();

    assert_eq!(results.len(), 3);

    let shard = manager.get_shard(shard_id).await.unwrap();
    assert_eq!(shard.vector_count, 5);
}
