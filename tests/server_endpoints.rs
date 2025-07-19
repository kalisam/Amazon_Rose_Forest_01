use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::server::{Server, ServerConfig};
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
