use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::vector::Vector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Centroid {
    pub id: Uuid,
    pub vector: Vector,
    pub count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Centroid {
    pub fn new(vector: Vector) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            vector,
            count: 1,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn update(&mut self, vector: &Vector) {
        // Update the centroid by moving it toward the new vector
        let weight_existing = self.count as f32;
        let weight_new = 1.0;
        let total_weight = weight_existing + weight_new;
        
        let weighted_existing = self.vector.clone() * weight_existing;
        let weighted_new = vector.clone() * weight_new;
        
        let combined = weighted_existing + weighted_new;
        let new_vector = combined / total_weight;
        
        self.vector = new_vector;
        self.count += 1;
        self.updated_at = chrono::Utc::now();
    }
    
    pub fn merge(&mut self, other: &Centroid) {
        let total_count = self.count + other.count;
        let weight_self = self.count as f32 / total_count as f32;
        let weight_other = other.count as f32 / total_count as f32;
        
        let weighted_self = self.vector.clone() * weight_self;
        let weighted_other = other.vector.clone() * weight_other;
        
        self.vector = weighted_self + weighted_other;
        self.count = total_count;
        self.updated_at = chrono::Utc::now();
    }
    
    pub fn distance_to(&self, vector: &Vector) -> f32 {
        self.vector.euclidean_distance(vector)
    }
    
    pub fn similarity_to(&self, vector: &Vector) -> f32 {
        self.vector.cosine_similarity(vector)
    }
}