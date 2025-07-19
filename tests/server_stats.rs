use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::server::{Server, ServerConfig};
use std::sync::Arc;
use warp::http::StatusCode;

#[tokio::test]
async fn stats_returns_metrics() {
    let metrics = Arc::new(MetricsCollector::new());
    let server = Server::new(ServerConfig::default(), metrics, None, None);
    let filter = server.filter();

    let res = warp::test::request()
        .method("GET")
        .path("/api/stats")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(res.body()).unwrap();
    assert_eq!(body["version"], amazon_rose_forest::VERSION);
    assert!(body["uptime_seconds"].as_u64().unwrap() >= 0);
}
