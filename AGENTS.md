# Contributor Guide

## Overview
Amazon Rose Forest is a distributed vector database built with Rust. Core logic lives under `src/` and tests are in `tests/`.

## Development
- Build with `cargo build`.
- Run `cargo +nightly build --features holochain_conductor` to include Holochain integration.
- Format the code with `cargo fmt --all`.
- Lint with `cargo clippy --all`.

## Testing
- Run `cargo test --all` for the full test suite.
- Benchmarks can be checked with `cargo bench --no-run`.

## PR Instructions
- Use the title format `[amazon_rose_forest] <Title>`.
- Ensure formatting, lint, tests, and benchmarks pass before opening a PR.
