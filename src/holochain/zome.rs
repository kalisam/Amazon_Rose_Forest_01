//! Zome functions for Holochain integration

use hdk::prelude::*;
use crate::core::vector::Vector;
use crate::holochain::{VectorEntry, CentroidEntry, AuditTrail, sys_time};
use crate::holochain::dna::get_distance_metric;
use std::collections::HashMap;
use uuid::Uuid;

/// Add a vector to the DHT
#[hdk_extern]
pub fn add_vector(input: VectorInput) -> ExternResult<VectorOutput> {
    let props = crate::holochain::dna::get_dna_properties()?;
    
    // Validate dimensions
    if input.values.len() != props.dimensions {
        return Err(wasm_error!(
            WasmErrorInner::Guest(format!(
                "Vector dimensions mismatch: expected {}, got {}",
                props.dimensions,
                input.values.len()
            ))
        ));
    }
    
    // Create Vector
    let vector = Vector::new(input.values);
    
    // Create VectorEntry
    let id = Uuid::new_v4().to_string();
    let entry = VectorEntry {
        id: id.clone(),
        values: vector.values.clone(),
        dimensions: vector.dimensions,
        metadata: input.metadata.clone(),
        created_at: sys_time()?,
    };
    
    // Create entry in DHT
    let entry_hash = create_entry(&entry)?;
    
    // Add to vector index
    let path = Path::from("vectors_by_id").path_entry_hash()?;
    let link_tag = LinkTag::new(id.as_bytes());
    create_link(path, entry_hash, link_tag)?;
    
    // Create audit trail
    create_audit_trail("add_vector", 
        json!({"vector_id": id, "dimensions": vector.dimensions}).to_string())?;
    
    Ok(VectorOutput {
        id,
        entry_hash: entry_hash.to_string(),
    })
}

/// Search for vectors similar to the query
#[hdk_extern]
pub fn search_vectors(input: SearchInput) -> ExternResult<SearchOutput> {
    let props = crate::holochain::dna::get_dna_properties()?;
    
    // Validate dimensions
    if input.query.len() != props.dimensions {
        return Err(wasm_error!(
            WasmErrorInner::Guest(format!(
                "Query vector dimensions mismatch: expected {}, got {}",
                props.dimensions,
                input.query.len()
            ))
        ));
    }
    
    // Create query vector
    let query = Vector::new(input.query);
    
    // Get distance metric from DNA properties
    let distance_metric = get_distance_metric()?;
    
    // Get all vectors
    let vector_entries = get_all_vectors()?;
    
    // Calculate distances
    let mut results: Vec<SearchResult> = vector_entries
        .into_iter()
        .map(|entry| {
            let vector = Vector::try_from(entry.clone())
                .map_err(|e| wasm_error!(WasmErrorInner::Guest(e)))?;
            
            let score = match distance_metric {
                crate::sharding::vector_index::DistanceMetric::Euclidean => {
                    query.euclidean_distance(&vector)
                },
                crate::sharding::vector_index::DistanceMetric::Cosine => {
                    1.0 - query.cosine_similarity(&vector)
                },
                crate::sharding::vector_index::DistanceMetric::Manhattan => {
                    query.manhattan_distance(&vector)
                },
                crate::sharding::vector_index::DistanceMetric::Hamming => {
                    query.hamming_distance(&vector) as f32
                },
            };
            
            Ok(SearchResult {
                id: entry.id.clone(),
                vector: entry.values.clone(),
                metadata: entry.metadata.clone(),
                score,
            })
        })
        .collect::<ExternResult<Vec<SearchResult>>>()?;
    
    // Sort by score (lower is better)
    results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
    
    // Limit results
    let limit = input.limit.unwrap_or(10).min(100);
    results.truncate(limit);
    
    // Create audit trail
    create_audit_trail("search_vectors", 
        json!({"query_dimensions": query.dimensions, "result_count": results.len()}).to_string())?;
    
    Ok(SearchOutput {
        results,
    })
}

/// Input for vector creation
#[derive(Serialize, Deserialize, Debug)]
pub struct VectorInput {
    pub values: Vec<f32>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Output from vector creation
#[derive(Serialize, Deserialize, Debug)]
pub struct VectorOutput {
    pub id: String,
    pub entry_hash: String,
}

/// Input for vector search
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchInput {
    pub query: Vec<f32>,
    pub limit: Option<usize>,
}

/// Output from vector search
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchOutput {
    pub results: Vec<SearchResult>,
}

/// Search result
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<HashMap<String, String>>,
    pub score: f32,
}

/// Get all vectors from the DHT
fn get_all_vectors() -> ExternResult<Vec<VectorEntry>> {
    let path = Path::from("vectors_by_id");
    let links = get_links(path.path_entry_hash()?, None)?;
    
    let entries = links
        .into_iter()
        .map(|link| {
            let entry_hash = link.target;
            let entry: VectorEntry = get_entry(entry_hash)?
                .ok_or_else(|| wasm_error!(
                    WasmErrorInner::Guest("Vector entry not found".to_string())
                ))?
                .try_into()
                .map_err(|e: SerializedBytesError| wasm_error!(WasmErrorInner::Serialize(e)))?;
                
            Ok(entry)
        })
        .collect::<ExternResult<Vec<VectorEntry>>>()?;
        
    Ok(entries)
}

/// Create an audit trail entry
fn create_audit_trail(action: &str, details: String) -> ExternResult<EntryHash> {
    let audit = AuditTrail {
        action: action.to_string(),
        initiator: agent_info()?.agent_latest_pubkey,
        validators: vec![], // Would be populated during validation
        decision_proof: Vec::new(), // Would be populated with a real merkle proof
        justification: details,
        timestamp: sys_time()?,
    };
    
    let entry_hash = create_entry(&audit)?;
    
    // Add to audit trail index
    let path = Path::from("audit_trails_by_timestamp");
    let link_tag = LinkTag::new(format!("{}", audit.timestamp).as_bytes());
    create_link(path.path_entry_hash()?, entry_hash.clone(), link_tag)?;
    
    Ok(entry_hash)
}
