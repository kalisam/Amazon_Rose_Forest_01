use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Debug)]
pub struct MetricsCollector {
    counters: RwLock<HashMap<String, AtomicU64>>,
    gauges: RwLock<HashMap<String, Arc<AtomicU64>>>,
    histograms: RwLock<HashMap<String, Vec<u64>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
        }
    }

    pub async fn increment_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.write().await;
        counters.entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(value, Ordering::Relaxed);
            
        debug!("Counter '{}' incremented by {}", name, value);
    }
    
    pub async fn set_gauge(&self, name: &str, value: u64) {
        let mut gauges = self.gauges.write().await;
        gauges.entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .store(value, Ordering::Relaxed);
            
        debug!("Gauge '{}' set to {}", name, value);
    }
    
    pub async fn record_histogram(&self, name: &str, value: u64) {
        let mut histograms = self.histograms.write().await;
        histograms.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
            
        debug!("Histogram '{}' recorded value {}", name, value);
    }
    
    pub async fn get_counter(&self, name: &str) -> Option<u64> {
        let counters = self.counters.read().await;
        counters.get(name).map(|c| c.load(Ordering::Relaxed))
    }
    
    pub async fn get_gauge(&self, name: &str) -> Option<u64> {
        let gauges = self.gauges.read().await;
        gauges.get(name).map(|g| g.load(Ordering::Relaxed))
    }
    
    pub async fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        let histograms = self.histograms.read().await;
        histograms.get(name).map(|values| {
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
    
    pub async fn report(&self) {
        info!("Metrics Report:");
        
        let counters = self.counters.read().await;
        for (name, counter) in counters.iter() {
            let value = counter.load(Ordering::Relaxed);
            info!("Counter {}: {}", name, value);
        }
        
        let gauges = self.gauges.read().await;
        for (name, gauge) in gauges.iter() {
            let value = gauge.load(Ordering::Relaxed);
            info!("Gauge {}: {}", name, value);
        }
        
        let histograms = self.histograms.read().await;
        for (name, _) in histograms.iter() {
            if let Some(stats) = self.get_histogram_stats(name).await {
                info!("Histogram {}: count={}, min={}, max={}, mean={:.2}, median={:.2}, p95={:.2}, p99={:.2}",
                    name, stats.count, stats.min, stats.max, stats.mean, stats.median, stats.p95, stats.p99);
            }
        }
    }
}

#[derive(Debug, Default)]
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