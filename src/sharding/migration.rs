use std::sync::Arc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationTask {
    pub id: Uuid,
    pub shard_id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub completed: bool,
    pub progress: f32, // 0.0 to 1.0
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl MigrationTask {
    pub fn new(
        id: Uuid,
        shard_id: Uuid,
        source_node: String,
        target_node: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        Self {
            id,
            shard_id,
            source_node,
            target_node,
            completed: false,
            progress: 0.0,
            created_at: now,
            updated_at: now,
        }
    }
}