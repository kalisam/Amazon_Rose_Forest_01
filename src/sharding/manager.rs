use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};

use crate::core::metrics::MetricsCollector;
use crate::sharding::migration::MigrationTask;
use crate::sharding::vector_index::{VectorIndex, DistanceMetric};
use crate::core::vector::Vector;

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

#[derive(Debug)]
pub struct ShardManager {
    metrics: Arc<MetricsCollector>,
    node_id: String,
    shards: RwLock<HashMap<Uuid, Shard>>,
    shard_assignments: RwLock<HashMap<String, HashSet<Uuid>>>,
    migrations: RwLock<HashMap<Uuid, MigrationTask>>,
    indices: RwLock<HashMap<Uuid, Arc<VectorIndex>>>,
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
        self.shard_assignments.write().await
            .entry(self.node_id.clone())
            .or_insert_with(HashSet::new)
            .insert(shard_id);
            
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
        );
        
        let index = Arc::new(index);
        
        // Store the index
        self.indices.write().await.insert(shard_id, index.clone());
        
        info!("Created new vector index '{}' with {} dimensions for shard {}", 
              name, dimensions, shard_id);
        
        Ok(index)
    }
    
    pub async fn get_vector_index(&self, shard_id: Uuid) -> Result<Arc<VectorIndex>> {
        let indices = self.indices.read().await;
        
        indices.get(&shard_id)
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
        let id = index.add(vector, metadata).await
            .map_err(|e| anyhow!("Failed to add vector: {}", e))?;
        
        // Update shard vector count
        {
            let mut shards = self.shards.write().await;
            if let Some(shard) = shards.get_mut(&shard_id) {
                shard.vector_count = index.count().await;
                shard.updated_at = chrono::Utc::now();
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
        let results = index.search(query, limit).await
            .map_err(|e| anyhow!("Failed to search vectors: {}", e))?;
        
        Ok(results)
    }
    
    pub async fn get_shard(&self, shard_id: Uuid) -> Result<Shard> {
        let shards = self.shards.read().await;
        
        shards.get(&shard_id)
            .cloned()
            .ok_or_else(|| anyhow!("Shard with ID {} not found", shard_id))
    }
    
    pub async fn get_shards(&self) -> Vec<Shard> {
        let shards = self.shards.read().await;
        shards.values().cloned().collect()
    }
    
    pub async fn update_shard_status(&self, shard_id: Uuid, status: ShardStatus) -> Result<()> {
        let mut shards = self.shards.write().await;
        
        let shard = shards.get_mut(&shard_id)
            .ok_or_else(|| anyhow!("Shard with ID {} not found", shard_id))?;
            
        shard.status = status.clone();
        shard.updated_at = chrono::Utc::now();
        
        info!("Updated shard {} status to {:?}", shard_id, status);
        
        Ok(())
    }
    
    pub async fn start_migration(&self, shard_id: Uuid, target_node: &str) -> Result<Uuid> {
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
        self.migrations.write().await.insert(migration_id, task.clone());
        
        // Update shard status
        self.update_shard_status(shard_id, ShardStatus::Draining).await?;
        
        // Start the migration in the background
        let self_clone = Arc::new(self.clone());
        tokio::spawn(async move {
            if let Err(e) = self_clone.execute_migration(migration_id).await {
                error!("Migration {} failed: {}", migration_id, e);
            }
        });
        
        info!("Started migration {} for shard {} to node {}", migration_id, shard_id, target_node);
        
        Ok(migration_id)
    }
    
    async fn execute_migration(&self, migration_id: Uuid) -> Result<()> {
        // Get the migration task
        let task = {
            let migrations = self.migrations.read().await;
            migrations.get(&migration_id)
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
            assignments.entry(task.target_node.clone())
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
        
        migrations.get(&migration_id)
            .map(|task| (task.completed, task.progress))
            .ok_or_else(|| anyhow!("Migration with ID {} not found", migration_id))
    }
}

// Support cloning for the manager to allow sharing between threads
impl Clone for ShardManager {
    fn clone(&self) -> Self {
        // Note: This creates a new instance with the same node_id and metrics
        // but empty collections. The collections are meant to be
        // accessed through the original instance's RwLocks.
        Self {
            metrics: self.metrics.clone(),
            node_id: self.node_id.clone(),
            shards: RwLock::new(HashMap::new()),
            shard_assignments: RwLock::new(HashMap::new()),
            migrations: RwLock::new(HashMap::new()),
            indices: RwLock::new(HashMap::new()),
        }
    }
}