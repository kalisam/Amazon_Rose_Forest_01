use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::core::vector::Vector;
use crate::core::metrics::MetricsCollector;

/// Represents a proposed modification to the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modification {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub code_changes: Vec<CodeChange>,
    pub validation_metrics: HashMap<String, f32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: ModificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModificationStatus {
    Proposed,
    Validating,
    Accepted,
    Rejected,
    Deployed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub original_content: String,
    pub modified_content: String,
    pub diff: String,
}

/// Core self-improvement engine
#[derive(Debug)]
pub struct SelfImprovementEngine {
    /// Metrics collector for performance tracking
    metrics: Arc<MetricsCollector>,
    
    /// History of all proposed modifications
    modifications: RwLock<Vec<Modification>>,
    
    /// Current validation pipeline
    validation_pipeline: Arc<crate::darwin::validation::ValidationPipeline>,
    
    /// Exploration strategy
    exploration_strategy: Arc<crate::darwin::exploration::ExplorationStrategy>,
    
    /// Maximum modifications to keep in history
    max_history_size: usize,
}

impl SelfImprovementEngine {
    pub fn new(
        metrics: Arc<MetricsCollector>,
        validation_pipeline: Arc<crate::darwin::validation::ValidationPipeline>,
        exploration_strategy: Arc<crate::darwin::exploration::ExplorationStrategy>,
    ) -> Self {
        Self {
            metrics,
            modifications: RwLock::new(Vec::new()),
            validation_pipeline,
            exploration_strategy,
            max_history_size: 1000,
        }
    }
    
    /// Propose a new system modification
    pub async fn propose_modification(&self, proposal: Modification) -> Result<Uuid> {
        let id = proposal.id;
        
        // Store the modification
        {
            let mut modifications = self.modifications.write().await;
            modifications.push(proposal.clone());
            
            // Trim history if needed
            if modifications.len() > self.max_history_size {
                modifications.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                modifications.truncate(self.max_history_size);
            }
        }
        
        // Update metrics
        self.metrics.increment_counter("darwin.modifications.proposed", 1).await;
        
        info!("New modification proposed: {} (ID: {})", proposal.name, id);
        
        // Start validation in the background
        let self_clone = Arc::new(self.clone());
        let proposal_id = proposal.id;
        
        tokio::spawn(async move {
            if let Err(e) = self_clone.validate_modification(proposal_id).await {
                error!("Failed to validate modification {}: {}", proposal_id, e);
            }
        });
        
        Ok(id)
    }
    
    /// Validate a proposed modification
    pub async fn validate_modification(&self, modification_id: Uuid) -> Result<bool> {
        // Update status to validating
        self.update_modification_status(modification_id, ModificationStatus::Validating).await?;
        
        // Get the modification
        let modification = self.get_modification(modification_id).await?;
        
        // Run validation
        let validation_result = self.validation_pipeline.validate(&modification).await;
        
        match validation_result {
            Ok(metrics) => {
                // Update modification with validation metrics
                self.update_modification_metrics(modification_id, metrics.clone()).await?;
                
                // Check if validation passed
                let passed = self.validation_pipeline.is_valid(&metrics);
                
                // Update status
                let new_status = if passed {
                    ModificationStatus::Accepted
                } else {
                    ModificationStatus::Rejected
                };
                
                self.update_modification_status(modification_id, new_status).await?;
                
                // Update metrics
                if passed {
                    self.metrics.increment_counter("darwin.modifications.accepted", 1).await;
                } else {
                    self.metrics.increment_counter("darwin.modifications.rejected", 1).await;
                }
                
                info!("Modification {} validation {}", modification_id, if passed { "passed" } else { "failed" });
                
                Ok(passed)
            },
            Err(e) => {
                // Update status to failed
                self.update_modification_status(modification_id, ModificationStatus::Failed).await?;
                
                // Update metrics
                self.metrics.increment_counter("darwin.modifications.failed", 1).await;
                
                error!("Modification {} validation failed: {}", modification_id, e);
                
                Err(anyhow!("Validation failed: {}", e))
            }
        }
    }
    
    /// Deploy an accepted modification
    pub async fn deploy_modification(&self, modification_id: Uuid) -> Result<()> {
        // Get the modification
        let modification = self.get_modification(modification_id).await?;
        
        // Check if it's accepted
        if modification.status != ModificationStatus::Accepted {
            return Err(anyhow!("Cannot deploy modification with status {:?}", modification.status));
        }
        
        // Update status to deploying
        self.update_modification_status(modification_id, ModificationStatus::Deployed).await?;
        
        // Apply the code changes
        // Note: In a real system, this would involve more sophisticated code manipulation
        // and potentially a restart of affected components
        for change in &modification.code_changes {
            info!("Would apply change to file: {}", change.file_path);
            // In a real system: apply_code_change(&change)?;
        }
        
        // Update metrics
        self.metrics.increment_counter("darwin.modifications.deployed", 1).await;
        
        info!("Modification {} deployed successfully", modification_id);
        
        Ok(())
    }
    
    /// Get a specific modification
    pub async fn get_modification(&self, id: Uuid) -> Result<Modification> {
        let modifications = self.modifications.read().await;
        
        modifications.iter()
            .find(|m| m.id == id)
            .cloned()
            .ok_or_else(|| anyhow!("Modification with ID {} not found", id))
    }
    
    /// Get all modifications
    pub async fn get_all_modifications(&self) -> Vec<Modification> {
        let modifications = self.modifications.read().await;
        modifications.clone()
    }
    
    /// Update modification status
    async fn update_modification_status(&self, id: Uuid, status: ModificationStatus) -> Result<()> {
        let mut modifications = self.modifications.write().await;
        
        let modification = modifications.iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| anyhow!("Modification with ID {} not found", id))?;
            
        modification.status = status;
        
        Ok(())
    }
    
    /// Update modification metrics
    async fn update_modification_metrics(&self, id: Uuid, metrics: HashMap<String, f32>) -> Result<()> {
        let mut modifications = self.modifications.write().await;
        
        let modification = modifications.iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| anyhow!("Modification with ID {} not found", id))?;
            
        modification.validation_metrics = metrics;
        
        Ok(())
    }
    
    /// Generate new modifications using exploration strategy
    pub async fn generate_modifications(&self) -> Result<Vec<Uuid>> {
        info!("Generating new modifications using exploration strategy");
        
        // Use exploration strategy to generate modifications
        let proposals = self.exploration_strategy.generate_proposals().await?;
        
        // Submit all proposals
        let mut ids = Vec::new();
        for proposal in proposals {
            let id = self.propose_modification(proposal).await?;
            ids.push(id);
        }
        
        info!("Generated {} new modification proposals", ids.len());
        
        Ok(ids)
    }
}

// Support cloning for the engine to allow sharing between threads
impl Clone for SelfImprovementEngine {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            modifications: RwLock::new(Vec::new()),
            validation_pipeline: self.validation_pipeline.clone(),
            exploration_strategy: self.exploration_strategy.clone(),
            max_history_size: self.max_history_size,
        }
    }
}

// Add missing HashMap import
use std::collections::HashMap;