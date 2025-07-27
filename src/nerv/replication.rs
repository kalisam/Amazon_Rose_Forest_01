use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplicationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug)]
struct ReplicationTask {
    id: Uuid,
    source_node: String,
    target_node: String,
    shard_id: Uuid,
    status: ReplicationStatus,
    progress: f32, // 0.0 to 1.0
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct ReplicationManager {
    tasks: RwLock<HashMap<Uuid, ReplicationTask>>,
    node_id: String,
    peers: RwLock<HashSet<String>>,
}

impl ReplicationManager {
    pub fn new(node_id: &str) -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            node_id: node_id.to_string(),
            peers: RwLock::new(HashSet::new()),
        }
    }

    pub async fn add_peer(&self, peer_id: &str) {
        self.peers.write().await.insert(peer_id.to_string());
        info!("Added peer {} to replication manager", peer_id);
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        self.peers.write().await.remove(peer_id);
        info!("Removed peer {} from replication manager", peer_id);
    }

    pub async fn start_replication(
        self: Arc<Self>,
        shard_id: Uuid,
        target_node: &str,
    ) -> Result<Uuid> {
        // Verify target node is in peers
        if !self.peers.read().await.contains(target_node) {
            return Err(anyhow!("Target node {} is not a known peer", target_node));
        }

        let task_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let task = ReplicationTask {
            id: task_id,
            source_node: self.node_id.clone(),
            target_node: target_node.to_string(),
            shard_id,
            status: ReplicationStatus::Pending,
            progress: 0.0,
            created_at: now,
            updated_at: now,
        };

        self.tasks.write().await.insert(task_id, task);

        // Spawn task to handle replication
        let task_id_clone = task_id;
        let self_clone = Arc::clone(&self);

        tokio::spawn(async move {
            if let Err(e) = self_clone.execute_replication(task_id_clone).await {
                error!("Replication task {} failed: {}", task_id_clone, e);
            }
        });

        Ok(task_id)
    }

    async fn execute_replication(&self, task_id: Uuid) -> Result<()> {
        // Update status to in progress
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = ReplicationStatus::InProgress;
                task.updated_at = chrono::Utc::now();
            } else {
                return Err(anyhow!("Task with ID {} not found", task_id));
            }
        }

        // Simulate replication progress
        for progress in (0..=100).step_by(10) {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.progress = progress as f32 / 100.0;
                task.updated_at = chrono::Utc::now();
            }
        }

        // Update status to completed
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = ReplicationStatus::Completed;
                task.progress = 1.0;
                task.updated_at = chrono::Utc::now();

                info!("Replication task {} completed successfully", task_id);
            }
        }

        Ok(())
    }

    pub async fn get_task_status(&self, task_id: Uuid) -> Result<(ReplicationStatus, f32)> {
        let tasks = self.tasks.read().await;

        if let Some(task) = tasks.get(&task_id) {
            Ok((task.status.clone(), task.progress))
        } else {
            Err(anyhow!("Task with ID {} not found", task_id))
        }
    }

    pub async fn cancel_replication(&self, task_id: Uuid) -> Result<()> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            if task.status == ReplicationStatus::Completed
                || task.status == ReplicationStatus::Failed
            {
                return Err(anyhow!("Cannot cancel task with status {:?}", task.status));
            }

            task.status = ReplicationStatus::Failed;
            task.updated_at = chrono::Utc::now();

            warn!("Replication task {} cancelled", task_id);
            Ok(())
        } else {
            Err(anyhow!("Task with ID {} not found", task_id))
        }
    }
}

// Support cloning for the manager to allow sharing between threads
impl Clone for ReplicationManager {
    fn clone(&self) -> Self {
        // Note: This creates a new instance with the same node_id
        // but empty tasks and peers. The tasks and peers are meant to be
        // accessed through the original instance's RwLocks.
        Self {
            tasks: RwLock::new(HashMap::new()),
            node_id: self.node_id.clone(),
            peers: RwLock::new(HashSet::new()),
        }
    }
}
