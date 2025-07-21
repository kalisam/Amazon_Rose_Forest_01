//! DNA configuration and management for Holochain integration
//!
//! **Note:** Operations that require a running Holochain conductor (DNA
//! creation, registration and cell installation) are not yet implemented. When
//! compiled with the `holochain_conductor` feature these functions will return
//! a descriptive `Err` indicating the missing integration.

use crate::holochain::DnaProperties;
use crate::sharding::vector_index::DistanceMetric;
use hdk::prelude::*;

/// Zome configuration for vector operations
#[derive(Serialize, Deserialize, Debug)]
pub struct VectorZomeConfig {
    pub dimensions: usize,
    pub distance_metric: String,
    pub max_vector_size: usize,
    pub default_search_limit: usize,
}

/// Get the DNA properties
pub fn get_dna_properties() -> ExternResult<DnaProperties> {
    let dna_info = dna_info()?;
    let props: DnaProperties = dna_info
        .properties
        .try_into()
        .map_err(|e| wasm_error!(WasmErrorInner::Guest(e.to_string())))?;

    Ok(props)
}

/// Get the distance metric from DNA properties
pub fn get_distance_metric() -> ExternResult<DistanceMetric> {
    let props = get_dna_properties()?;

    match props.distance_metric.to_lowercase().as_str() {
        "euclidean" => Ok(DistanceMetric::Euclidean),
        "cosine" => Ok(DistanceMetric::Cosine),
        "manhattan" => Ok(DistanceMetric::Manhattan),
        "hamming" => Ok(DistanceMetric::Hamming),
        _ => Err(wasm_error!(WasmErrorInner::Guest(format!(
            "Unknown distance metric: {}",
            props.distance_metric
        )))),
    }
}

/// Create a new DNA template for a vector index
pub fn create_vector_index_dna(
    _name: String,
    _dimensions: usize,
    _distance_metric: DistanceMetric,
) -> ExternResult<DnaFile> {
    panic!("This function should be provided by the Holochain conductor");
}

/// Register DNA with the conductor
pub fn register_dna(_dna: DnaFile) -> ExternResult<DnaHash> {
    panic!("This function should be provided by the Holochain conductor");
}

/// Create a cell from a DNA and install it
pub fn create_and_install_cell(_dna_hash: DnaHash) -> ExternResult<AgentPubKey> {
    panic!("This function should be provided by the Holochain conductor");
}
