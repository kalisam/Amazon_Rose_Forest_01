# Amazon Rose Forest

A distributed vector database optimized for machine learning workloads with Holochain integration.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- High-performance vector search with multiple distance metrics (Euclidean, Cosine, Manhattan, Hamming)
- SIMD-accelerated vector operations for improved performance
- Distributed architecture with automatic sharding
- CRDT-based replication for high availability
- Circuit breaker pattern for fault tolerance
- Adaptive indexing using Hilbert space-filling curves
- NERV (Network-Efficient Resilient Vectorization) system
- Prometheus-compatible metrics
- Holochain integration for distributed and secure storage
- Community arbitration system with trinary decision logic
- Transparent audit trails for all system decisions

## Project Structure

```
src/
├── core/           # Core data structures and algorithms
│   ├── centroid.rs       # Centroid management
│   ├── centroid_crdt.rs  # CRDT implementation for centroids
│   ├── metrics.rs        # Performance metrics collection
│   └── vector.rs         # Vector operations
├── holochain/      # Holochain integration
│   ├── arbitration.rs    # Community arbitration system
│   ├── dna.rs            # DNA configuration
│   ├── entries.rs        # Entry definitions
│   ├── transparency.rs   # Audit trail system
│   ├── utils.rs          # Utility functions
│   └── zome.rs           # Zome functions
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
│   ├── migration.rs      # Shard migration
│   └── vector_index.rs   # Vector indexing
└── utils/          # Utility functions
    ├── config.rs         # Configuration management
    └── errors.rs         # Error types
```

## Performance

The codebase has been optimized for performance with the following features:

- SIMD acceleration for vector operations when possible
- Efficient batch processing capabilities
- Optimized distance calculations
- Benchmarking tools to measure performance
- Holochain DHT for distributed storage and retrieval

## Holochain Integration

Amazon Rose Forest integrates with Holochain to provide:

1. **Distributed Storage**: Vectors and metadata stored in the Holochain DHT
2. **Community Arbitration**: Trinary decision logic (Resolve/Review/Reject)
3. **Transparent Audit Trails**: Cryptographic proof of all system decisions
4. **Agent-Centric Security**: Authentication and authorization via Holochain

## Development

### Building

```bash
cargo build
```

To enable functions that interact with the Holochain conductor, compile with the
`holochain_conductor` feature:

```bash
cargo build --features holochain_conductor
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

### Benchmarking

```bash
cargo bench
```

## Configuration

Amazon Rose Forest can be configured through a JSON configuration file. Here's an example configuration:

```json
{
  "node": {
    "id": "node-1",
    "host": "127.0.0.1",
    "port": 9000,
    "data_dir": "./data"
  },
  "network": {
    "peers": [
      "node-2:9000",
      "node-3:9000"
    ],
    "timeout_ms": 5000,
    "retry_interval_ms": 1000,
    "max_retries": 3
  },
  "storage": {
    "engine": "holochain",
    "path": "./data/vectors",
    "cache_size_mb": 1024
  },
  "sharding": {
    "num_shards": 16,
    "replication_factor": 3,
    "auto_rebalance": true
  },
  "holochain": {
    "conductor_path": "./holochain",
    "admin_port": 8000,
    "app_port": 8001
  }
}
```

## License

MIT