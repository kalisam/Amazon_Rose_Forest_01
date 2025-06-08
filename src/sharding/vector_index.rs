use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{debug, info, warn};

use crate::core::vector::Vector;
use crate::sharding::hilbert::HilbertCurve;
use crate::core::metrics::MetricsCollector;

/// Vector index entry that maps a vector to its ID and metadata
#[derive(Debug, Clone)]
pub struct VectorEntry {
    /// Unique ID for this vector
    pub id: Uuid,
    
    /// The vector data
    pub vector: Vector,
    
    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
    
    /// When this vector was added to the index
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Search results returned from the index
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// ID of the found vector
    pub id: Uuid,
    
    /// The vector itself
    pub vector: Vector,
    
    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
    
    /// Similarity or distance score
    pub score: f32,
}

/// Type of distance metric to use for search
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceMetric {
    Euclidean,
    Cosine,
    Manhattan,
    Hamming,
}

impl DistanceMetric {
    /// Calculate distance between two vectors using the specified metric
    pub fn calculate(&self, a: &Vector, b: &Vector) -> f32 {
        match self {
            Self::Euclidean => a.euclidean_distance(b),
            Self::Cosine => 1.0 - a.cosine_similarity(b), // Convert similarity to distance
            Self::Manhattan => a.manhattan_distance(b),
            Self::Hamming => a.hamming_distance(b) as f32,
        }
    }
    
    /// Check if lower scores are better (true for distances, false for similarities)
    pub fn is_lower_better(&self) -> bool {
        match self {
            Self::Euclidean | Self::Manhattan | Self::Hamming => true,
            Self::Cosine => true, // Since we convert similarity to distance
        }
    }
}

/// Hilbert curve-based vector index for efficient similarity search
#[derive(Debug)]
pub struct VectorIndex {
    /// Name of the index
    name: String,
    
    /// Mapping of vector IDs to vector entries
    vectors: RwLock<HashMap<Uuid, VectorEntry>>,
    
    /// Hilbert curve used for mapping vectors to 1D space
    hilbert_curve: HilbertCurve,
    
    /// Mapping of Hilbert indices to vector IDs
    hilbert_map: RwLock<HashMap<u64, Vec<Uuid>>>,
    
    /// Dimensions of vectors in this index
    dimensions: usize,
    
    /// Distance metric used for similarity search
    distance_metric: DistanceMetric,
    
    /// Metrics collector
    metrics: Option<Arc<MetricsCollector>>,
}

impl VectorIndex {
    /// Create a new vector index
    pub fn new(
        name: &str, 
        dimensions: usize, 
        distance_metric: DistanceMetric,
        metrics: Option<Arc<MetricsCollector>>,
    ) -> Self {
        // Determine bits per dimension based on dimensions
        // We want to keep the total bits under 64 (for u64 hilbert index)
        let max_total_bits = 60; // Leave some room for safety
        let bits_per_dimension = std::cmp::min(
            10, // Maximum reasonable value
            max_total_bits / dimensions
        );
        
        let hilbert_curve = HilbertCurve::new(dimensions, bits_per_dimension);
        
        Self {
            name: name.to_string(),
            vectors: RwLock::new(HashMap::new()),
            hilbert_curve,
            hilbert_map: RwLock::new(HashMap::new()),
            dimensions,
            distance_metric,
            metrics,
        }
    }
    
    /// Convert a vector to a Hilbert index
    fn vector_to_hilbert_index(&self, vector: &Vector) -> u64 {
        // Normalize the vector components to fit within our bit range
        let max_value = (1 << self.hilbert_curve.bits_per_dimension) - 1;
        let point: Vec<u64> = vector.values.iter()
            .map(|&v| {
                // Map from [-1.0, 1.0] to [0, max_value]
                // First clamp the value to ensure it's in range
                let normalized = v.max(-1.0).min(1.0);
                let scaled = ((normalized + 1.0) / 2.0) * (max_value as f32);
                scaled.round() as u64
            })
            .collect();
            
        self.hilbert_curve.point_to_index(&point)
    }
    
    /// Add a vector to the index
    pub async fn add(&self, vector: Vector, metadata: Option<HashMap<String, String>>) -> Result<Uuid, String> {
        // Validate dimensions
        if vector.dimensions != self.dimensions {
            return Err(format!(
                "Vector dimensions mismatch: expected {}, got {}",
                self.dimensions, vector.dimensions
            ));
        }
        
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        let entry = VectorEntry {
            id,
            vector: vector.clone(),
            metadata,
            created_at: now,
        };
        
        // Calculate Hilbert index
        let hilbert_index = self.vector_to_hilbert_index(&vector);
        
        // Add to vectors map
        {
            let mut vectors = self.vectors.write().await;
            vectors.insert(id, entry);
        }
        
        // Add to Hilbert map
        {
            let mut hilbert_map = self.hilbert_map.write().await;
            hilbert_map
                .entry(hilbert_index)
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        // Update metrics
        if let Some(metrics) = &self.metrics {
            metrics.increment_counter(&format!("vector_index.{}.vectors_added", self.name), 1).await;
            metrics.set_gauge(&format!("vector_index.{}.vector_count", self.name), self.count().await as u64).await;
        }
        
        debug!("Added vector to index '{}' with ID: {}", self.name, id);
        
        Ok(id)
    }
    
    /// Remove a vector from the index
    pub async fn remove(&self, id: Uuid) -> Result<(), String> {
        // Get the vector to calculate its Hilbert index
        let vector = {
            let vectors = self.vectors.read().await;
            match vectors.get(&id) {
                Some(entry) => entry.vector.clone(),
                None => return Err(format!("Vector with ID {} not found", id)),
            }
        };
        
        let hilbert_index = self.vector_to_hilbert_index(&vector);
        
        // Remove from vectors map
        {
            let mut vectors = self.vectors.write().await;
            vectors.remove(&id);
        }
        
        // Remove from Hilbert map
        {
            let mut hilbert_map = self.hilbert_map.write().await;
            if let Some(ids) = hilbert_map.get_mut(&hilbert_index) {
                ids.retain(|&x| x != id);
                
                // Remove the entire entry if there are no more vectors at this index
                if ids.is_empty() {
                    hilbert_map.remove(&hilbert_index);
                }
            }
        }
        
        // Update metrics
        if let Some(metrics) = &self.metrics {
            metrics.increment_counter(&format!("vector_index.{}.vectors_removed", self.name), 1).await;
            metrics.set_gauge(&format!("vector_index.{}.vector_count", self.name), self.count().await as u64).await;
        }
        
        debug!("Removed vector from index '{}' with ID: {}", self.name, id);
        
        Ok(())
    }
    
    /// Find nearest vectors using the index
    pub async fn search(&self, query: &Vector, limit: usize) -> Result<Vec<SearchResult>, String> {
        let start = std::time::Instant::now();
        
        // Validate dimensions
        if query.dimensions != self.dimensions {
            return Err(format!(
                "Query vector dimensions mismatch: expected {}, got {}",
                self.dimensions, query.dimensions
            ));
        }
        
        // Calculate Hilbert index of the query
        let query_hilbert_index = self.vector_to_hilbert_index(query);
        
        // Get nearby indices in Hilbert space
        // This is a simplified implementation - a more sophisticated version would
        // explore the Hilbert space more intelligently
        let nearby_indices = self.get_nearby_indices(query_hilbert_index).await;
        
        // Collect candidate vectors
        let mut candidates: Vec<VectorEntry> = Vec::new();
        {
            let vectors = self.vectors.read().await;
            let hilbert_map = self.hilbert_map.read().await;
            
            for &index in &nearby_indices {
                if let Some(ids) = hilbert_map.get(&index) {
                    for &id in ids {
                        if let Some(entry) = vectors.get(&id) {
                            candidates.push(entry.clone());
                        }
                    }
                }
            }
            
            // If we have too few candidates, fall back to linear search
            if candidates.len() < limit * 4 && candidates.len() < vectors.len() / 2 {
                debug!("Falling back to linear search for index '{}'", self.name);
                candidates = vectors.values().cloned().collect();
            }
        }
        
        // Calculate distances
        let mut results: Vec<SearchResult> = candidates
            .iter()
            .map(|entry| {
                let score = self.distance_metric.calculate(query, &entry.vector);
                SearchResult {
                    id: entry.id,
                    vector: entry.vector.clone(),
                    metadata: entry.metadata.clone(),
                    score,
                }
            })
            .collect();
            
        // Sort by score
        results.sort_by(|a, b| {
            if self.distance_metric.is_lower_better() {
                a.score.partial_cmp(&b.score).unwrap()
            } else {
                b.score.partial_cmp(&a.score).unwrap()
            }
        });
        
        // Limit results
        results.truncate(limit);
        
        let elapsed = start.elapsed();
        
        // Update metrics
        if let Some(metrics) = &self.metrics {
            metrics.increment_counter(&format!("vector_index.{}.searches", self.name), 1).await;
            metrics.record_histogram(
                &format!("vector_index.{}.search_time_ms", self.name),
                elapsed.as_millis() as u64
            ).await;
        }
        
        debug!("Search in index '{}' found {} results in {:?}", 
               self.name, results.len(), elapsed);
        
        Ok(results)
    }
    
    /// Get nearby indices in Hilbert space
    async fn get_nearby_indices(&self, center_index: u64) -> Vec<u64> {
        // Start with the exact index
        let mut indices = vec![center_index];
        
        // Add some nearby indices (this is a simple implementation)
        // In a more sophisticated version, we would explore the Hilbert curve more intelligently
        let window_size = 5;
        for i in 1..=window_size {
            // Add indices before
            if center_index >= i {
                indices.push(center_index - i);
            }
            
            // Add indices after
            indices.push(center_index + i);
        }
        
        // Check if these indices exist in our map
        let hilbert_map = self.hilbert_map.read().await;
        indices.retain(|&idx| hilbert_map.contains_key(&idx));
        
        indices
    }
    
    /// Get the number of vectors in the index
    pub async fn count(&self) -> usize {
        self.vectors.read().await.len()
    }
    
    /// Get detailed statistics about the index
    pub async fn stats(&self) -> IndexStats {
        let vectors = self.vectors.read().await;
        let hilbert_map = self.hilbert_map.read().await;
        
        let mut bucket_sizes = Vec::new();
        for (_, ids) in hilbert_map.iter() {
            bucket_sizes.push(ids.len());
        }
        
        bucket_sizes.sort_unstable();
        
        let bucket_count = bucket_sizes.len();
        let total_vectors = vectors.len();
        
        let min_bucket = bucket_sizes.first().cloned().unwrap_or(0);
        let max_bucket = bucket_sizes.last().cloned().unwrap_or(0);
        
        let avg_bucket = if !bucket_sizes.is_empty() {
            bucket_sizes.iter().sum::<usize>() as f32 / bucket_sizes.len() as f32
        } else {
            0.0
        };
        
        let median_bucket = if !bucket_sizes.is_empty() {
            if bucket_sizes.len() % 2 == 0 {
                (bucket_sizes[bucket_sizes.len() / 2 - 1] + bucket_sizes[bucket_sizes.len() / 2]) as f32 / 2.0
            } else {
                bucket_sizes[bucket_sizes.len() / 2] as f32
            }
        } else {
            0.0
        };
        
        IndexStats {
            name: self.name.clone(),
            vector_count: total_vectors,
            dimensions: self.dimensions,
            distance_metric: self.distance_metric,
            bucket_count,
            min_bucket_size: min_bucket,
            max_bucket_size: max_bucket,
            avg_bucket_size: avg_bucket,
            median_bucket_size: median_bucket,
        }
    }
}

/// Statistics about the vector index
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Name of the index
    pub name: String,
    
    /// Number of vectors in the index
    pub vector_count: usize,
    
    /// Dimensions of vectors in this index
    pub dimensions: usize,
    
    /// Distance metric used for similarity search
    pub distance_metric: DistanceMetric,
    
    /// Number of Hilbert space buckets
    pub bucket_count: usize,
    
    /// Minimum bucket size
    pub min_bucket_size: usize,
    
    /// Maximum bucket size
    pub max_bucket_size: usize,
    
    /// Average bucket size
    pub avg_bucket_size: f32,
    
    /// Median bucket size
    pub median_bucket_size: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    
    async fn create_test_index(vector_count: usize, dimensions: usize) -> VectorIndex {
        let index = VectorIndex::new("test_index", dimensions, DistanceMetric::Euclidean, None);
        
        // Add random vectors
        for _ in 0..vector_count {
            let vector = Vector::random(dimensions);
            let mut metadata = HashMap::new();
            metadata.insert("test".to_string(), "value".to_string());
            
            index.add(vector, Some(metadata)).await.unwrap();
        }
        
        index
    }
    
    #[tokio::test]
    async fn test_add_and_count() {
        let index = create_test_index(10, 3).await;
        assert_eq!(index.count().await, 10);
    }
    
    #[tokio::test]
    async fn test_remove() {
        let index = VectorIndex::new("test_remove", 3, DistanceMetric::Euclidean, None);
        
        // Add a vector
        let vector = Vector::random(3);
        let id = index.add(vector.clone(), None).await.unwrap();
        assert_eq!(index.count().await, 1);
        
        // Remove it
        index.remove(id).await.unwrap();
        assert_eq!(index.count().await, 0);
        
        // Try to remove it again (should fail)
        assert!(index.remove(id).await.is_err());
    }
    
    #[tokio::test]
    async fn test_search() {
        let dimensions = 10;
        let vector_count = 100;
        
        let index = create_test_index(vector_count, dimensions).await;
        
        // Create a query vector
        let query = Vector::random(dimensions);
        
        // Search for nearest vectors
        let results = index.search(&query, 5).await.unwrap();
        
        // We should get 5 results
        assert_eq!(results.len(), 5);
        
        // Scores should be sorted
        for i in 1..results.len() {
            assert!(results[i-1].score <= results[i].score);
        }
    }
    
    #[tokio::test]
    async fn test_different_metrics() {
        // Test with different distance metrics
        let dimensions = 8;
        let vector_count = 50;
        
        // Create some vectors with a known pattern
        let mut vectors = Vec::new();
        let mut rng = rand::thread_rng();
        
        for _ in 0..vector_count {
            let values: Vec<f32> = (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect();
            vectors.push(Vector::new(values));
        }
        
        // Create query vector
        let query = Vector::random(dimensions);
        
        // Test each metric
        for metric in [DistanceMetric::Euclidean, DistanceMetric::Cosine, 
                      DistanceMetric::Manhattan, DistanceMetric::Hamming].iter() {
            let index = VectorIndex::new("test_metric", dimensions, *metric, None);
            
            // Add all vectors
            for v in &vectors {
                index.add(v.clone(), None).await.unwrap();
            }
            
            // Search
            let results = index.search(&query, 5).await.unwrap();
            assert_eq!(results.len(), 5);
            
            // Scores should be in ascending order for distance metrics
            if metric.is_lower_better() {
                for i in 1..results.len() {
                    assert!(results[i-1].score <= results[i].score);
                }
            } else {
                // Or descending order for similarity metrics
                for i in 1..results.len() {
                    assert!(results[i-1].score >= results[i].score);
                }
            }
        }
    }
}