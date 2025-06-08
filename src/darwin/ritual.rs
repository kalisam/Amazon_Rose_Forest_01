use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};

use crate::core::metrics::MetricsCollector;
use crate::darwin::self_improvement::Modification;

/// Ritual represents a structured learning cycle for the Darwin Gödel Machine
#[derive(Debug, Clone)]
pub struct Ritual {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub stages: Vec<RitualStage>,
    pub metrics: HashMap<String, f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// A stage in a ritual learning cycle
#[derive(Debug, Clone)]
pub struct RitualStage {
    pub name: String,
    pub description: String,
    pub status: RitualStageStatus,
    pub depends_on: Vec<String>,
    pub artifacts: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Status of a ritual stage
#[derive(Debug, Clone, PartialEq)]
pub enum RitualStageStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Manager for ritual-based learning cycles
#[derive(Debug)]
pub struct RitualManager {
    metrics: Arc<MetricsCollector>,
    rituals: RwLock<HashMap<Uuid, Ritual>>,
    active_rituals: RwLock<HashSet<Uuid>>,
}

impl RitualManager {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            rituals: RwLock::new(HashMap::new()),
            active_rituals: RwLock::new(HashSet::new()),
        }
    }
    
    /// Create a new ritual learning cycle
    pub async fn create_ritual(&self, name: &str, description: &str, stages: Vec<RitualStage>) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let ritual = Ritual {
            id,
            name: name.to_string(),
            description: description.to_string(),
            stages,
            metrics: HashMap::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
        };
        
        // Store the ritual
        self.rituals.write().await.insert(id, ritual);
        self.active_rituals.write().await.insert(id);
        
        // Update metrics
        self.metrics.increment_counter("darwin.rituals.created", 1).await;
        
        info!("Created new ritual '{}' with ID: {}", name, id);
        
        Ok(id)
    }
    
    /// Get a ritual by ID
    pub async fn get_ritual(&self, id: Uuid) -> Result<Ritual> {
        let rituals = self.rituals.read().await;
        
        rituals.get(&id)
            .cloned()
            .ok_or_else(|| anyhow!("Ritual with ID {} not found", id))
    }
    
    /// Start a ritual stage
    pub async fn start_stage(&self, ritual_id: Uuid, stage_name: &str) -> Result<()> {
        let mut rituals = self.rituals.write().await;
        
        let ritual = rituals.get_mut(&ritual_id)
            .ok_or_else(|| anyhow!("Ritual with ID {} not found", ritual_id))?;
            
        // Find the stage position first to avoid simultaneous mutable and immutable borrows
        let stage_pos = ritual
            .stages
            .iter()
            .position(|s| s.name == stage_name)
            .ok_or_else(|| anyhow!("Stage {} not found in ritual {}", stage_name, ritual_id))?;

        {
            // Check dependencies using an immutable borrow
            let stage_ref = &ritual.stages[stage_pos];
            for dep_name in &stage_ref.depends_on {
                let dep_stage = ritual
                    .stages
                    .iter()
                    .find(|s| &s.name == dep_name)
                    .ok_or_else(|| anyhow!("Dependency stage {} not found", dep_name))?;

                if dep_stage.status != RitualStageStatus::Completed {
                    return Err(anyhow!("Dependency stage {} is not completed", dep_name));
                }
            }
        }

        // Now mutate the stage
        let stage = &mut ritual.stages[stage_pos];
        stage.status = RitualStageStatus::InProgress;
        stage.started_at = Some(Utc::now());
        ritual.updated_at = Utc::now();
        
        info!("Started stage '{}' of ritual '{}'", stage_name, ritual.name);
        
        Ok(())
    }
    
    /// Complete a ritual stage
    pub async fn complete_stage(&self, ritual_id: Uuid, stage_name: &str, artifacts: Vec<String>) -> Result<()> {
        let mut rituals = self.rituals.write().await;
        
        let ritual = rituals.get_mut(&ritual_id)
            .ok_or_else(|| anyhow!("Ritual with ID {} not found", ritual_id))?;
            
        // Find the stage
        let stage = ritual.stages.iter_mut()
            .find(|s| s.name == stage_name)
            .ok_or_else(|| anyhow!("Stage {} not found in ritual {}", stage_name, ritual_id))?;
            
        // Update stage status
        stage.status = RitualStageStatus::Completed;
        stage.completed_at = Some(Utc::now());
        stage.artifacts = artifacts;
        ritual.updated_at = Utc::now();
        
        // Check if all stages are completed
        let all_completed = ritual.stages.iter().all(|s| s.status == RitualStageStatus::Completed);
        
        if all_completed {
            ritual.completed_at = Some(Utc::now());
            self.active_rituals.write().await.remove(&ritual_id);
            info!("Completed ritual '{}'", ritual.name);
            
            // Update metrics
            self.metrics.increment_counter("darwin.rituals.completed", 1).await;
        }
        
        info!("Completed stage '{}' of ritual '{}'", stage_name, ritual.name);
        
        Ok(())
    }
    
    /// Link a modification to a ritual
    pub async fn link_modification(&self, ritual_id: Uuid, modification: &Modification) -> Result<()> {
        let mut rituals = self.rituals.write().await;
        
        let ritual = rituals.get_mut(&ritual_id)
            .ok_or_else(|| anyhow!("Ritual with ID {} not found", ritual_id))?;
            
        // Add the modification ID as an artifact to the current active stage
        if let Some(stage) = ritual.stages.iter_mut().find(|s| s.status == RitualStageStatus::InProgress) {
            stage.artifacts.push(modification.id.to_string());
            info!("Linked modification {} to stage '{}' of ritual '{}'", 
                  modification.id, stage.name, ritual.name);
        }
        
        Ok(())
    }
    
    /// Get active rituals
    pub async fn get_active_rituals(&self) -> Result<Vec<Ritual>> {
        let rituals = self.rituals.read().await;
        let active_ids = self.active_rituals.read().await;
        
        let active_rituals = active_ids.iter()
            .filter_map(|id| rituals.get(id))
            .cloned()
            .collect();
            
        Ok(active_rituals)
    }
    
    /// Add metrics to a ritual
    pub async fn add_ritual_metrics(&self, ritual_id: Uuid, metrics: HashMap<String, f32>) -> Result<()> {
        let mut rituals = self.rituals.write().await;
        
        let ritual = rituals.get_mut(&ritual_id)
            .ok_or_else(|| anyhow!("Ritual with ID {} not found", ritual_id))?;
            
        // Merge metrics
        for (key, value) in metrics {
            ritual.metrics.insert(key, value);
        }
        
        ritual.updated_at = Utc::now();
        
        Ok(())
    }
}

// Support cloning for the manager to allow sharing between threads
impl Clone for RitualManager {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            rituals: RwLock::new(HashMap::new()),
            active_rituals: RwLock::new(HashSet::new()),
        }
    }
}