use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedObject<T: Clone> {
    pub id: Uuid,
    pub version: u64,
    pub data: T,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct VersionManager<T: Clone> {
    objects: RwLock<HashMap<Uuid, Vec<VersionedObject<T>>>>,
}

impl<T: Clone + Send + Sync + 'static> VersionManager<T> {
    pub fn new() -> Self {
        Self {
            objects: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_object(&self, data: T) -> VersionedObject<T> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let object = VersionedObject {
            id,
            version: 1,
            data,
            created_at: now,
            updated_at: now,
        };

        let mut objects = self.objects.write().await;
        objects.insert(id, vec![object.clone()]);

        info!("Created new versioned object with ID: {}", id);
        object
    }

    pub async fn update_object(&self, id: Uuid, data: T) -> Result<VersionedObject<T>> {
        let mut objects = self.objects.write().await;

        let versions = objects
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Object with ID {} not found", id))?;

        let latest = versions
            .last()
            .ok_or_else(|| anyhow!("No versions found for object {}", id))?;

        let new_version = VersionedObject {
            id,
            version: latest.version + 1,
            data,
            created_at: latest.created_at,
            updated_at: chrono::Utc::now(),
        };

        versions.push(new_version.clone());

        info!("Updated object {} to version {}", id, new_version.version);
        Ok(new_version)
    }

    pub async fn get_latest(&self, id: Uuid) -> Result<VersionedObject<T>> {
        let objects = self.objects.read().await;

        let versions = objects
            .get(&id)
            .ok_or_else(|| anyhow!("Object with ID {} not found", id))?;

        let latest = versions
            .last()
            .ok_or_else(|| anyhow!("No versions found for object {}", id))?;

        Ok(latest.clone())
    }

    pub async fn get_version(&self, id: Uuid, version: u64) -> Result<VersionedObject<T>> {
        let objects = self.objects.read().await;

        let versions = objects
            .get(&id)
            .ok_or_else(|| anyhow!("Object with ID {} not found", id))?;

        let requested_version = versions
            .iter()
            .find(|obj| obj.version == version)
            .ok_or_else(|| anyhow!("Version {} not found for object {}", version, id))?;

        Ok(requested_version.clone())
    }

    pub async fn get_history(&self, id: Uuid) -> Result<Vec<VersionedObject<T>>> {
        let objects = self.objects.read().await;

        let versions = objects
            .get(&id)
            .ok_or_else(|| anyhow!("Object with ID {} not found", id))?;

        Ok(versions.clone())
    }

    pub async fn delete_object(&self, id: Uuid) -> Result<()> {
        let mut objects = self.objects.write().await;

        if !objects.contains_key(&id) {
            return Err(anyhow!("Object with ID {} not found", id));
        }

        objects.remove(&id);
        info!("Deleted object with ID: {}", id);

        Ok(())
    }
}
