# ADR 0002: Use Holochain for the control plane

- Status: Proposed
- Date: 2025-08-10
- Tags: [control-plane, identity, governance]

## Context
We need identity, capability-based auth, catalog metadata, audit trails, and peer discovery without central servers.

## Decision
Use Holochain source-chains + DHT for:
- node membership & capabilities,
- catalog of shards, replicas, schemas, index versions,
- placement/re-shard proposals and audit logs.

## Rationale
Agent-centric identity, append-only auditability, P2P replication adapted to our ethos.

## Consequences
+ Decentralized governance, tamper-evident history
- Need thin scheduler around DHT for efficient placement

## Alternatives
- etcd/consul: simpler but centralizes trust.
- libp2p+custom: more work, re-invent identity/audit.

## Operationalization
- Create `control` zomes: `catalog`, `membership`, `placement`, `policy`.
- Define schemas & validation rules; integration tests in CI.
