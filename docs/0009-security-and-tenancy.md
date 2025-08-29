# ADR 0009: Capability-based auth; per-tenant isolation

- Status: Proposed
- Date: 2025-08-10
- Tags: [security, multi-tenancy]

## Decision
Use Holochain capabilities for auth; tenant_id propagated end-to-end; encrypt at-rest (per-tenant keys); mTLS on transport.

## Consequences
+ Fine-grained access; aligns with agent-centric model
- Key management complexity

## Operationalization
- Cap-grant validation middleware; secrets via KMS or age/miniLock stores.
