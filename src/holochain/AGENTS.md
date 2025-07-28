# Holochain Module

See the [root AGENTS](../../AGENTS.md) for the overall development workflow.

## Purpose
Provides Holochain DNA integration and zome utilities.

## Build
Some functions expect a running Holochain conductor. Enable the conductor feature with:

```
cargo +nightly build --features holochain_conductor
```

## Notes
Standard tests run via `cargo test`.
