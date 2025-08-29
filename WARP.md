# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Essential commands

Build:
```
cargo build --release
```

Test (all crates, verbose):
```
cargo test --all --verbose
```

Run a single test:
```
# by test name (substring match)
cargo test it_works -- --nocapture

# by module path
cargo test some_module::some_test

# by integration test target name
cargo test --test vector_operations
```

Format:
```
cargo fmt --all
```

Lint:
```
cargo clippy --all -D warnings
```

Benchmarks:
```
# run benchmarks
cargo bench

# compile benches without running (mirrors CI)
cargo bench --no-run
```

## Start the node server

Recommended (works regardless of binary name):
```
# Linux/macOS
RUST_LOG=info cargo run --release -- --init

# Windows (PowerShell)
$env:RUST_LOG="info"; cargo run --release -- --init
```

After building, run the binary directly (verify the actual binary name in target/release):
```
# If the binary is named 'rose_node' as in README:
./target/release/rose_node --init

# If Cargo used the package name (amazon-rose-forest):
./target/release/amazon-rose-forest --init
# Windows:
.\target\release\amazon-rose-forest.exe --init
```

Server notes:
- The HTTP server exposes metrics and API endpoints; configuration flags `enable_metrics` and `enable_api` control availability (off returns 404 with descriptive body).
- Set RUST_LOG to control verbosity (e.g., info, debug).

## Architecture at a glance

- **Agent-centric distributed vector database**
  - Control plane: Holochain source-chains and DHT (membership, capabilities, catalog, placement).
  - Data plane: Sharded ANN-style vector storage and search.
- **Core runtime** (`nerv::runtime::Runtime`): starts services, exposes `shard_manager()`, integrates metrics.
- **Sharding and vector index**
  - `sharding::manager::ShardManager` orchestrates shards and vector indices.
  - `sharding::vector_index` with `DistanceMetric` (e.g., Cosine).
  - `sharding::hilbert::HilbertCurve` for space-filling keying.
- **Vector math and clustering**
  - `core::vector::Vector`, `core::centroid`, `core::centroid_crdt`, `core::hierarchical`.
- **Consciousness and symbiosis**
  - `consciousness::{ad4m_bridge, introspection, swarm}`, ad4m-client integration.
- **Intelligence**
  - `intelligence::{federated_learning, orchestrator}` for distributed learning and multi-agent coordination.
- **Governance**
  - `governance::{dao, zkp}` for participatory proposals and verifiable claims.
- **Network**
  - `network::circuit_breaker` for resilient calls.
- **Server**
  - server (warp-based HTTP/WebSocket endpoints; metrics and API switches).
- **Semantic/CRDT**
  - `semantic_crdt` for convergent shared state.

Entry point: `src/main.rs` wires Runtime, ShardManager, vector indexing, metrics reporting, and Darwin Gödel Machine components (validation pipeline, exploration strategy, self-improvement).

## ADR highlights

- **ADR 0002**: Use Holochain for the control plane (`docs/0002-control-plane-holochain.md`)
  - Identity, capability-based auth, shard/catalog metadata, placement proposals, audit trails.
- **ADR 0007**: gRPC/HTTP API with streaming search (`docs/0007-api-surface-and-protocols.md`)
  - Protobuf as source of truth, browser gateway (HTTP+WS), gRPC for services/operators.

Operational follow-ups:
- Define control zomes `catalog`, `membership`, `placement`, `policy`.
- Proto repo (`/api`) with linting and contract tests (buf.build suggested).

## Control-plane (Holochain) workflows

- **Features**:
  - Enable conductor code paths when developing control-plane integration:
    ```
    cargo build --features holochain_conductor
    cargo test --features holochain_conductor
    ```
- **Zomes to implement** (per ADR 0002):
  - `membership`: node join/leave, capabilities.
  - `catalog`: shards, replicas, schemas, index versions.
  - `placement`: proposals, re-shard actions, audit log entries.
  - `policy`: validation rules, governance hooks.
- **Patterns**:
  - Define entry types and validation callbacks in zomes; aim for append-only auditability.
  - Write integration tests for validation rules and DHT propagation.
- **Tooling**:
  - Pin Holochain crates to 0.5.4 family as in Cargo.toml for compatibility.

## Vector ops and sharding

- Typical flow:
  ```rust
  // initialize Runtime and metrics
  let mut runtime = Runtime::new(metrics.clone());
  runtime.start().await?;
  let shard_manager = runtime.shard_manager().expect("initialized");

  // create a shard and vector index
  let shard_id = shard_manager.create_shard("demo_shard").await?;
  let index = shard_manager
      .create_vector_index(shard_id, "demo_index", 60, DistanceMetric::Cosine)
      .await?;

  // add vectors with optional metadata
  let vector_id = shard_manager.add_vector(shard_id, Vector::random(60), None).await?;

  // search
  let results = shard_manager.search_vectors(shard_id, &Vector::random(60), 5).await?;
  ```
- **Tips**:
  - Dimensions must match the index's configured size.
  - Choose DistanceMetric per workload (Cosine, etc.).
  - Use `index.stats()` for visibility: `vector_count`, `bucket_count`, `avg_bucket_size`.

## Darwin Gödel Machine

- **Components**:
  - `ValidationPipeline` with stages:
    - `UnitTestStage`
    - `PerformanceBenchmarkStage`
    - `SecurityValidationStage`
  - `ExplorationStrategy` to propose modifications.
  - `SelfImprovementEngine` orchestrates `generate_modifications` and validation.
  - `RitualManager` to structure improvement cycles with stages (exploration, validation, deployment, reflection).
- **Thresholds**:
  - Example set in main.rs:
    - `unit_tests.pass_rate` ≥ 0.9
    - `performance.vector_search_latency_ms` ≤ 10
    - `security.vulnerability_score` ≤ 0.2
- **Extensibility**:
  - Add new validation stages by implementing a stage and registering it in ValidationPipeline.
  - Use metrics to gate promotions; run periodically via tokio tasks.

## Codacy MCP rules

- **After any file edit via tools** (edit_file or reapply):
  - Immediately run `codacy_cli_analyze` with:
    - `rootPath`: repository root (standard file path, not URL-encoded)
    - `file`: the edited file path
    - `tool`: leave empty/unset
  - If issues are found, propose and apply fixes, then re-run analysis.
- **After any package manager operation** or dependency change:
  - Run `codacy_cli_analyze` with:
    - `rootPath`: repository root
    - `tool`: `trivy`
    - `file`: leave empty
  - If vulnerabilities are found, stop and resolve before continuing.
- **If Codacy MCP tools are unavailable** or unreachable:
  - Try resetting the MCP extension.
  - If using VSCode, check Settings > Copilot > Enable MCP servers (https://github.com/settings/copilot/features).
  - If still blocked, contact Codacy support.
- **If an action using repository or organization returns 404**:
  - Offer to run `codacy_setup_repository` (only with user consent).
  - After setup, retry once.
- Do not install Codacy CLI manually; always use the Codacy MCP Server tools.

## Pre-commit checklist

```
cargo fmt --all -- --check
cargo clippy --all -D warnings
cargo build --verbose
cargo test --all --verbose
cargo bench --no-run
```

Optional features and crypto variants:
```
# Holochain conductor integration
cargo build --features holochain_conductor

# Alternative hashers
cargo build --no-default-features --features sha3
cargo build --no-default-features --features blake3
```
