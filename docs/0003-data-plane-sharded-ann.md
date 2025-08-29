# ADR 0003: Sharded ANN services with router/aggregator

- Status: Proposed
- Date: 2025-08-10
- Tags: [data-plane, ann, sharding]

## Context
Billions of vectors, low latency, heterogeneous peers.

## Decision
Separate API Gateway/Router/Aggregator from Shard Services (ANN per shard).
Router does coarse prefilter; shards do fine ANN; aggregator merges/reranks.

## Rationale
Classic fanout pattern; isolates hot path; easy horizontal scale.

## Consequences
+ Scales with shards; independent upgrades
- Extra hop (router) → needs efficient selection

## Operationalization
- Crates: `/router`, `/shard`, `/engine`.
- gRPC between components; backpressure and streaming responses.
