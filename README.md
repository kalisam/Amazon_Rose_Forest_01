# Amazon Rose Forest

A distributed vector database optimized for machine learning workloads.

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

## Performance

The codebase has been optimized for performance with the following features:

- SIMD acceleration for vector operations when possible
- Efficient batch processing capabilities
- Optimized distance calculations
- Benchmarking tools to measure performance

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
    "engine": "memory",
    "path": "./data/vectors",
    "cache_size_mb": 1024
  },
  "sharding": {
    "num_shards": 16,
    "replication_factor": 3,
    "auto_rebalance": true
  }
}
```

## License

MIT