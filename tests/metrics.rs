use amazon_rose_forest::core::metrics::MetricsCollector;
use std::time::Duration;

#[tokio::test]
async fn test_increment_counter_and_get() {
    let metrics = MetricsCollector::new();
    metrics.increment_counter("hits", 1).await;
    metrics.increment_counter("hits", 4).await;

    let value = metrics.get_counter("hits").await;
    assert_eq!(value, Some(5));

    let series = metrics.get_timeseries("hits").await.unwrap();
    assert_eq!(series.metric_type, "counter");
    assert_eq!(series.values, vec![1.0, 4.0]);
}

#[tokio::test]
async fn test_set_gauge_and_get() {
    let metrics = MetricsCollector::new();
    metrics.set_gauge("temp", 10).await;
    assert_eq!(metrics.get_gauge("temp").await, Some(10));
    metrics.set_gauge("temp", 20).await;
    assert_eq!(metrics.get_gauge("temp").await, Some(20));

    let series = metrics.get_timeseries("temp").await.unwrap();
    assert_eq!(series.metric_type, "gauge");
    assert_eq!(series.values, vec![10.0, 20.0]);
}

#[tokio::test]
async fn test_record_histogram_and_stats() {
    let metrics = MetricsCollector::new();
    metrics.record_histogram("latency", 5).await;
    metrics.record_histogram("latency", 7).await;
    metrics.record_histogram("latency", 3).await;

    let stats = metrics.get_histogram_stats("latency").await.unwrap();
    assert_eq!(stats.count, 3);
    assert_eq!(stats.min, 3);
    assert_eq!(stats.max, 7);
    assert_eq!(stats.sum, 15);
    assert!((stats.mean - 5.0).abs() < f64::EPSILON);
    assert_eq!(stats.median, 5.0);
    assert_eq!(stats.p95, 7.0);
    assert_eq!(stats.p99, 7.0);
}

#[tokio::test]
async fn test_report_interval() {
    let metrics = MetricsCollector::new().with_report_interval(Duration::from_millis(100));
    assert!(metrics.report().await);
    assert!(!metrics.report().await);
    tokio::time::sleep(Duration::from_millis(110)).await;
    assert!(metrics.report().await);
}
