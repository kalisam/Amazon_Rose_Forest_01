use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::centroid::Centroid;
use crate::core::vector::Vector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentroidOperation {
    pub id: Uuid,
    pub centroid_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation_type: OperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Create(Vector),
    Update(Vector),
    Delete,
}

#[derive(Debug, Clone)]
pub struct CentroidCRDT {
    node_id: Uuid,
    centroids: HashMap<Uuid, Centroid>,
    operations: HashMap<Uuid, CentroidOperation>,
    observed: HashSet<Uuid>,
}

impl CentroidCRDT {
    pub fn new(node_id: Uuid) -> Self {
        Self {
            node_id,
            centroids: HashMap::new(),
            operations: HashMap::new(),
            observed: HashSet::new(),
        }
    }
    
    pub fn create_centroid(&mut self, vector: Vector) -> Uuid {
        let centroid = Centroid::new(vector.clone());
        let centroid_id = centroid.id;
        
        let operation = CentroidOperation {
            id: Uuid::new_v4(),
            centroid_id,
            timestamp: chrono::Utc::now(),
            operation_type: OperationType::Create(vector),
        };
        
        self.apply_operation(operation.clone());
        
        centroid_id
    }
    
    pub fn update_centroid(&mut self, centroid_id: Uuid, vector: Vector) -> Result<(), String> {
        if !self.centroids.contains_key(&centroid_id) {
            return Err(format!("Centroid with ID {} not found", centroid_id));
        }
        
        let operation = CentroidOperation {
            id: Uuid::new_v4(),
            centroid_id,
            timestamp: chrono::Utc::now(),
            operation_type: OperationType::Update(vector),
        };
        
        self.apply_operation(operation.clone());
        
        Ok(())
    }
    
    pub fn delete_centroid(&mut self, centroid_id: Uuid) -> Result<(), String> {
        if !self.centroids.contains_key(&centroid_id) {
            return Err(format!("Centroid with ID {} not found", centroid_id));
        }
        
        let operation = CentroidOperation {
            id: Uuid::new_v4(),
            centroid_id,
            timestamp: chrono::Utc::now(),
            operation_type: OperationType::Delete,
        };
        
        self.apply_operation(operation.clone());
        
        Ok(())
    }
    
    pub fn apply_operation(&mut self, operation: CentroidOperation) {
        if self.observed.contains(&operation.id) {
            return; // Already observed this operation
        }
        
        match &operation.operation_type {
            OperationType::Create(vector) => {
                // Only create if it doesn't exist or if this is newer than the existing centroid
                let should_create = if let Some(existing) = self.centroids.get(&operation.centroid_id) {
                    operation.timestamp > existing.updated_at
                } else {
                    true
                };
                
                if should_create {
                    let now = chrono::Utc::now();
                    let centroid = Centroid {
                        id: operation.centroid_id,
                        vector: vector.clone(),
                        count: 1,
                        created_at: operation.timestamp,
                        updated_at: operation.timestamp,
                    };
                    self.centroids.insert(operation.centroid_id, centroid);
                }
            },
            OperationType::Update(vector) => {
                if let Some(centroid) = self.centroids.get_mut(&operation.centroid_id) {
                    if operation.timestamp > centroid.updated_at {
                        centroid.update(vector);
                        centroid.updated_at = operation.timestamp;
                    }
                }
            },
            OperationType::Delete => {
                if let Some(centroid) = self.centroids.get(&operation.centroid_id) {
                    if operation.timestamp > centroid.updated_at {
                        self.centroids.remove(&operation.centroid_id);
                    }
                }
            },
        }
        
        self.operations.insert(operation.id, operation);
        self.observed.insert(operation.id);
    }
    
    pub fn merge(&mut self, other: &CentroidCRDT) {
        for (op_id, operation) in &other.operations {
            if !self.observed.contains(op_id) {
                self.apply_operation(operation.clone());
            }
        }
    }
    
    pub fn get_centroid(&self, id: &Uuid) -> Option<&Centroid> {
        self.centroids.get(id)
    }
    
    pub fn get_centroids(&self) -> Vec<&Centroid> {
        self.centroids.values().collect()
    }
    
    pub fn find_nearest(&self, vector: &Vector, limit: usize) -> Vec<(&Centroid, f32)> {
        let mut distances: Vec<(&Centroid, f32)> = self.centroids
            .values()
            .map(|c| (c, c.distance_to(vector)))
            .collect();
            
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        distances.truncate(limit);
        distances
    }
}