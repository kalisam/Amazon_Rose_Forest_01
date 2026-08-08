use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::server::api::{SearchResult, SearchVectorsRequest};
use amazon_rose_forest::server::{Server, ServerConfig};
use amazon_rose_forest::{
    sharding::manager::ShardManager, sharding::vector_index::DistanceMetric, Vector,
};
use std::sync::Arc;
use warp::http::StatusCode;
use warp::ws::Message;
use warp::Filter;

use serde_json::Value;

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

    let mut client = warp::test::ws().path("/ws/search").handshake(filter).await;

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

#[tokio::test]
async fn enabled_endpoints_return_data() {
    let metrics = Arc::new(MetricsCollector::new());
    metrics.increment_counter("test_counter", 1).await;
    metrics.set_gauge("test_gauge", 5).await;

    let config = ServerConfig {
        address: "127.0.0.1".into(),
        port: 0,
        enable_metrics: true,
        metrics_path: "/metrics".into(),
        enable_api: true,
        api_path: "/api".into(),
    };

    let server = Server::new(config.clone(), metrics.clone(), None, None);
    let filter = server.routes(metrics, config, None, None);

    let resp = warp::test::request()
        .method("GET")
        .path("/metrics")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers()["content-type"], "text/plain; version=0.0.4");
    let body = std::str::from_utf8(resp.body()).unwrap();
    assert!(body.contains("test_counter"));
    assert!(body.contains("test_gauge"));

    let resp = warp::test::request()
        .method("GET")
        .path("/api/version")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body_json: Value = serde_json::from_slice(resp.body()).unwrap();
    assert_eq!(body_json["version"], amazon_rose_forest::VERSION);
}

#[tokio::test]
async fn api_vector_endpoints_work() {
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(ShardManager::new(metrics.clone()));
    let config = ServerConfig::default();
    let server = Server::new(config.clone(), metrics.clone(), None, Some(manager.clone()));
    let filter = server.routes(metrics, config, None, Some(manager.clone()));

    let create_req = amazon_rose_forest::server::api::CreateShardRequest {
        name: "test".into(),
    };
    let resp = warp::test::request()
        .method("POST")
        .path("/api/shards")
        .json(&create_req)
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let shard_resp: amazon_rose_forest::server::api::CreateShardResponse =
        serde_json::from_slice(resp.body()).unwrap();
    let shard_id = shard_resp.shard_id;

    let index_req = amazon_rose_forest::server::api::CreateIndexRequest {
        shard_id,
        name: "main".into(),
        dimensions: 3,
        distance_metric: "euclidean".into(),
    };
    let resp = warp::test::request()
        .method("POST")
        .path("/api/indexes")
        .json(&index_req)
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), StatusCode::OK);

    let add_req = amazon_rose_forest::server::api::AddVectorRequest {
        shard_id,
        vector: vec![0.0, 0.0, 0.0],
        metadata: None,
    };
    let resp = warp::test::request()
        .method("POST")
        .path("/api/vectors")
        .json(&add_req)
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), StatusCode::OK);

    let search_req = SearchVectorsRequest {
        shard_id,
        query_vector: vec![0.0, 0.0, 0.0],
        limit: 1,
    };
    let resp = warp::test::request()
        .method("POST")
        .path("/api/search")
        .json(&search_req)
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let search_resp: amazon_rose_forest::server::api::SearchVectorsResponse =
        serde_json::from_slice(resp.body()).unwrap();
    assert_eq!(search_resp.results.len(), 1);
}
