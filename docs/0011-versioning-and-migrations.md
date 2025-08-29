# ADR 0011: Dual-write & dark-read for index upgrades

- Status: Proposed
- Date: 2025-08-10
- Tags: [migrations, safety]

## Decision
For index schema/engine changes: dual-write to new index; dark-read comparison until error budget ok; then cut over; keep rollback window.

## Rationale
Zero-downtime upgrades; measure impact before switching.

## Operationalization
- Feature flags; migration controller; telemetry gating.
