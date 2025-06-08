use std::net::SocketAddr;
use std::sync::Arc;
use prometheus::{Registry, TextEncoder, Encoder};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use anyhow::{Result, anyhow};
use tracing::{info, error, debug, warn};
use warp;

use crate::core::metrics::MetricsCollector;
use crate::nerv::runtime::Runtime;
use crate::sharding::manager::ShardManager;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind the server to
    pub address: String,
    
    /// Port for the HTTP server
    pub port: u16,
    
    /// Whether to enable the metrics endpoint
    pub enable_metrics: bool,
    
    /// Path for the metrics endpoint
    pub metrics_path: String,
    
    /// Whether to enable the API endpoint
    pub enable_api: bool,
    
    /// Path for the API endpoint
    pub api_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 9000,
            enable_metrics: true,
            metrics_path: "/metrics".to_string(),
            enable_api: true,
            api_path: "/api".to_string(),
        }
    }
}

/// HTTP server for metrics and API
pub struct Server {
    config: ServerConfig,
    metrics: Arc<MetricsCollector>,
    runtime: Option<Arc<Runtime>>,
    shard_manager: Option<Arc<ShardManager>>,
    server_handle: RwLock<Option<JoinHandle<Result<()>>>>,
}

impl Server {
    /// Create a new server
    pub fn new(
        config: ServerConfig,
        metrics: Arc<MetricsCollector>,
        runtime: Option<Arc<Runtime>>,
        shard_manager: Option<Arc<ShardManager>>,
    ) -> Self {
        Self {
            config,
            metrics,
            runtime,
            shard_manager,
            server_handle: RwLock::new(None),
        }
    }
    
    /// Start the server
    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.address, self.config.port);
        let addr: SocketAddr = addr.parse()?;
        
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        let runtime = self.runtime.clone();
        let shard_manager = self.shard_manager.clone();
        
        let server = warp::serve(self.routes(metrics, config, runtime, shard_manager));
        
        info!("Starting server on {}", addr);
        
        let (_, server_handle) = server.bind_with_graceful_shutdown(addr, async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for CTRL+C");
                
            info!("Received shutdown signal, stopping server...");
        });
        
        // Store server handle
        let mut handle = self.server_handle.write().await;
        *handle = Some(tokio::spawn(async move {
            server_handle.await;
            Ok(())
        }));
        
        Ok(())
    }
    
    /// Stop the server
    pub async fn stop(&self) -> Result<()> {
        let handle = {
            let mut handle = self.server_handle.write().await;
            handle.take()
        };
        
        if let Some(handle) = handle {
            handle.abort();
            info!("Server stopped");
        } else {
            warn!("Server was not running");
        }
        
        Ok(())
    }
    
    /// Create the server routes
    fn routes(
        &self,
        metrics: Arc<MetricsCollector>,
        config: ServerConfig,
        runtime: Option<Arc<Runtime>>,
        shard_manager: Option<Arc<ShardManager>>,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let health_route = warp::path("health")
            .map(move || {
                debug!("Health check request received");
                warp::reply::json(&serde_json::json!({
                    "status": "ok",
                    "version": crate::VERSION,
                }))
            });
            
        let metrics_route = if config.enable_metrics {
            let metrics_clone = metrics.clone();
            warp::path(config.metrics_path.trim_start_matches('/'))
                .map(move || {
                    debug!("Metrics request received");
                    let prometheus_metrics = metrics_clone.prometheus_metrics();
                    warp::reply::with_header(
                        prometheus_metrics,
                        "Content-Type",
                        "text/plain; version=0.0.4",
                    )
                })
                .boxed()
        } else {
            warp::any()
                .and(warp::path(config.metrics_path.trim_start_matches('/')))
                .map(|| {
                    warp::reply::with_status(
                        "Metrics endpoint disabled",
                        warp::http::StatusCode::NOT_FOUND,
                    )
                })
                .boxed()
        };
        
        let api_routes = if config.enable_api {
            let api_path = config.api_path.trim_start_matches('/').to_string();
            
            // API version endpoint
            let version_route = warp::path(api_path.clone())
                .and(warp::path("version"))
                .map(|| {
                    warp::reply::json(&serde_json::json!({
                        "version": crate::VERSION,
                    }))
                });
                
            // Statistics endpoint
            let stats_route = warp::path(api_path)
                .and(warp::path("stats"))
                .map(move || {
                    let stats = serde_json::json!({
                        "version": crate::VERSION,
                        "uptime_seconds": 0, // TODO: Add actual uptime
                        "memory_usage_mb": 0, // TODO: Add actual memory usage
                    });
                    
                    warp::reply::json(&stats)
                });
                
            version_route.or(stats_route).boxed()
        } else {
            warp::any()
                .and(warp::path(config.api_path.trim_start_matches('/')))
                .map(|| {
                    warp::reply::with_status(
                        "API endpoint disabled",
                        warp::http::StatusCode::NOT_FOUND,
                    )
                })
                .boxed()
        };
        
        health_route.or(metrics_route).or(api_routes)
    }
}