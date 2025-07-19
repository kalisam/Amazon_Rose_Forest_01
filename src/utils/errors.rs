use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Vector not found: {0}")]
    VectorNotFound(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Shard not found: {0}")]
    ShardNotFound(String),

    #[error("Invalid dimensions: expected {expected}, got {actual}")]
    InvalidDimensions { expected: usize, actual: usize },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Operation timeout after {0}ms")]
    Timeout(u64),

    #[error("Concurrent modification error: {0}")]
    ConcurrentModification(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}
