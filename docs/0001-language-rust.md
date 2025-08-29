# ADR 0001: Implement core services in Rust

- Status: Proposed
- Date: 2025-08-10
- Tags: [language]

## Context
We need low latency ANN search, memory safety, and cross-platform binaries.

## Decision
Use Rust for data-plane services (router, shard, engine) and most control-plane clients.

## Rationale
- Zero-cost abstractions, strong tooling (tokio, tonic).
- Safer than C++ for concurrent, long-running services.
- Ecosystem support for ANN, RocksDB/LMDB, OpenTelemetry.

## Consequences
+ Performance, safety
- Steeper learning curve vs. Python/Go

## Alternatives
- Go: simpler, but GC hiccups, fewer low-level ANN libs.
- C++: perf but higher maintenance risk.

## Operationalization
- Enforce `clippy` + `rustfmt` in CI.
- Minimal MSRV pinned in `rust-toolchain.toml`.
