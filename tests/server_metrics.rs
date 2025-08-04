use amazon_rose_forest::core::metrics::MetricsCollector;
use amazon_rose_forest::server::metrics::{HTTP_REQUEST_DURATION, INCOMING_REQUESTS};
use amazon_rose_forest::server::{Server, ServerConfig};
use std::sync::Arc;
use warp::Filter;

#[tokio::test]
async fn records_request_metrics() {
    INCOMING_REQUESTS.reset();
    HTTP_REQUEST_DURATION.reset();

    let metrics = Arc::new(MetricsCollector::new());
    let server = Server::new(ServerConfig::default(), metrics, None, None);

    // Health endpoint
    let filter = server.filter();
    let resp = warp::test::request()
        .method("GET")
        .path("/health")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), warp::http::StatusCode::OK);
    assert_eq!(
        INCOMING_REQUESTS
            .with_label_values(&["GET", "/health"])
            .get(),
        1.0
    );
    assert_eq!(
        HTTP_REQUEST_DURATION
            .with_label_values(&["GET", "/health", "200"])
            .get_sample_count(),
        1
    );

    // Metrics endpoint
    let filter = server.filter();
    let resp = warp::test::request()
        .method("GET")
        .path("/metrics")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), warp::http::StatusCode::OK);
    assert_eq!(
        INCOMING_REQUESTS
            .with_label_values(&["GET", "/metrics"])
            .get(),
        1.0
    );
    assert_eq!(
        HTTP_REQUEST_DURATION
            .with_label_values(&["GET", "/metrics", "200"])
            .get_sample_count(),
        1
    );

    // API version endpoint with POST
    let filter = server.filter();
    let resp = warp::test::request()
        .method("POST")
        .path("/api/version")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), warp::http::StatusCode::OK);
    assert_eq!(
        INCOMING_REQUESTS
            .with_label_values(&["POST", "/api/version"])
            .get(),
        1.0
    );
    assert_eq!(
        HTTP_REQUEST_DURATION
            .with_label_values(&["POST", "/api/version", "200"])
            .get_sample_count(),
        1
    );

    // WebSocket handshake
    let filter = server.filter();
    let resp = warp::test::request()
        .method("GET")
        .path("/ws/search")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), warp::http::StatusCode::SWITCHING_PROTOCOLS);
    assert_eq!(
        INCOMING_REQUESTS
            .with_label_values(&["GET", "/ws/search"])
            .get(),
        1.0
    );
    assert_eq!(
        HTTP_REQUEST_DURATION
            .with_label_values(&["GET", "/ws/search", "101"])
            .get_sample_count(),
        1
    );

    // Disabled metrics endpoint should return 404
    INCOMING_REQUESTS.reset();
    HTTP_REQUEST_DURATION.reset();
    let mut cfg = ServerConfig::default();
    cfg.enable_metrics = false;
    let server_disabled = Server::new(cfg, Arc::new(MetricsCollector::new()), None, None);
    let filter = server_disabled.filter();
    let resp = warp::test::request()
        .method("GET")
        .path("/metrics")
        .reply(&filter)
        .await;
    assert_eq!(resp.status(), warp::http::StatusCode::NOT_FOUND);
    assert_eq!(
        INCOMING_REQUESTS
            .with_label_values(&["GET", "/metrics"])
            .get(),
        1.0
    );
    assert_eq!(
        HTTP_REQUEST_DURATION
            .with_label_values(&["GET", "/metrics", "404"])
            .get_sample_count(),
        1
    );
}
