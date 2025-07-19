use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    pub count: usize,
    pub min: u64,
    pub max: u64,
    pub sum: u64,
    pub mean: f64,
    pub median: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self {
            count: 0,
            min: 0,
            max: 0,
            sum: 0,
            mean: 0.0,
            median: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTimeseries {
    pub timestamps: Vec<chrono::DateTime<chrono::Utc>>,
    pub values: Vec<f64>,
    pub name: String,
    pub metric_type: String,
}

#[derive(Debug)]
pub struct MetricsCollector {
    counters: DashMap<String, AtomicU64>,
    gauges: DashMap<String, Arc<AtomicU64>>,
    histograms: DashMap<String, Vec<u64>>,
    timeseries: DashMap<String, MetricTimeseries>,
    last_report: RwLock<Option<Instant>>,
    report_interval: Duration,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: DashMap::new(),
            gauges: DashMap::new(),
            histograms: DashMap::new(),
            timeseries: DashMap::new(),
            last_report: RwLock::new(None),
            report_interval: Duration::from_secs(60), // Default to 1 minute
        }
    }

    pub fn with_report_interval(mut self, interval: Duration) -> Self {
        self.report_interval = interval;
        self
    }

    pub async fn increment_counter(&self, name: &str, value: u64) {
        let mut entry = self
            .counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        entry.fetch_add(value, Ordering::Relaxed);

        debug!("Counter '{}' incremented by {}", name, value);

        self.record_timeseries(name, "counter", value as f64).await;
    }

    pub async fn set_gauge(&self, name: &str, value: u64) {
        let mut entry = self
            .gauges
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        entry.store(value, Ordering::Relaxed);

        debug!("Gauge '{}' set to {}", name, value);

        self.record_timeseries(name, "gauge", value as f64).await;
    }

    pub async fn record_histogram(&self, name: &str, value: u64) {
        let mut entry = self
            .histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        entry.push(value);

        debug!("Histogram '{}' recorded value {}", name, value);

        self.record_timeseries(name, "histogram", value as f64)
            .await;
    }

    async fn record_timeseries(&self, name: &str, metric_type: &str, value: f64) {
        let mut series =
            self.timeseries
                .entry(name.to_string())
                .or_insert_with(|| MetricTimeseries {
                    timestamps: Vec::new(),
                    values: Vec::new(),
                    name: name.to_string(),
                    metric_type: metric_type.to_string(),
                });

        series.timestamps.push(chrono::Utc::now());
        series.values.push(value);

        // Limit the number of points to keep memory usage in check
        const MAX_POINTS: usize = 1000;
        if series.timestamps.len() > MAX_POINTS {
            series.timestamps.remove(0);
            series.values.remove(0);
        }
    }

    pub async fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.get(name).map(|c| c.load(Ordering::Relaxed))
    }

    pub async fn get_gauge(&self, name: &str) -> Option<u64> {
        self.gauges.get(name).map(|g| g.load(Ordering::Relaxed))
    }

    pub async fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        self.histograms.get(name).map(|values| {
            let mut sorted = values.clone();
            sorted.sort_unstable();

            let count = sorted.len();
            if count == 0 {
                return HistogramStats::default();
            }

            let min = *sorted.first().unwrap();
            let max = *sorted.last().unwrap();
            let sum: u64 = sorted.iter().sum();
            let mean = (sum as f64) / (count as f64);

            let median = if count % 2 == 0 {
                (sorted[count / 2 - 1] + sorted[count / 2]) as f64 / 2.0
            } else {
                sorted[count / 2] as f64
            };

            let p95_idx = (count as f64 * 0.95) as usize;
            let p99_idx = (count as f64 * 0.99) as usize;

            HistogramStats {
                count,
                min,
                max,
                sum,
                mean,
                median,
                p95: sorted[p95_idx.min(count - 1)] as f64,
                p99: sorted[p99_idx.min(count - 1)] as f64,
            }
        })
    }

    pub async fn get_timeseries(&self, name: &str) -> Option<MetricTimeseries> {
        self.timeseries.get(name).map(|v| v.clone())
    }

    pub async fn get_all_timeseries(&self) -> Vec<MetricTimeseries> {
        self.timeseries
            .iter()
            .map(|kv| kv.value().clone())
            .collect()
    }

    pub async fn report(&self) -> bool {
        let mut should_report = false;
        {
            let mut last_report = self.last_report.write().await;
            let now = Instant::now();

            if let Some(last) = *last_report {
                if now.duration_since(last) >= self.report_interval {
                    *last_report = Some(now);
                    should_report = true;
                }
            } else {
                *last_report = Some(now);
                should_report = true;
            }
        }

        if !should_report {
            return false;
        }

        info!("Metrics Report:");

        for entry in self.counters.iter() {
            let value = entry.value().load(Ordering::Relaxed);
            info!("Counter {}: {}", entry.key(), value);
        }

        for entry in self.gauges.iter() {
            let value = entry.value().load(Ordering::Relaxed);
            info!("Gauge {}: {}", entry.key(), value);
        }

        let histogram_keys: Vec<String> = self.histograms.iter().map(|e| e.key().clone()).collect();
        for name in histogram_keys {
            if let Some(stats) = self.get_histogram_stats(&name).await {
                info!("Histogram {}: count={}, min={}, max={}, mean={:.2}, median={:.2}, p95={:.2}, p99={:.2}",
                    name, stats.count, stats.min, stats.max, stats.mean, stats.median, stats.p95, stats.p99);
            }
        }

        true
    }

    pub fn prometheus_metrics(&self) -> String {
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async { self.generate_prometheus_metrics().await })
        })
    }

    async fn generate_prometheus_metrics(&self) -> String {
        let mut output = String::new();

        // Add counters
        for entry in self.counters.iter() {
            let value = entry.value().load(Ordering::Relaxed);
            output.push_str(&format!("# TYPE {} counter\n", entry.key()));
            output.push_str(&format!("{} {}\n", entry.key(), value));
        }

        // Add gauges
        for entry in self.gauges.iter() {
            let value = entry.value().load(Ordering::Relaxed);
            output.push_str(&format!("# TYPE {} gauge\n", entry.key()));
            output.push_str(&format!("{} {}\n", entry.key(), value));
        }

        // Add histograms
        let histogram_keys: Vec<String> = self.histograms.iter().map(|e| e.key().clone()).collect();
        for name in histogram_keys {
            if let Some(stats) = self.get_histogram_stats(&name).await {
                output.push_str(&format!("# TYPE {}_sum gauge\n", name));
                output.push_str(&format!("{}_sum {}\n", name, stats.sum));

                output.push_str(&format!("# TYPE {}_count gauge\n", name));
                output.push_str(&format!("{}_count {}\n", name, stats.count));

                output.push_str(&format!("# TYPE {}_min gauge\n", name));
                output.push_str(&format!("{}_min {}\n", name, stats.min));

                output.push_str(&format!("# TYPE {}_max gauge\n", name));
                output.push_str(&format!("{}_max {}\n", name, stats.max));

                output.push_str(&format!("# TYPE {}_avg gauge\n", name));
                output.push_str(&format!("{}_avg {}\n", name, stats.mean));
            }
        }

        output
    }
}
