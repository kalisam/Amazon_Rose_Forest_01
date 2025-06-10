use amazon_rose_forest::core::centroid_crdt::{CentroidOperation, OperationType};
use amazon_rose_forest::{CentroidCRDT, Vector};
use uuid::Uuid;

#[tokio::test]
async fn test_merge_concurrent_operations() {
    let node_id1 = Uuid::new_v4();
    let node_id2 = Uuid::new_v4();
    let mut crdt1 = CentroidCRDT::new(node_id1);
    let mut crdt2 = CentroidCRDT::new(node_id2);

    let centroid_id = Uuid::new_v4();
    let vector1 = Vector::new(vec![1.0, 2.0, 3.0]);
    let vector2 = Vector::new(vec![4.0, 5.0, 6.0]);

    let now = chrono::Utc::now();
    let later = now + chrono::Duration::seconds(5);

    let op1 = CentroidOperation {
        id: Uuid::new_v4(),
        centroid_id,
        timestamp: now,
        operation_type: OperationType::Create(vector1),
    };

    let op2 = CentroidOperation {
        id: Uuid::new_v4(),
        centroid_id,
        timestamp: later,
        operation_type: OperationType::Create(vector2.clone()),
    };

    crdt1.apply_operation(op1);
    crdt2.apply_operation(op2);

    crdt1.merge(&crdt2);

    let centroid = crdt1.get_centroid(&centroid_id).unwrap();
    assert_eq!(centroid.vector.values, vector2.values);
}
