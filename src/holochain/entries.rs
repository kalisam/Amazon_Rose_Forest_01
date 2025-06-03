//! Entry definitions for Holochain integration

use hdk::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::holochain::DnaProperties;

/// Entry definition for knowledge contributions
#[hdk_entry(id = "knowledge_contribution")]
#[derive(Clone)]
pub struct KnowledgeContribution {
    pub content_hash: String,
    pub embedding: Vec<f32>,
    pub metadata: Metadata,
    pub timestamp: u64,
}

/// Metadata for knowledge contributions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub tags: Vec<String>,
    pub lesson_learned: String,
}

/// Entry definition for conflict resolution
#[hdk_entry(id = "conflict_resolution")]
#[derive(Clone)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub resolution_type: ResolutionType,
    pub participants: Vec<AgentPubKey>,
    pub justification: String,
    pub created_at: u64,
}

/// Types of conflict resolution
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResolutionType {
    Accept,
    Reject,
    Modify(String),
    Merge(String),
}

/// Entry definition for community arbitration
#[hdk_entry(id = "arbitration_case")]
#[derive(Clone)]
pub struct ArbitrationCase {
    pub id: String,
    pub content_hash: String,
    pub reporter: AgentPubKey,
    pub status: ArbitrationStatus,
    pub votes: Vec<ArbitrationVote>,
    pub resolution: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Status of an arbitration case
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ArbitrationStatus {
    Open,
    UnderReview,
    Resolved,
    Rejected,
}

/// Vote in an arbitration case
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArbitrationVote {
    pub voter: AgentPubKey,
    pub vote: ArbitrationState,
    pub weight: f32,
    pub justification: String,
    pub timestamp: u64,
}

/// Trinary arbitration states
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ArbitrationState {
    Resolve,  // +1: Conflict resolved through understanding
    Review,   //  0: Needs community input
    Reject,   // -1: Harmful content requiring intervention
}

/// Create entry validation
#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<(), KnowledgeContribution>()? {
        FlatOp::StoreEntry(store_entry) => {
            match store_entry {
                OpEntry::CreateEntry { app_entry, action } => {
                    // Validate knowledge contributions
                    match app_entry {
                        AppEntryType::KnowledgeContribution(contribution) => {
                            // Validate embedding dimensions
                            let props = crate::holochain::dna::get_dna_properties()?;
                            if contribution.embedding.len() != props.dimensions {
                                return Ok(ValidateCallbackResult::Invalid(
                                    "Embedding dimensions don't match DNA properties".to_string(),
                                ));
                            }
                            
                            // More validation rules can be added here
                            Ok(ValidateCallbackResult::Valid)
                        },
                        AppEntryType::ArbitrationCase(case) => {
                            // Validate arbitration case
                            if case.status != ArbitrationStatus::Open && case.resolution.is_none() {
                                return Ok(ValidateCallbackResult::Invalid(
                                    "Closed cases must have a resolution".to_string(),
                                ));
                            }
                            
                            Ok(ValidateCallbackResult::Valid)
                        },
                        _ => Ok(ValidateCallbackResult::Valid),
                    }
                },
                _ => Ok(ValidateCallbackResult::Valid),
            }
        },
        _ => Ok(ValidateCallbackResult::Valid),
    }
}