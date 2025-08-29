# ADR 0000: Use Architectural Decision Records

- Status: Proposed
- Date: 2025-08-10
- Owners: @kalisam
- Tags: [process, governance]

## Context
Many contributors; decisions risk being lost in chats and PR threads.

## Decision
Adopt lightweight ADRs in `/docs/adrs`. One decision per file, numbered,
small and immutable except for status. Major changes = new ADR superseding old.

## Rationale
Shared memory, on-boarding speed, auditability.

## Consequences
+ Clarity and traceability
- Slight overhead

## Operationalization
- New decisions require ADR + label `needs-adr` on PRs.
- Weekly triage: move *Proposed* → *Accepted* when merged and implemented.

## References
- https://adr.github.io/
