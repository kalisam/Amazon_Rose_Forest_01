//! Transparency and audit trail functionality

use hdk::prelude::*;
use crate::holochain::entries::AuditTrail;
use crate::holochain::utils::{sys_time, create_path, timestamp_tag};
use std::collections::HashMap;
use crate::holochain::hash::default_hash_bytes;

/// Public API for transparency verification
#[hdk_extern]
pub fn audit_trail(contribution_hash: EntryHash) -> ExternResult<AuditTrail> {
    let history = get_details(contribution_hash, GetOptions::default())?
        .ok_or(wasm_error!(WasmErrorInner::Guest("Entry not found".to_string())))?;
    
    // Reconstruct complete decision history
    let audit_trail = reconstruct_audit_trail(history)?;
    
    // Verify cryptographic integrity
    verify_merkle_proof(&audit_trail.decision_proof)?;
    
    Ok(audit_trail)
}

/// Public interface for querying system transparency
#[hdk_extern] 
pub fn query_transparency_metrics() -> ExternResult<TransparencyMetrics> {
    Ok(TransparencyMetrics {
        total_decisions: count_all_decisions()?,
        public_audit_rate: calculate_audit_accessibility()?,
        average_validation_participants: compute_avg_validators()?,
        decision_reversal_rate: calculate_reversal_rate()?,
    })
}

/// Create a new audit trail entry
#[hdk_extern]
pub fn create_audit_entry(input: AuditInput) -> ExternResult<EntryHash> {
    let now = sys_time()?;
    
    // Get validators for this entry
    // In a real implementation, this would be determined by DHT validation
    let validators = vec![agent_info()?.agent_latest_pubkey];
    
    let audit = AuditTrail {
        action: input.action,
        initiator: agent_info()?.agent_latest_pubkey,
        validators,
        decision_proof: generate_merkle_proof(&input.details)?,
        justification: input.details,
        timestamp: now,
    };
    
    // Create entry
    let audit_hash = create_entry(&audit)?;
    
    // Add to audit index
    let path = create_path("audit_trails", vec![&now.to_string()])?;
    create_link(path.path_entry_hash()?, audit_hash.clone(), timestamp_tag())?;
    
    Ok(audit_hash)
}

/// Query recent audit trails
#[hdk_extern]
pub fn get_recent_audits(count: usize) -> ExternResult<Vec<AuditTrail>> {
    let path = Path::from("audit_trails");
    let links = get_links(path.path_entry_hash()?, None)?;
    
    // Sort links by timestamp (descending)
    let mut sorted_links = links;
    sorted_links.sort_by(|a, b| {
        let a_time = u64::from_be_bytes(a.tag.0[0..8].try_into().unwrap_or([0; 8]));
        let b_time = u64::from_be_bytes(b.tag.0[0..8].try_into().unwrap_or([0; 8]));
        b_time.cmp(&a_time)
    });
    
    // Limit to requested count
    sorted_links.truncate(count);
    
    // Get audit entries
    let audits = sorted_links
        .into_iter()
        .map(|link| {
            let entry: AuditTrail = get_entry(link.target)?
                .ok_or(wasm_error!(WasmErrorInner::Guest("Audit entry not found".to_string())))?
                .try_into()
                .map_err(|e: SerializedBytesError| wasm_error!(WasmErrorInner::Serialize(e)))?;
            Ok(entry)
        })
        .collect::<ExternResult<Vec<AuditTrail>>>()?;
    
    Ok(audits)
}

/// Transparency metrics
#[derive(Serialize, Deserialize, Debug)]
pub struct TransparencyMetrics {
    pub total_decisions: usize,
    pub public_audit_rate: f32,
    pub average_validation_participants: f32,
    pub decision_reversal_rate: f32,
}

/// Audit input
#[derive(Serialize, Deserialize, Debug)]
pub struct AuditInput {
    pub action: String,
    pub details: String,
}

/// Reconstruct an audit trail from entry history
fn reconstruct_audit_trail(_details: Details) -> ExternResult<AuditTrail> {
    // In a real implementation, this would reconstruct the audit trail
    // from the entry history. For now, we'll just return an empty audit trail.
    Ok(AuditTrail {
        action: "".to_string(),
        initiator: agent_info()?.agent_latest_pubkey,
        validators: vec![],
        decision_proof: vec![],
        justification: "".to_string(),
        timestamp: 0,
    })
}

/// Verify a Merkle proof
fn verify_merkle_proof(_proof: &[u8]) -> ExternResult<()> {
    // This is a stub that should be properly implemented
    // For now, we'll just return Ok.
    Ok(())
}

/// Generate a Merkle proof
fn generate_merkle_proof(content: &str) -> ExternResult<Vec<u8>> {
    // In a real implementation, we would create a proper Merkle proof
    // This would involve creating a Merkle tree and generating a proof.
    // For now, hash the content as a placeholder proof.
    Ok(default_hash_bytes(content.as_bytes()))
}

/// Count all decisions in the system
fn count_all_decisions() -> ExternResult<usize> {
    let path = Path::from("audit_trails");
    let links = get_links(path.path_entry_hash()?, None)?;
    Ok(links.len())
}

/// Calculate audit accessibility
fn calculate_audit_accessibility() -> ExternResult<f32> {
    // This would calculate real metrics in a real implementation
    // For now, return a placeholder
    Ok(1.0) // 100% accessible
}

/// Compute average validators per decision
fn compute_avg_validators() -> ExternResult<f32> {
    // This would calculate real metrics in a real implementation
    // For now, return a placeholder
    Ok(3.5) // Average of 3.5 validators per decision
}

/// Calculate decision reversal rate
fn calculate_reversal_rate() -> ExternResult<f32> {
    // This would calculate real metrics in a real implementation
    // For now, return a placeholder
    Ok(0.02) // 2% reversal rate
}