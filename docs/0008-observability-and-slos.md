# ADR 0008: OpenTelemetry + Prometheus; declared SLOs

- Status: Proposed
- Date: 2025-08-10
- Tags: [observability, reliability]

## Decision
Trace across gateway→router→shards→aggregator; metrics for p50/p95 latency, ef_search, recall@K, replica lag, build lag.

## Rationale
You can’t improve what you can’t see.

## SLOs (initial)
- Intra-cluster p95 top-K ≤ 150 ms at K=10
- Recall@10 ≥ 0.95 on calibration set
- Replica WAL lag p95 ≤ 3 s

## Operationalization
- OTLP exporter, Prometheus scrape; dashboards + alert rules.
