use crate::core::centroid::Centroid;
use crate::core::vector::Vector;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Debug, Error)]
pub enum CentroidCRDTError {
    #[error("Centroid with ID {0} not found")]
    NotFound(Uuid),

    #[error("Invalid distance value encountered during comparison")]
    InvalidDistance,
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

    pub fn update_centroid(
        &mut self,
        centroid_id: Uuid,
        vector: Vector,
    ) -> Result<(), CentroidCRDTError> {
        if !self.centroids.contains_key(&centroid_id) {
            return Err(CentroidCRDTError::NotFound(centroid_id));
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

    pub fn delete_centroid(&mut self, centroid_id: Uuid) -> Result<(), CentroidCRDTError> {
        if !self.centroids.contains_key(&centroid_id) {
            return Err(CentroidCRDTError::NotFound(centroid_id));
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
                let should_create =
                    if let Some(existing) = self.centroids.get(&operation.centroid_id) {
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
            }
            OperationType::Update(vector) => {
                if let Some(centroid) = self.centroids.get_mut(&operation.centroid_id) {
                    if operation.timestamp > centroid.updated_at {
                        centroid.update(vector);
                        centroid.updated_at = operation.timestamp;
                    }
                }
            }
            OperationType::Delete => {
                if let Some(centroid) = self.centroids.get(&operation.centroid_id) {
                    if operation.timestamp > centroid.updated_at {
                        self.centroids.remove(&operation.centroid_id);
                    }
                }
            }
        }

        let op_id = operation.id;
        self.operations.insert(op_id, operation);
        self.observed.insert(op_id);
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

    pub fn find_nearest(
        &self,
        vector: &Vector,
        limit: usize,
    ) -> Result<Vec<(&Centroid, f32)>, CentroidCRDTError> {
        let mut distances: Vec<(&Centroid, f32)> = self
            .centroids
            .values()
            .map(|c| (c, c.distance_to(vector)))
            .collect();
        if distances.iter().any(|(_, d)| !d.is_finite()) {
            return Err(CentroidCRDTError::InvalidDistance);
        }

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        distances.truncate(limit);
        Ok(distances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_create_centroid() {
        let node_id = Uuid::new_v4();
        let mut crdt = CentroidCRDT::new(node_id);

        let vector = Vector::new(vec![1.0, 2.0, 3.0]);
        let centroid_id = crdt.create_centroid(vector.clone());

        assert_eq!(crdt.centroids.len(), 1);
        assert_eq!(crdt.operations.len(), 1);

        let centroid = crdt.get_centroid(&centroid_id).unwrap();
        assert_eq!(centroid.vector.values, vector.values);
    }

    #[test]
    fn test_update_centroid() {
        let node_id = Uuid::new_v4();
        let mut crdt = CentroidCRDT::new(node_id);

        let vector1 = Vector::new(vec![1.0, 2.0, 3.0]);
        let centroid_id = crdt.create_centroid(vector1);

        let vector2 = Vector::new(vec![4.0, 5.0, 6.0]);
        let res = crdt.update_centroid(centroid_id, vector2);
        assert!(res.is_ok());

        assert_eq!(crdt.operations.len(), 2);

        let centroid = crdt.get_centroid(&centroid_id).unwrap();
        // After updating, the vector should be a weighted average of the original and new vector
        // With 1 existing element and 1 new element, the weights are 1/2 each
        // So the expected value is (1.0*1 + 4.0*1)/2, (2.0*1 + 5.0*1)/2, (3.0*1 + 6.0*1)/2
        // Which equals 2.5, 3.5, 4.5
        assert!(centroid.vector.values[0] > 1.0 && centroid.vector.values[0] < 4.0);
        assert!(centroid.vector.values[1] > 2.0 && centroid.vector.values[1] < 5.0);
        assert!(centroid.vector.values[2] > 3.0 && centroid.vector.values[2] < 6.0);
        assert_eq!(centroid.count, 2);
    }

    #[test]
    fn test_delete_centroid() {
        let node_id = Uuid::new_v4();
        let mut crdt = CentroidCRDT::new(node_id);

        let vector = Vector::new(vec![1.0, 2.0, 3.0]);
        let centroid_id = crdt.create_centroid(vector.clone());

        let res = crdt.delete_centroid(centroid_id);
        assert!(res.is_ok());

        assert_eq!(crdt.centroids.len(), 0);
        assert_eq!(crdt.operations.len(), 2);
    }

    #[test]
    fn test_merge() {
        // Create two CRDTs
        let node_id1 = Uuid::new_v4();
        let node_id2 = Uuid::new_v4();
        let mut crdt1 = CentroidCRDT::new(node_id1);
        let mut crdt2 = CentroidCRDT::new(node_id2);

        // Add a centroid to the first CRDT
        let vector1 = Vector::new(vec![1.0, 2.0, 3.0]);
        let centroid_id1 = crdt1.create_centroid(vector1.clone());

        // Add a different centroid to the second CRDT
        let vector2 = Vector::new(vec![4.0, 5.0, 6.0]);
        let centroid_id2 = crdt2.create_centroid(vector2.clone());

        // Merge the second CRDT into the first
        crdt1.merge(&crdt2);

        // First CRDT should now have both centroids
        assert_eq!(crdt1.centroids.len(), 2);
        assert!(crdt1.centroids.contains_key(&centroid_id1));
        assert!(crdt1.centroids.contains_key(&centroid_id2));

        // Verify the centroids have the correct vectors
        let merged_centroid1 = crdt1.get_centroid(&centroid_id1).unwrap();
        let merged_centroid2 = crdt1.get_centroid(&centroid_id2).unwrap();

        assert_eq!(merged_centroid1.vector.values, vector1.values);
        assert_eq!(merged_centroid2.vector.values, vector2.values);

        // Operations should be merged too
        assert_eq!(crdt1.operations.len(), 2);
        assert_eq!(crdt1.observed.len(), 2);
    }

    #[test]
    fn test_concurrent_operations() {
        // Test handling of concurrent operations with different timestamps
        let node_id1 = Uuid::new_v4();
        let node_id2 = Uuid::new_v4();
        let mut crdt1 = CentroidCRDT::new(node_id1);
        let mut crdt2 = CentroidCRDT::new(node_id2);

        // Both CRDTs create a centroid with the same ID but different vectors
        let centroid_id = Uuid::new_v4();
        let vector1 = Vector::new(vec![1.0, 2.0, 3.0]);
        let vector2 = Vector::new(vec![4.0, 5.0, 6.0]);

        // Create the same centroid in both CRDTs with different timestamps
        let now = chrono::Utc::now();
        let later = now + chrono::Duration::seconds(10);

        // Earlier operation in CRDT1
        let op1 = CentroidOperation {
            id: Uuid::new_v4(),
            centroid_id,
            timestamp: now,
            operation_type: OperationType::Create(vector1),
        };

        // Later operation in CRDT2
        let op2 = CentroidOperation {
            id: Uuid::new_v4(),
            centroid_id,
            timestamp: later,
            operation_type: OperationType::Create(vector2.clone()),
        };

        crdt1.apply_operation(op1);
        crdt2.apply_operation(op2);

        // Merge CRDT2 into CRDT1
        crdt1.merge(&crdt2);

        // The centroid in CRDT1 should now have the vector from CRDT2
        // because it has a later timestamp
        let centroid = crdt1.get_centroid(&centroid_id).unwrap();
        assert_eq!(centroid.vector.values, vector2.values);
    }
}
