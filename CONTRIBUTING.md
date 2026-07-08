# Contributing to RWA Liquidity Bootstrapping Protocol

First off, thanks for taking the time to contribute! 🎉

## Code of Conduct

This project adheres to a [Code of Conduct](./CODE_OF_CONDUCT.md). By participating you agree to uphold its terms.

## How to Contribute

### 1. Find or Open an Issue

- Browse [open issues](https://github.com/your-org/rwa-liquidity-bootstrapping-protocol/issues) for areas you'd like to work on.
- Issues labeled `good first issue` are ideal for newcomers.
- If you spot a bug or have an idea, open a new issue before writing code so we can discuss it.

### 2. Set Up Your Environment

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli --features opt

# Clone and build
git clone https://github.com/your-org/rwa-liquidity-bootstrapping-protocol.git
cd rwa-liquidity-bootstrapping-protocol
cargo build --target wasm32-unknown-unknown --release
```

### 3. Create a Feature Branch

```bash
git checkout -b fix/your-fix        # for bug fixes
git checkout -b feature/your-thing  # for new features
```

### 4. Make Your Changes

- Follow existing code style — match the patterns you see in neighboring files.
- Use the same fixed-point math conventions (`i128` with 1e7 scaling).
- Write tests for any new logic.
- Keep pull requests focused on a single concern.

### 5. Run Tests

```bash
# All unit tests
cargo test --workspace

# Integration tests
cargo test --test integration

# Lint
cargo clippy --workspace -- -D warnings
```

### 6. Submit a Pull Request

- PRs should target the `main` branch.
- Fill out the [PR template](./.github/PULL_REQUEST_TEMPLATE.md).
- A maintainer will review your changes — expect constructive feedback.

## Project Structure

```
contracts/
├── math/          — Shared fixed-point AMM math library
├── factory/       — Pool factory (deploys and tracks pools)
├── lbp/           — Liquidity Bootstrapping Pool contract
├── bonding/       — Bonding curve pool contract
├── cl/            — Concentrated liquidity module
├── fairlaunch/    — Anti-whale, blackout, cooldown controller
├── graduation/    — Graduation engine + DEX migration
├── oracle/        — TWAP oracle
├── compliance/    — KYC/AML compliance bridge (ARCM)
└── rewards/       — LP rewards distributor
```

## Coding Conventions

- **Rust edition:** 2021
- **Formatting:** `cargo fmt` (run before committing)
- **Errors:** Use the project's shared error enum (`contracts/math/src/error.rs`)
- **Math:** All arithmetic uses `i128` with 1e7 fixed-point scaling. Never use floats.
- **Tests:** Unit tests live in a `#[cfg(test)] mod test` at the bottom of each source file. Integration tests go in `tests/`.

## Priority Contribution Areas

| Area | Description | Where to Start |
|------|-------------|----------------|
| Curve math | Implement sigmoid/polynomial curves | `contracts/bonding/src/curves/` |
| Fuzz testing | Property-based tests for math edge cases | `contracts/math/src/` |
| Simulations | Python simulators for new curve types | `simulations/` (create if missing) |
| Issuer guides | Configuration docs per asset class | `docs/` (create if missing) |
| Integrations | Adapters for Stellar DeFi protocols | `contracts/compliance/src/` |

## Need Help?

Open a [discussion](https://github.com/your-org/rwa-liquidity-bootstrapping-protocol/discussions) or ask in the project's community channel.
