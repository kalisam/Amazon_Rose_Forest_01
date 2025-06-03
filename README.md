# Amazon Rose Forest

A distributed vector database optimized for machine learning workloads.

## Features

- High-performance vector search with multiple distance metrics
- Distributed architecture with automatic sharding
- CRDT-based replication for high availability
- Circuit breaker pattern for fault tolerance
- Adaptive indexing using Hilbert space-filling curves
- NERV (Network-Efficient Resilient Vectorization) system

## Project Structure

```
src/
├── core/           # Core data structures and algorithms
│   ├── centroid.rs       # Centroid management
│   ├── centroid_crdt.rs  # CRDT implementation for centroids
│   ├── metrics.rs        # Performance metrics collection
│   └── vector.rs         # Vector operations
├── nerv/           # NERV system components
│   ├── evolution.rs      # Model evolution
│   ├── replication.rs    # Data replication
│   ├── runtime.rs        # Runtime environment
│   ├── synchrony.rs      # Clock synchronization
│   └── versioning.rs     # Data versioning
├── network/        # Networking components
│   └── circuit_breaker.rs # Circuit breaker implementation
├── sharding/       # Data sharding
│   ├── hilbert.rs        # Hilbert curve implementation
│   ├── manager.rs        # Shard management
│   └── migration.rs      # Shard migration
└── utils/          # Utility functions
    ├── config.rs         # Configuration management
    └── errors.rs         # Error types
```

## Development

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

## License

MIT