use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::server::{Server, ServerConfig};
use std::sync::Arc;
use warp::http::StatusCode;
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
