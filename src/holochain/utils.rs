//! Utility functions for Holochain integration

use hdk::prelude::*;
use crate::core::vector::Vector;
use crate::holochain::VectorEntry;
use uuid::Uuid;
use std::collections::HashMap;

/// Convert a Vector to a Holochain VectorEntry
pub fn vector_to_entry(vector: Vector, metadata: Option<HashMap<String, String>>) -> VectorEntry {
    VectorEntry {
        id: Uuid::new_v4().to_string(),
        values: vector.values.clone(),
        dimensions: vector.dimensions,
        metadata,
        created_at: sys_time().expect("Failed to get system time"),
    }
}

/// Convert a Holochain VectorEntry to a Vector
pub fn entry_to_vector(entry: VectorEntry) -> Vector {
    Vector::new(entry.values)
}

/// Get the current system time
pub fn sys_time() -> ExternResult<u64> {
    let time = sys_time_precise()?;
    Ok(time.as_micros() as u64)
}

/// Check if an agent has a specific capability
pub fn agent_has_capability(agent: &AgentPubKey, capability: &str) -> ExternResult<bool> {
    // This is a simplified implementation
    // In a real-world scenario, this would check against a capability grant
    
    // Get all capability grants
    let path = Path::from("capabilities");
    let links = get_links(path.path_entry_hash()?, Some(LinkTag::new(capability.as_bytes())))?;
    
    // Check if the agent is in the list
    Ok(links.into_iter().any(|link| {
        let target_agent = AgentPubKey::from_raw_39(link.tag.0.clone())
            .map(|key| &key == agent)
            .unwrap_or(false);
        target_agent
    }))
}

/// Create a path for indexing
pub fn create_path(base: &str, components: Vec<&str>) -> ExternResult<Path> {
    let mut path = Path::from(base);
    for component in components {
        path = path.append_component(component)?;
    }
    path.ensure()?;
    Ok(path)
}

/// Create a timestamp-based link tag
pub fn timestamp_tag() -> LinkTag {
    let now = sys_time().unwrap_or(0);
    LinkTag::new(now.to_be_bytes().to_vec())
}

/// Generate a deterministic hash from content
pub fn hash_content(content: &str) -> String {
    // This is a simplified implementation
    // In a real-world scenario, this would use a proper cryptographic hash function
    let mut hash = String::new();
    for byte in content.bytes() {
        hash.push_str(&format!("{:02x}", byte));
    }
    hash
}

/// Generate an embedding from text
pub fn generate_embedding(text: &str) -> Vec<f32> {
    // This is a stub implementation
    // In a real-world scenario, this would use a proper embedding model
    let dimensions = 128;
    let mut rng = rand::thread_rng();
    (0..dimensions).map(|_| rand::random::<f32>()).collect()
}