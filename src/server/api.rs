use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::core::vector::Vector;
use crate::sharding::vector_index::DistanceMetric;

// API request and response types

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShardRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShardResponse {
    pub shard_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIndexRequest {
    pub shard_id: Uuid,
    pub name: String,
    pub dimensions: usize,
    pub distance_metric: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIndexResponse {
    pub shard_id: Uuid,
    pub index_name: String,
    pub dimensions: usize,
    pub distance_metric: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddVectorRequest {
    pub shard_id: Uuid,
    pub vector: Vec<f32>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddVectorResponse {
    pub vector_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchVectorsRequest {
    pub shard_id: Uuid,
    pub query_vector: Vec<f32>,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchVectorsResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// Helper functions for converting between API and internal types

/// Convert a string distance metric to the internal enum
pub fn parse_distance_metric(metric: &str) -> Result<DistanceMetric, String> {
    match metric.to_lowercase().as_str() {
        "euclidean" => Ok(DistanceMetric::Euclidean),
        "cosine" => Ok(DistanceMetric::Cosine),
        "manhattan" => Ok(DistanceMetric::Manhattan),
        "hamming" => Ok(DistanceMetric::Hamming),
        _ => Err(format!("Unknown distance metric: {}", metric)),
    }
}

/// Convert a distance metric enum to a string
pub fn distance_metric_to_string(metric: DistanceMetric) -> String {
    match metric {
        DistanceMetric::Euclidean => "euclidean".to_string(),
        DistanceMetric::Cosine => "cosine".to_string(),
        DistanceMetric::Manhattan => "manhattan".to_string(),
        DistanceMetric::Hamming => "hamming".to_string(),
    }
}

/// Create a Vector from a vec of f32
pub fn create_vector(values: Vec<f32>) -> Vector {
    Vector::new(values)
}

/// Convert internal search results to API search results
pub fn convert_search_results(
    results: Vec<crate::sharding::vector_index::SearchResult>
) -> Vec<SearchResult> {
    results.into_iter()
        .map(|result| SearchResult {
            id: result.id.to_string(),
            score: result.score,
            metadata: result.metadata,
        })
        .collect()
}