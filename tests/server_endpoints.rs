use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::server::{Server, ServerConfig};
use amazon_rose_forest::server::api::{SearchVectorsRequest, SearchResult};
use amazon_rose_forest::{Vector, sharding::manager::ShardManager, sharding::vector_index::DistanceMetric};
use warp::ws::Message;
use std::sync::Arc;
use warp::Filter;

#[tokio::test]
async fn disabled_endpoints_return_404() {
    let metrics = Arc::new(MetricsCollector::new());
    let config = ServerConfig {
        address: "127.0.0.1".into(),
        port: 0,
        enable_metrics: false,
        metrics_path: "/metrics".into(),
        enable_api: false,
        api_path: "/api".into(),
    };

    let server = Server::new(config.clone(), metrics.clone(), None, None);
    let filter = server.routes(metrics, config, None, None);

    let resp = warp::test::request()
        .method("GET")
        .path("/metrics")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 404);
    assert_eq!(resp.body(), "Metrics endpoint disabled");

    let resp = warp::test::request()
        .method("GET")
        .path("/api/version")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), 404);
    assert_eq!(resp.body(), "API endpoint disabled");
}

#[tokio::test]
async fn websocket_search_returns_results() {
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(ShardManager::new(metrics.clone()));
    let shard_id = manager.create_shard("test").await.unwrap();
    manager
        .create_vector_index(shard_id, "main", 3, DistanceMetric::Euclidean)
        .await
        .unwrap();

    manager
        .add_vector(shard_id, Vector::new(vec![0.0, 0.0, 0.0]), None)
        .await
        .unwrap();
    manager
        .add_vector(shard_id, Vector::new(vec![1.0, 1.0, 1.0]), None)
        .await
        .unwrap();

    let config = ServerConfig::default();
    let server = Server::new(config.clone(), metrics.clone(), None, Some(manager.clone()));
    let filter = server.routes(metrics, config, None, Some(manager));

    let mut client = warp::test::ws()
        .path("/ws/search")
        .handshake(filter)
        .await;

    let req = SearchVectorsRequest {
        shard_id,
        query_vector: vec![0.0, 0.0, 0.0],
        limit: 1,
    };
    client
        .send(Message::text(serde_json::to_string(&req).unwrap()))
        .await;

    let msg = client.recv().await.unwrap();
    assert!(msg.is_text());
    let _res: SearchResult = serde_json::from_str(msg.to_str().unwrap()).unwrap();
}
