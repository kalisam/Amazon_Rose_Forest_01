# ADR 0006: R=3 replication with WAL shipping and repair

- Status: Proposed
- Date: 2025-08-10
- Tags: [replication, availability]

## Decision
Each shard has 3 replicas coordinated by control plane. Replication via WAL streaming; background repair fills gaps. Quorum reads optional; monotonic per-ID.

## Rationale
Simple, robust, and bandwidth-efficient.

## Consequences
+ Tolerates node churn
- Lag monitoring needed

## Operationalization
- Health/lag metrics; repair jobs; snapshot seeding for new replicas.
- Cutover protocol for leadership changes.
