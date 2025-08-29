# ADR 0007: gRPC/HTTP API with streaming search

- Status: Proposed
- Date: 2025-08-10
- Tags: [api, clients]

## Decision
Define protobuf as single source of truth. Expose:
- gateway (HTTP+WS) for browsers,
- gRPC for services/operators.

## Rationale
Typed contracts, streaming top-K, good tooling.

## Consequences
+ Cross-language clients
- More CI (proto lint, breaking-change checks)

## Operationalization
- Repo `/api` with protos + buf.build.
- Contract tests; generated SDKs pinned by tag.
