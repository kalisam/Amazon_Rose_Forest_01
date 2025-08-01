#[rustfmt::skip]
use crate::core::metrics::MetricsCollector;
use crate::nerv::runtime::Runtime;
use crate::sharding::manager::ShardManager;
use anyhow::{anyhow, Result};
use prometheus::{Encoder, Registry, TextEncoder};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock as StdRwLock};
use std::time::Instant;
use sysinfo::{get_current_pid, ProcessExt, System, SystemExt};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use warp::ws::{Message, WebSocket};
use futures::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use crate::server::api::{SearchVectorsRequest, ErrorResponse, create_vector, convert_search_results};
use uuid::Uuid;
use warp::{Filter, Reply};

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
    start_time: Arc<StdRwLock<Option<Instant>>>,
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
            start_time: Arc::new(StdRwLock::new(None)),
        }
    }

    /// Start the server
    pub async fn start(&mut self) -> Result<()> {
        *self.start_time.write().unwrap() = Some(Instant::now());
        let addr = format!("{}:{}", self.config.address, self.config.port);
        let addr: SocketAddr = addr.parse()?;
      
        let server = warp::serve(self.filter());

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

    async fn handle_ws_search(socket: WebSocket, manager: Arc<ShardManager>) {
        let (mut tx_ws, mut rx_ws) = socket.split();
        while let Some(Ok(msg)) = rx_ws.next().await {
            if !msg.is_text() {
                continue;
            }
            let req: Result<SearchVectorsRequest, _> = serde_json::from_str(msg.to_str().unwrap());
            let req = match req {
                Ok(r) => r,
                Err(e) => {
                    let err = ErrorResponse { error: e.to_string() };
                    let _ = tx_ws.send(Message::text(serde_json::to_string(&err).unwrap())).await;
                    continue;
                }
            };

            let query = create_vector(req.query_vector.clone());
            let results = match manager.search_vectors(req.shard_id, &query, req.limit).await {
                Ok(r) => r,
                Err(e) => {
                    let err = ErrorResponse { error: e.to_string() };
                    let _ = tx_ws.send(Message::text(serde_json::to_string(&err).unwrap())).await;
                    continue;
                }
            };

            let api_results = convert_search_results(results);
            let (tx, mut rx) = broadcast::channel::<String>(16);
            let mut tx_ws_clone = tx_ws.clone();
            let forward = tokio::spawn(async move {
                while let Ok(res) = rx.recv().await {
                    if tx_ws_clone.send(Message::text(res)).await.is_err() {
                        break;
                    }
                }
            });
            for r in api_results {
                let _ = tx.send(serde_json::to_string(&r).unwrap());
            }
            drop(tx);
            let _ = forward.await;
        }
    }


    /// Get the Warp filter for this server
    pub fn filter(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        self.routes(
            self.metrics.clone(),
            self.config.clone(),
            self.runtime.clone(),
            self.shard_manager.clone(),
            self.start_time.clone(),
        )
    }

    /// Create the server routes
    pub fn routes(
        &self,
        metrics: Arc<MetricsCollector>,
        config: ServerConfig,
        runtime: Option<Arc<Runtime>>,
        shard_manager: Option<Arc<ShardManager>>,
        start_time: Arc<StdRwLock<Option<Instant>>>,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let health_route = warp::path("health").map(move || {
            debug!("Health check request received");
            warp::reply::json(&serde_json::json!({
                "status": "ok",
                "version": crate::VERSION,
            }))
        });

        let metrics_path = config.metrics_path.trim_start_matches('/').to_string();
        let metrics_route = if config.enable_metrics {
            let metrics_clone = metrics.clone();
            warp::path(metrics_path.clone())
                .map(move || {
                    debug!("Metrics request received");
                    let prometheus_metrics = metrics_clone.prometheus_metrics();
                    warp::reply::with_header(
                        prometheus_metrics,
                        "Content-Type",
                        "text/plain; version=0.0.4",
                    )
                    .into_response()
                })
                .boxed()
        } else {
            warp::path(metrics_path)
                .map(|| {
                    warp::reply::with_status(
                        "Metrics endpoint disabled",
                        warp::http::StatusCode::NOT_FOUND,
                    )
                    .into_response()
                })
                .boxed()
        };

        let api_path = config.api_path.trim_start_matches('/').to_string();
        let api_routes = if config.enable_api {
            // API version endpoint
            let version_route = warp::path(api_path.clone())
                .and(warp::path("version"))
                .map(|| {
                    warp::reply::json(&serde_json::json!({
                        "version": crate::VERSION,
                    }))
                    .into_response()
                })
                .boxed();

            // Statistics endpoint
            let stats_start_time = start_time.clone();
            let stats_route = warp::path(api_path)
                .and(warp::path("stats"))
                .map(move || {
                    let mut sys = System::new();
                    let pid = get_current_pid().unwrap();
                    sys.refresh_process(pid);
                    let mem_mb = sys.process(pid).map(|p| p.memory() / 1024).unwrap_or(0);
                    let uptime_seconds = if let Some(start) = *stats_start_time.read().unwrap() {
                        start.elapsed().as_secs()
                    } else {
                        0
                    };
                    let stats = serde_json::json!({
                        "version": crate::VERSION,
                        "uptime_seconds": uptime_seconds,
                        "memory_usage_mb": mem_mb,
                    });

                    warp::reply::json(&stats).into_response()
                })
                .boxed();
            version_route.or(stats_route).unify().boxed()
        } else {
            warp::path(api_path)
                .map(|| {
                    warp::reply::with_status(
                        "API endpoint disabled",
                        warp::http::StatusCode::NOT_FOUND,
                    )
                    .into_response()
                })
                .boxed()
        };

        let ws_search_route = {
            let manager_opt = shard_manager.clone();
            warp::path("ws")
                .and(warp::path("search"))
                .and(warp::ws())
                .map(move |ws: warp::ws::Ws| {
                    let manager_clone = manager_opt.clone();
                    ws.on_upgrade(move |socket| async move {
                        if let Some(manager) = manager_clone {
                            Server::handle_ws_search(socket, manager).await;
                        }
                    })
                })
                .boxed()
        };

        health_route
            .or(metrics_route)
            .or(api_routes)
            .or(ws_search_route)
    }
}
