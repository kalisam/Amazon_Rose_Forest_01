#[rustfmt::skip]
use crate::core::metrics::MetricsCollector;
use crate::nerv::runtime::Runtime;
use crate::server::api::{
    convert_search_results, create_vector, parse_distance_metric, AddVectorRequest,
    AddVectorResponse, CreateIndexRequest, CreateIndexResponse, CreateShardRequest,
    CreateShardResponse, ErrorResponse, SearchVectorsRequest, SearchVectorsResponse,
};
use crate::sharding::manager::ShardManager;
use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use prometheus::{Encoder, Registry, TextEncoder};
use serde::de::DeserializeOwned;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock as StdRwLock};
use std::time::Instant;
use sysinfo::{get_current_pid, ProcessExt, System, SystemExt};
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::{Filter, Reply};

fn json_body<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
where
    T: DeserializeOwned + Send + 'static,
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

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
                    let err = ErrorResponse {
                        error: e.to_string(),
                    };
                    let _ = tx_ws
                        .send(Message::text(serde_json::to_string(&err).unwrap()))
                        .await;
                    continue;
                }
            };

            let query = create_vector(req.query_vector.clone());
            let results = match manager
                .search_vectors(req.shard_id, &query, req.limit)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let err = ErrorResponse {
                        error: e.to_string(),
                    };
                    let _ = tx_ws
                        .send(Message::text(serde_json::to_string(&err).unwrap()))
                        .await;
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
            let stats_route = warp::path(api_path.clone())
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

            let manager_for_create = shard_manager.clone();
            let create_shard = warp::path(api_path.clone())
                .and(warp::path("shards"))
                .and(warp::post())
                .and(json_body::<CreateShardRequest>())
                .and_then(move |req: CreateShardRequest| {
                    let manager_opt = manager_for_create.clone();
                    async move {
                        if let Some(manager) = manager_opt {
                            match manager.create_shard(&req.name).await {
                                Ok(id) => Ok::<_, warp::Rejection>(
                                    warp::reply::json(&CreateShardResponse { shard_id: id })
                                        .into_response(),
                                ),
                                Err(e) => Ok(warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse {
                                        error: e.to_string(),
                                    }),
                                    warp::http::StatusCode::BAD_REQUEST,
                                )
                                .into_response()),
                            }
                        } else {
                            Ok::<_, warp::Rejection>(
                                warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse {
                                        error: "Shard manager not configured".into(),
                                    }),
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                )
                                .into_response(),
                            )
                        }
                    }
                })
                .boxed();

            let manager_for_index = shard_manager.clone();
            let create_index = warp::path(api_path.clone())
                .and(warp::path("indexes"))
                .and(warp::post())
                .and(json_body::<CreateIndexRequest>())
                .and_then(move |req: CreateIndexRequest| {
                    let manager_opt = manager_for_index.clone();
                    async move {
                        if let Some(manager) = manager_opt {
                            match parse_distance_metric(&req.distance_metric) {
                                Ok(metric) => match manager
                                    .create_vector_index(
                                        req.shard_id,
                                        &req.name,
                                        req.dimensions,
                                        metric,
                                    )
                                    .await
                                {
                                    Ok(_) => Ok::<_, warp::Rejection>(
                                        warp::reply::json(&CreateIndexResponse {
                                            shard_id: req.shard_id,
                                            index_name: req.name,
                                            dimensions: req.dimensions,
                                            distance_metric: req.distance_metric.to_lowercase(),
                                        })
                                        .into_response(),
                                    ),
                                    Err(e) => Ok(warp::reply::with_status(
                                        warp::reply::json(&ErrorResponse {
                                            error: e.to_string(),
                                        }),
                                        warp::http::StatusCode::BAD_REQUEST,
                                    )
                                    .into_response()),
                                },
                                Err(e) => Ok(warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse { error: e }),
                                    warp::http::StatusCode::BAD_REQUEST,
                                )
                                .into_response()),
                            }
                        } else {
                            Ok::<_, warp::Rejection>(
                                warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse {
                                        error: "Shard manager not configured".into(),
                                    }),
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                )
                                .into_response(),
                            )
                        }
                    }
                })
                .boxed();

            let manager_for_add = shard_manager.clone();
            let add_vector = warp::path(api_path.clone())
                .and(warp::path("vectors"))
                .and(warp::post())
                .and(json_body::<AddVectorRequest>())
                .and_then(move |req: AddVectorRequest| {
                    let manager_opt = manager_for_add.clone();
                    async move {
                        if let Some(manager) = manager_opt {
                            let vector = create_vector(req.vector);
                            match manager.add_vector(req.shard_id, vector, req.metadata).await {
                                Ok(id) => Ok::<_, warp::Rejection>(
                                    warp::reply::json(&AddVectorResponse { vector_id: id })
                                        .into_response(),
                                ),
                                Err(e) => Ok(warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse {
                                        error: e.to_string(),
                                    }),
                                    warp::http::StatusCode::BAD_REQUEST,
                                )
                                .into_response()),
                            }
                        } else {
                            Ok::<_, warp::Rejection>(
                                warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse {
                                        error: "Shard manager not configured".into(),
                                    }),
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                )
                                .into_response(),
                            )
                        }
                    }
                })
                .boxed();

            let manager_for_search = shard_manager.clone();
            let search_vectors = warp::path(api_path.clone())
                .and(warp::path("search"))
                .and(warp::post())
                .and(json_body::<SearchVectorsRequest>())
                .and_then(move |req: SearchVectorsRequest| {
                    let manager_opt = manager_for_search.clone();
                    async move {
                        if let Some(manager) = manager_opt {
                            if req.limit == 0 {
                                return Ok::<_, warp::Rejection>(warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse { error: "limit must be greater than zero".into() }),
                                    warp::http::StatusCode::BAD_REQUEST,
                                ).into_response());
                            }
                            let query = create_vector(req.query_vector);
                            if let Ok(index) = manager.get_vector_index(req.shard_id).await {
                                let stats = index.stats().await;
                                if query.dimensions != stats.dimensions {
                                    return Ok::<_, warp::Rejection>(warp::reply::with_status(
                                        warp::reply::json(&ErrorResponse {
                                            error: format!(
                                                "Query vector dimensions mismatch: expected {}, got {}",
                                                stats.dimensions, query.dimensions
                                            ),
                                        }),
                                        warp::http::StatusCode::BAD_REQUEST,
                                    ).into_response());
                                }
                            } else {
                                return Ok::<_, warp::Rejection>(warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse { error: "Vector index not found".into() }),
                                    warp::http::StatusCode::BAD_REQUEST,
                                ).into_response());
                            }
                            match manager.search_vectors(req.shard_id, &query, req.limit).await {
                                Ok(results) => {
                                    let results = convert_search_results(results);
                                    Ok::<_, warp::Rejection>(warp::reply::json(&SearchVectorsResponse { results }).into_response())
                                }
                                Err(e) => Ok(warp::reply::with_status(
                                    warp::reply::json(&ErrorResponse { error: e.to_string() }),
                                    warp::http::StatusCode::BAD_REQUEST,
                                ).into_response()),
                            }
                        } else {
                            Ok::<_, warp::Rejection>(warp::reply::with_status(
                                warp::reply::json(&ErrorResponse { error: "Shard manager not configured".into() }),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            ).into_response())
                        }
                    }
                })
                .boxed();

            version_route
                .or(stats_route)
                .or(create_shard)
                .or(create_index)
                .or(add_vector)
                .or(search_vectors)
                .unify()
                .boxed()
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
