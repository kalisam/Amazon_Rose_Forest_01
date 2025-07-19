use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::server::{Server, ServerConfig};
use std::sync::Arc;
use warp::http::StatusCode;

#[tokio::test]
async fn stats_returns_metrics() {
    let metrics = Arc::new(MetricsCollector::new());
    let mut config = ServerConfig::default();
    config.port = 0;
    let server = Server::new(config, metrics, None, None);
    server.start().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let filter = server.filter();

    let res = warp::test::request()
        .method("GET")
        .path("/api/stats")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(res.body()).unwrap();
    assert_eq!(body["version"], amazon_rose_forest::VERSION);
    assert!(body["uptime_seconds"].as_u64().unwrap() > 0);
    assert!(body["memory_usage_mb"].as_u64().unwrap() > 0);
    server.stop().await.unwrap();
}
