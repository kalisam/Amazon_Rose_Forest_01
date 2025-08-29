# ADR 0010: Periodic snapshots to IPFS/S3 with manifests

- Status: Proposed
- Date: 2025-08-10
- Tags: [backup, disaster-recovery]

## Decision
Shard snapshots + ANN state exported periodically; content-addressed; manifests referenced in control plane.

## Rationale
Fast bootstrap, reproducible deployments, verifiable integrity.

## Operationalization
- Schedule + retention; restore tooling; hash verification in CI.
