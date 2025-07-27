use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::core::metrics::MetricsCollector;
use crate::core::vector::Vector;
use crate::sharding::migration::MigrationTask;
use crate::sharding::vector_index::{DistanceMetric, VectorIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShardStatus {
    Active,
    ReadOnly,
    Draining,
    Inactive,
}

#[derive(Debug, Clone)]
pub struct Shard {
    pub id: Uuid,
    pub name: String,
    pub status: ShardStatus,
    pub node_id: String,
    pub vector_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Shard load information for balancing
#[derive(Debug, Clone)]
pub struct ShardLoad {
    pub id: Uuid,
    pub vector_count: usize,
    pub query_rate: f32, // Queries per second
    pub memory_usage_mb: f32,
    pub cpu_usage_pct: f32,
}

#[derive(Debug)]
pub struct ShardManager {
    metrics: Arc<MetricsCollector>,
    node_id: String,
    shards: RwLock<HashMap<Uuid, Shard>>,
    shard_assignments: RwLock<HashMap<String, HashSet<Uuid>>>,
    migrations: RwLock<HashMap<Uuid, MigrationTask>>,
    indices: RwLock<HashMap<Uuid, Arc<VectorIndex>>>,
    shard_loads: RwLock<HashMap<Uuid, ShardLoad>>,
}

impl ShardManager {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        // Generate a random node ID if not provided
        let node_id = format!("node-{}", Uuid::new_v4());

        Self {
            metrics,
            node_id,
            shards: RwLock::new(HashMap::new()),
            shard_assignments: RwLock::new(HashMap::new()),
            migrations: RwLock::new(HashMap::new()),
            indices: RwLock::new(HashMap::new()),
            shard_loads: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_shard(&self, name: &str) -> Result<Uuid> {
        let shard_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let shard = Shard {
            id: shard_id,
            name: name.to_string(),
            status: ShardStatus::Active,
            node_id: self.node_id.clone(),
            vector_count: 0,
            created_at: now,
            updated_at: now,
        };

        // Store the shard
        self.shards.write().await.insert(shard_id, shard.clone());

        // Update assignments
        self.shard_assignments
            .write()
            .await
            .entry(self.node_id.clone())
            .or_insert_with(HashSet::new)
            .insert(shard_id);

        // Initialize shard load tracking
        self.shard_loads.write().await.insert(
            shard_id,
            ShardLoad {
                id: shard_id,
                vector_count: 0,
                query_rate: 0.0,
                memory_usage_mb: 0.0,
                cpu_usage_pct: 0.0,
            },
        );

        // Update metrics
        self.metrics.increment_counter("shards.created", 1).await;

        info!("Created new shard '{}' with ID: {}", name, shard_id);

        Ok(shard_id)
    }

    pub async fn create_vector_index(
        &self,
        shard_id: Uuid,
        name: &str,
        dimensions: usize,
        distance_metric: DistanceMetric,
    ) -> Result<Arc<VectorIndex>> {
        // Verify the shard exists
        self.get_shard(shard_id).await?;

        // Create the index
        let index = VectorIndex::new(
            name,
            dimensions,
            distance_metric,
            Some(self.metrics.clone()),
        )
        .map_err(|e| anyhow!("Failed to create vector index: {}", e))?;

        let index = Arc::new(index);

        // Store the index
        self.indices.write().await.insert(shard_id, index.clone());

        info!(
            "Created new vector index '{}' with {} dimensions for shard {}",
            name, dimensions, shard_id
        );

        Ok(index)
    }

    pub async fn get_vector_index(&self, shard_id: Uuid) -> Result<Arc<VectorIndex>> {
        let indices = self.indices.read().await;

        indices
            .get(&shard_id)
            .cloned()
            .ok_or_else(|| anyhow!("Vector index not found for shard {}", shard_id))
    }

    pub async fn add_vector(
        &self,
        shard_id: Uuid,
        vector: Vector,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Uuid> {
        // Get the index
        let index = self.get_vector_index(shard_id).await?;

        // Add the vector
        let id = index
            .add(vector, metadata)
            .await
            .map_err(|e| anyhow!("Failed to add vector: {}", e))?;

        // Update shard vector count
        {
            let mut shards = self.shards.write().await;
            if let Some(shard) = shards.get_mut(&shard_id) {
                shard.vector_count = index.count().await;
                shard.updated_at = chrono::Utc::now();
            }
        }

        // Update shard load info
        {
            let mut loads = self.shard_loads.write().await;
            if let Some(load) = loads.get_mut(&shard_id) {
                load.vector_count = index.count().await;
            }
        }

        Ok(id)
    }

    pub async fn search_vectors(
        &self,
        shard_id: Uuid,
        query: &Vector,
        limit: usize,
    ) -> Result<Vec<crate::sharding::vector_index::SearchResult>> {
        // Get the index
        let index = self.get_vector_index(shard_id).await?;

        // Search for vectors
        let results = index
            .search(query, limit)
            .await
            .map_err(|e| anyhow!("Failed to search vectors: {}", e))?;

        // Update query rate in shard load
        {
            let mut loads = self.shard_loads.write().await;
            if let Some(load) = loads.get_mut(&shard_id) {
                // Simple exponential moving average for query rate
                load.query_rate = load.query_rate * 0.9 + 0.1; // Add one query
            }
        }

        Ok(results)
    }

    pub async fn get_shard(&self, shard_id: Uuid) -> Result<Shard> {
        let shards = self.shards.read().await;

        shards
            .get(&shard_id)
            .cloned()
            .ok_or_else(|| anyhow!("Shard with ID {} not found", shard_id))
    }

    pub async fn get_shards(&self) -> Vec<Shard> {
        let shards = self.shards.read().await;
        shards.values().cloned().collect()
    }

    pub async fn update_shard_status(&self, shard_id: Uuid, status: ShardStatus) -> Result<()> {
        let mut shards = self.shards.write().await;

        let shard = shards
            .get_mut(&shard_id)
            .ok_or_else(|| anyhow!("Shard with ID {} not found", shard_id))?;

        shard.status = status.clone();
        shard.updated_at = chrono::Utc::now();

        info!("Updated shard {} status to {:?}", shard_id, status);

        Ok(())
    }

    pub async fn start_migration(
        self: Arc<Self>,
        shard_id: Uuid,
        target_node: &str,
    ) -> Result<Uuid> {
        // Verify the shard exists
        let shard = self.get_shard(shard_id).await?;

        // Create migration task
        let migration_id = Uuid::new_v4();
        let task = MigrationTask::new(
            migration_id,
            shard_id,
            self.node_id.clone(),
            target_node.to_string(),
        );

        // Store the migration
        self.migrations
            .write()
            .await
            .insert(migration_id, task.clone());

        // Update shard status
        self.update_shard_status(shard_id, ShardStatus::Draining)
            .await?;

        // Start the migration in the background
        let self_clone = Arc::clone(&self);
        tokio::spawn(async move {
            if let Err(e) = self_clone.execute_migration(migration_id).await {
                error!("Migration {} failed: {}", migration_id, e);
            }
        });

        info!(
            "Started migration {} for shard {} to node {}",
            migration_id, shard_id, target_node
        );

        Ok(migration_id)
    }

    async fn execute_migration(&self, migration_id: Uuid) -> Result<()> {
        // Get the migration task
        let task = {
            let migrations = self.migrations.read().await;
            migrations
                .get(&migration_id)
                .cloned()
                .ok_or_else(|| anyhow!("Migration task with ID {} not found", migration_id))?
        };

        // Simulate migration progress
        for progress in (0..=100).step_by(10) {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let mut migrations = self.migrations.write().await;
            if let Some(mut_task) = migrations.get_mut(&migration_id) {
                mut_task.progress = progress as f32 / 100.0;
                mut_task.updated_at = chrono::Utc::now();
            }
        }

        // Update assignments
        {
            let mut assignments = self.shard_assignments.write().await;

            // Remove from source
            if let Some(source_shards) = assignments.get_mut(&task.source_node) {
                source_shards.remove(&task.shard_id);
            }

            // Add to target
            assignments
                .entry(task.target_node.clone())
                .or_insert_with(HashSet::new)
                .insert(task.shard_id);
        }

        // Update shard info
        {
            let mut shards = self.shards.write().await;
            if let Some(shard) = shards.get_mut(&task.shard_id) {
                shard.node_id = task.target_node.clone();
                shard.status = ShardStatus::Active;
                shard.updated_at = chrono::Utc::now();
            }
        }

        // Mark migration as complete
        {
            let mut migrations = self.migrations.write().await;
            if let Some(mut_task) = migrations.get_mut(&migration_id) {
                mut_task.completed = true;
                mut_task.progress = 1.0;
                mut_task.updated_at = chrono::Utc::now();
            }
        }

        info!("Migration {} completed successfully", migration_id);

        Ok(())
    }

    pub async fn get_migration_status(&self, migration_id: Uuid) -> Result<(bool, f32)> {
        let migrations = self.migrations.read().await;

        migrations
            .get(&migration_id)
            .map(|task| (task.completed, task.progress))
            .ok_or_else(|| anyhow!("Migration with ID {} not found", migration_id))
    }

    /// Update shard load information
    pub async fn update_shard_load(
        &self,
        shard_id: Uuid,
        memory_mb: f32,
        cpu_pct: f32,
    ) -> Result<()> {
        let mut loads = self.shard_loads.write().await;

        let load = loads
            .get_mut(&shard_id)
            .ok_or_else(|| anyhow!("Shard load info for ID {} not found", shard_id))?;

        load.memory_usage_mb = memory_mb;
        load.cpu_usage_pct = cpu_pct;

        Ok(())
    }

    /// Get load information for all shards
    pub async fn get_shard_loads(&self) -> HashMap<Uuid, ShardLoad> {
        self.shard_loads.read().await.clone()
    }

    /// Find overloaded shards based on configurable thresholds
    pub async fn find_overloaded_shards(
        &self,
        memory_threshold_mb: f32,
        cpu_threshold_pct: f32,
        query_threshold_rate: f32,
    ) -> Vec<Uuid> {
        let loads = self.shard_loads.read().await;

        loads
            .values()
            .filter(|load| {
                load.memory_usage_mb > memory_threshold_mb
                    || load.cpu_usage_pct > cpu_threshold_pct
                    || load.query_rate > query_threshold_rate
            })
            .map(|load| load.id)
            .collect()
    }

    /// Hierarchical shard balancing
    pub async fn balance_shards(&self, nodes: Vec<String>) -> Result<HashMap<Uuid, String>> {
        if nodes.is_empty() {
            return Err(anyhow!("No nodes provided for balancing"));
        }

        // Get current loads
        let loads = self.shard_loads.read().await;
        let shards = self.shards.read().await;

        // Build a weighted distribution model
        let mut node_weights: HashMap<String, f32> = HashMap::new();
        for node in &nodes {
            node_weights.insert(node.clone(), 1.0); // Start with equal weights
        }

        // Calculate optimal shard distribution
        let mut distribution: HashMap<Uuid, String> = HashMap::new();

        // Sort shards by load (memory + CPU usage)
        let mut weighted_shards: Vec<(Uuid, f32)> = loads
            .values()
            .filter_map(|load| {
                shards.get(&load.id).map(|shard| {
                    // Calculate a weighted score based on resource usage
                    let weight = load.memory_usage_mb * 0.6
                        + load.cpu_usage_pct * 0.3
                        + load.query_rate * 0.1;
                    (load.id, weight)
                })
            })
            .collect();

        weighted_shards.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Distribute shards using hierarchical approach
        let mut node_loads: HashMap<String, f32> = HashMap::new();
        for node in &nodes {
            node_loads.insert(node.clone(), 0.0);
        }

        for (shard_id, weight) in weighted_shards {
            // Find the least loaded node
            let target_node = node_loads
                .iter()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(node, _)| node.clone())
                .unwrap_or_else(|| nodes[0].clone());

            // Assign shard to node
            distribution.insert(shard_id, target_node.clone());

            // Update node load
            if let Some(load) = node_loads.get_mut(&target_node) {
                *load += weight;
            }
        }

        info!(
            "Hierarchical shard balancing complete, recommended moves: {}",
            distribution.len()
        );

        Ok(distribution)
    }
}
