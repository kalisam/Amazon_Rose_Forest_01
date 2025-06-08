//! Holochain integration module for Amazon Rose Forest

pub mod dna;
pub mod zome;
pub mod entries;
pub mod utils;
pub mod arbitration;
pub mod transparency;
pub mod hash;

use hdk::prelude::*;
use uuid::Uuid;
use crate::core::vector::Vector;
use crate::core::centroid::Centroid;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Entry definition for a vector in Holochain
#[hdk_entry(id = "vector")]
#[derive(Clone)]
pub struct VectorEntry {
    pub id: String,
    pub values: Vec<f32>,
    pub dimensions: usize,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: u64,
}

impl From<Vector> for VectorEntry {
    fn from(vector: Vector) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            values: vector.values.clone(),
            dimensions: vector.dimensions,
            metadata: None,
            created_at: sys_time()?,
        }
    }
}

impl TryFrom<VectorEntry> for Vector {
    type Error = String;
    
    fn try_from(entry: VectorEntry) -> Result<Self, Self::Error> {
        Ok(Vector::new(entry.values))
    }
}

/// Entry definition for a centroid in Holochain
#[hdk_entry(id = "centroid")]
#[derive(Clone)]
pub struct CentroidEntry {
    pub id: String,
    pub vector: VectorEntry,
    pub count: usize,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<Centroid> for CentroidEntry {
    fn from(centroid: Centroid) -> Self {
        Self {
            id: centroid.id.to_string(),
            vector: VectorEntry::from(centroid.vector),
            count: centroid.count,
            created_at: centroid.created_at.timestamp_millis() as u64,
            updated_at: centroid.updated_at.timestamp_millis() as u64,
        }
    }
}

/// Entry definition for an audit trail in Holochain
#[hdk_entry(id = "audit_trail")]
#[derive(Clone)]
pub struct AuditTrail {
    /// The action being audited
    pub action: String,
    
    /// Who initiated the action
    pub initiator: AgentPubKey,
    
    /// Who participated in validation
    pub validators: Vec<AgentPubKey>,
    
    /// Cryptographic proof of decision process
    #[serde(with = "serde_bytes")]
    pub decision_proof: Vec<u8>,
    
    /// Human-readable justification
    pub justification: String,
    
    /// Timestamp with nanosecond precision
    pub timestamp: u64,
}

/// Get the current system time
fn sys_time() -> ExternResult<u64> {
    let time = sys_time_precise()?;
    Ok(time.as_micros() as u64)
}

/// DNA properties configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct DnaProperties {
    pub name: String,
    pub uuid: String,
    pub distance_metric: String,
    pub dimensions: usize,
    pub similarity_threshold: f32,
}

/// Initialize the DNA with the provided properties
#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // Create necessary indexes
    create_index("vectors_by_id")?;
    create_index("centroids_by_id")?;
    create_index("audit_trails_by_timestamp")?;
    
    // Get DNA properties
    let props: DnaProperties = dna_info()?.properties
        .try_into()
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;
    
    debug!("Initializing Rose Forest DNA: {}", props.name);
    
    Ok(InitCallbackResult::Pass)
}

/// Create a path for indexing entries
fn create_index(name: &str) -> ExternResult<()> {
    let path = Path::from(name);
    path.ensure()?;
    Ok(())
}