# ADR 0005: Write-Ahead Log with RocksDB/LMDB

- Status: Proposed
- Date: 2025-08-10
- Tags: [storage, durability]

## Context
We need crash safety, replay, and clear write ordering.

## Decision
Each shard uses an append-only WAL (fsync policy configurable), primary KV (RocksDB/LMDB) for vectors/meta, and background ANN builders.

## Rationale
WAL guarantees durability & recovery; KV provides simple, fast random access.

## Consequences
+ Reliable recovery; easy replication via WAL shipping
- Extra disk writes; need compaction

## Operationalization
- Upsert path: WAL→KV→ANN queue; read-your-write shim.
- Export periodic snapshots; record manifests in control plane.
