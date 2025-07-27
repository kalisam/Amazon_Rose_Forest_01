use crate::core::metrics::MetricsCollector;
use crate::sharding::manager::ShardManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

#[derive(Debug)]
pub struct Runtime {
    metrics: Arc<MetricsCollector>,
    shard_manager: Option<Arc<ShardManager>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl Runtime {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            shard_manager: None,
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Amazon Rose Forest runtime...");

        // Start the shard manager
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Initialize shard manager
        let shard_manager = ShardManager::new(self.metrics.clone());
        self.shard_manager = Some(Arc::new(shard_manager));

        // Start the background task
        let metrics = self.metrics.clone();
        let shard_manager = self.shard_manager.clone();

        tokio::spawn(async move {
            info!("Runtime background task started");

            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping runtime");
                }
            }

            info!("Runtime background task stopping");
        });

        info!("Amazon Rose Forest runtime started successfully");

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Amazon Rose Forest runtime...");

        if let Some(tx) = &self.shutdown_tx {
            if let Err(e) = tx.send(()).await {
                error!("Failed to send shutdown signal: {}", e);
            }
        }

        info!("Amazon Rose Forest runtime stopped");
        Ok(())
    }

    pub fn metrics(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }

    pub fn shard_manager(&self) -> Option<Arc<ShardManager>> {
        self.shard_manager.clone()
    }

    /// Expose the shutdown sender for testing and external monitoring
    pub fn shutdown_sender(&self) -> Option<mpsc::Sender<()>> {
        self.shutdown_tx.clone()
    }
}
