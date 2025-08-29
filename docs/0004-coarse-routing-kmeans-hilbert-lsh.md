# ADR 0004: Pluggable coarse routing (K-means default; Hilbert, LSH optional)

- Status: Proposed
- Date: 2025-08-10
- Tags: [routing, sharding]

## Decision
Default coarse routing via global K-means centroids (catalog-managed).
Offer plugins: Hilbert space-filling keys and LSH for very high dims.

## Rationale
K-means adapts to data and minimizes cross-shard calls; Hilbert gives stable keys; LSH is robust to high-d variance.

## Consequences
+ Flexibility; can tune recall/latency
- More code paths to test

## Operationalization
- Router trait `CoarseIndex` with `select_shards(query, M)`.
- Control plane stores centroids and Hilbert params; runtime hot-swappable.
