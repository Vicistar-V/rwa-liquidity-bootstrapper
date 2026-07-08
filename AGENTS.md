# AI-Assisted Development Context

This file helps AI coding assistants understand the project conventions.

## Project Overview

RWA Liquidity Bootstrapping Protocol — a Soroban smart contract suite for launching tokenized real-world assets on Stellar. Implements Liquidity Bootstrapping Pools, bonding curves, concentrated liquidity, fair launch mechanics, TWAP oracle, and compliance hooks.

## Key Conventions

- **Language:** Rust (edition 2021)
- **Target:** `wasm32-unknown-unknown`
- **SDK:** `soroban-sdk` v20
- **Workspace:** Cargo workspace at root, member crates in `contracts/*`
- **Math:** All arithmetic uses `i128` with 1e7 fixed-point scaling. No floats.
- **Errors:** Use the shared error enum in `contracts/math/src/error.rs`
- **Testing:** Unit tests in `#[cfg(test)] mod test` blocks per source file; integration tests in `tests/`
- **Formatting:** `cargo fmt` before commit
- **Linting:** `cargo clippy --workspace -- -D warnings`

## Repository Structure

```
contracts/
├── math/        — Fixed-point math lib (i128, 1e7 scale)
├── factory/     — Pool factory
├── lbp/         — LBP pool
├── bonding/     — Bonding curve pool
├── cl/          — Concentrated liquidity
├── fairlaunch/  — Fair launch controller
├── graduation/  — Graduation engine + DEX migration
├── oracle/      — TWAP oracle
├── compliance/  — KYC/AML bridge
└── rewards/     — LP rewards
```

## Build & Test Commands

```bash
cargo build --target wasm32-unknown-unknown --release
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt
```

## Priority Contribution Areas

1. **Curve math** — sigmoid/polynomial curves in `contracts/bonding/src/curves/`
2. **Fuzz testing** — property-based tests for math
3. **CL swap** — concentrated liquidity swap execution
4. **Compliance** — full ARCM integration
5. **Rewards** — LP rewards distribution
6. **Simulations** — Python simulators for curve types
7. **Issuer guides** — per-asset-class configuration docs
8. **Integrations** — Stellar DeFi adapters
