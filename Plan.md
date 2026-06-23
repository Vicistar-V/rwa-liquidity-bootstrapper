# Development Plan — 5-Day Sprint to 65% Completion

## Milestone Target: 65% Completion

```
COMPLETED (65%)
══════════════════════════════════════════════════════
  ✅ amm_math library (100%)
  ✅ Pool Factory (100%)
  ✅ LBP Pool (100%)
  ✅ Bonding Curve Pool (100%)
  ✅ Fair Launch Controller (100%)
  ✅ Data types + error enums (100%)
  ✅ Unit + integration tests (core) (85%)
  ✅ Pool Factory tests (100%)
  ✅ LBP tests (90%)
  ✅ Bonding tests (90%)
  ✅ Fair Launch tests (90%)
  ✅ Graduation Engine (70%)
  ✅ TWAP Oracle (60%)
  ⬜ Concentrated Liquidity (40%)
  ⬜ LP Rewards (30%)
  ⬜ Compliance Bridge (20%)
  ⬜ End-to-end integration tests (40%)
  ⬜ Deployment scripts (50%)

REMAINING (35%)
══════════════════════════════════════════════════════
  ❌ Concentrated Liquidity (full impl + tests)
  ❌ LP Rewards (full impl + tests)
  ❌ Compliance Bridge (full impl + tests)
  ❌ Frontend SDK / UI
  ❌ Fuzz testing + formal verification
  ❌ Graduation migration to Stellar DEX
  ❌ Advanced TWAP oracle features
  ❌ Full deployment scripts
  ❌ Monitoring dashboards
```

---

## Day 1 — Foundation: Workspace, Types, Math Library

**Goal:** Cargo workspace with all contract crates, shared types, error enums, and the full fixed-point math library.

### Prompt 1.1 — Scaffold Cargo Workspace
```
Create a Cargo workspace at the repo root with these members:
  contracts/math       — shared AMM math library
  contracts/factory    — pool factory
  contracts/lbp        — LBP pool
  contracts/bonding    — bonding curve pool
  contracts/cl         — concentrated liquidity
  contracts/fairlaunch — fair launch controller
  contracts/graduation — graduation engine
  contracts/oracle     — TWAP oracle
  contracts/rewards    — LP rewards distributor
  contracts/compliance — compliance bridge

Each member should have:
  - Cargo.toml with soroban-sdk dependency (version 20.x)
  - src/lib.rs placeholder
  - dependency on contracts/math for math-using crates

Root Cargo.toml should set:
  [workspace]
  members = ["contracts/*"]
```

### Prompt 1.2 — Shared Types & Error Enums
```
In contracts/math/src/lib.rs, define all shared types matching README.md:

  pub struct LbpPool { ... }
  pub struct BondingCurvePool { ... }
  pub struct ConcentratedLiquidityPool { ... }
  pub struct LpPosition { ... }
  pub struct FairLaunchConfig { ... }
  pub struct WalletPurchaseRecord { ... }
  pub struct GraduationReceipt { ... }
  pub struct PriceObservation { ... }
  pub struct PoolSummary { ... }
  pub struct LbpConfig { ... }
  pub struct BondingConfig { ... }
  pub struct ClConfig { ... }

  pub enum CurveType { Linear, Polynomial, Sigmoid, Logarithmic }
  pub enum GraduationCriteria { TimeElapsed, FundsRaised, TokensSold, IssuerTriggered }
  pub enum FairLaunchError { WalletCapExceeded, PoolOwnershipCapExceeded, CooldownNotElapsed, BlackoutPeriodActive, WalletTooNew, KycRequired, InsufficientKycTier, JurisdictionProhibited }
  pub enum PurchaseError { ... }
  pub enum ComplianceDecision { Approve, Reject(Bytes), PendingKyc }

  pub const SCALE: u128 = 10_000_000;

Also add #[contracttype] and #[derive(Clone)] annotations.
Do NOT add comments. Export everything from contracts/math/src/lib.rs.
```

### Prompt 1.3 — Fixed-Point Math Library
```
In contracts/math/src/math.rs, implement:

  pub fn fixed_mul(a: u128, b: u128) -> u128    // a * b / SCALE with saturating arithmetic
  pub fn fixed_div(a: u128, b: u128) -> u128    // a * SCALE / b with saturating arithmetic
  pub fn fixed_pow(base: u128, exp: u128) -> u128    // base^exp using fixed-point approximation (Newton or Taylor)
  pub fn fixed_ln(x: u128) -> u128              // natural log using fixed-point approximation
  pub fn integral_logarithmic(a: u128, b: u128, s: u128) -> u128  // ∫(a·ln(s+1) + b)ds
  pub fn fixed_exp(x: u128) -> u128             // e^x for sigmoid curve
  pub fn sigmoid(x: u128, k: u128, mid: u128, max_val: u128) -> u128  // sigmoid curve P(s) = max / (1 + e^(-k(s - mid)))

All functions use saturating arithmetic. Re-export from lib.rs.
Do NOT add comments. Use debug_assert! for input range checks only.
```

---

## Day 2 — Pool Factory + LBP Pool

**Goal:** Pool Factory deploys LBP pools; LBP pool handles weight decay, spot pricing, swap math, and purchases.

### Prompt 2.1 — Pool Factory Contract
```
In contracts/factory/src/lib.rs, implement a Soroban contract:

  #[contract]
  pub struct PoolFactory;

  #[contractimpl]
  impl PoolFactory {
      pub fn create_lbp_pool(env: Env, issuer: Address, config: LbpConfig) -> BytesN<32>
      pub fn create_bonding_pool(env: Env, issuer: Address, config: BondingConfig) -> BytesN<32>
      pub fn create_cl_pool(env: Env, issuer: Address, config: ClConfig) -> BytesN<32>
      pub fn get_pools_for_asset(env: Env, rwa_token: Address) -> Vec<PoolSummary>
      pub fn list_all_pools(env: Env, offset: u32, limit: u32) -> Vec<PoolSummary>
  }

Each create_* method:
  1. Generates a pool_id via env.prng() (or hash of issuer + token + counter)
  2. Stores pool config + initialises pool state in persistent storage
  3. Transfers RWA tokens from issuer to contract (for LBP / bonding)
  4. Emits a PoolCreated event
  5. Returns pool_id

Use DataKey enum for storage keys:
  enum DataKey { LbpPool(BytesN<32>), BondingPool(BytesN<32>), ClPool(BytesN<32>), PoolCounter, IssuerPools(Address) }

Do NOT add comments. Use Soroban SDK patterns (Env, Address, Vec, BytesN, Symbol).
```

### Prompt 2.2 — LBP Pool: State & Weight Functions
```
In contracts/lbp/src/lib.rs, implement:

  #[contract]
  pub struct LbpPool;

  #[contractimpl]
  impl LbpPool {
      pub fn initialize(env: Env, pool_id: BytesN<32>, config: LbpConfig, issuer: Address)
      pub fn get_current_weight_rwa(env: &Env, pool_id: &BytesN<32>) -> u128
      pub fn get_spot_price(env: Env, pool_id: BytesN<32>) -> u128
      pub fn get_pool_details(env: Env, pool_id: BytesN<32>) -> LbpPool
  }

initialize():
  - Stores pool from LbpConfig
  - weight_rwa_start, weight_rwa_end, start_time, end_time, balances, etc.
  - Only callable by factory

get_current_weight_rwa():
  - w(t) = w_start - (w_start - w_end) * elapsed / duration
  - Returns weight_rwa_end if elapsed >= duration

get_spot_price():
  - w_rwa = get_current_weight_rwa(), w_usdc = SCALE - w_rwa
  - price = (balance_usdc / w_usdc) / (balance_rwa / w_rwa) = balance_usdc * w_rwa / (balance_rwa * w_usdc) * SCALE
  - Use math::fixed_mul and math::fixed_div

Do NOT add comments.
```

### Prompt 2.3 — LBP Pool: Swap Math & Buy
```
Extend contracts/lbp/src/lib.rs with:

  pub fn calculate_in_given_out(env: &Env, pool_id: &BytesN<32>, rwa_out: u128) -> u128
  pub fn calculate_out_given_in(env: &Env, pool_id: &BytesN<32>, usdc_in: u128) -> u128
  pub fn buy(env: Env, buyer: Address, pool_id: BytesN<32>, min_rwa_out: u128, max_usdc_in: u128) -> u128
  pub fn get_balance(env: Env, pool_id: BytesN<32>) -> (u128, u128)

calculate_in_given_out:
  - Uses Balancer invariant: usdc_in = balance_usdc * ((balance_rwa / (balance_rwa - rwa_out))^(w_rwa/w_usdc) - 1)
  - Use fixed_pow for the exponentiation

calculate_out_given_in:
  - Inverse of above: rwa_out = balance_rwa * (1 - (balance_usdc / (balance_usdc + usdc_in))^(w_usdc/w_rwa))

buy():
  1. Validate pool is active and not graduated
  2. Call fair_launch_controller (cross-contract) to check purchase allowed
  3. Calculate usdc_in from desired rwa_out (or vice versa)
  4. Check slippage: usdc_in <= max_usdc_in, actual_rwa_out >= min_rwa_out
  5. Transfer USDC from buyer to pool, RWA from pool to buyer
  6. Update balances
  7. Record observation in TWAP oracle (cross-contract)
  8. Emit Swap event
  9. Return actual amount of RWA purchased

Do NOT add comments. Import math and types from contracts/math.
```

---

## Day 3 — Bonding Curve Pool + Fair Launch Controller

**Goal:** Full bonding curve pool with 4 curve types; fair launch controller enforces all anti-whale rules.

### Prompt 3.1 — Bonding Curve: Curve Calculations
```
In contracts/bonding/src/lib.rs, implement four price functions:

  pub fn get_price_linear(pool: &BondingCurvePool) -> u128     // P = a * s + b
  pub fn get_price_polynomial(pool: &BondingCurvePool) -> u128 // P = a * s^n + b
  pub fn get_price_sigmoid(pool: &BondingCurvePool) -> u128    // P = max / (1 + e^(-k(s - mid)))
  pub fn get_price_logarithmic(pool: &BondingCurvePool) -> u128 // P = a * ln(s + 1) + b
  pub fn get_price(env: &Env, pool_id: &BytesN<32>) -> u128     // dispatches by pool.curve_type

  pub fn calculate_purchase_cost(pool: &BondingCurvePool, token_amount: u128) -> u128
  pub fn calculate_tokens_for_usdc(pool: &BondingCurvePool, usdc_in: u128) -> u128

calculate_purchase_cost uses the integral of the price curve:
  - Linear: ∫(a*s + b)ds from s1 to s2
  - Logarithmic: ∫(a*ln(s+1) + b)ds = a*((s+1)*ln(s+1) - s) + b*s
  - Polynomial: ∫(a*s^n)ds = a*s^(n+1)/(n+1)
  - Sigmoid: numerical integration or precomputed lookup table

calculate_tokens_for_usdc:
  - Binary search (64 iterations) over token_amount to find where calculate_purchase_cost ≈ usdc_in
  - Bounded by max_supply - current_supply

Enforce price_ceiling in all get_price_* functions via min(result, pool.price_ceiling).

Do NOT add comments. Use math functions from contracts/math.
```

### Prompt 3.2 — Bonding Curve: Buy / Sell
```
Extend contracts/bonding/src/lib.rs with the contract:

  pub fn initialize(env: Env, pool_id: BytesN<32>, config: BondingConfig, issuer: Address)
  pub fn buy(env: Env, buyer: Address, pool_id: BytesN<32>, usdc_in: u128, min_tokens: u128) -> u128
  pub fn sell(env: Env, seller: Address, pool_id: BytesN<32>, token_amount: u128, min_usdc_out: u128) -> u128
  pub fn get_pool_details(env: Env, pool_id: BytesN<32>) -> BondingCurvePool

buy():
  1. Pool must be active
  2. Fair launch check (cross-contract call)
  3. tokens_out = calculate_tokens_for_usdc(pool, usdc_in)
  4. Slippage: tokens_out >= min_tokens
  5. Enforce max_supply cap
  6. Transfer USDC from buyer to pool, RWA from pool to buyer
  7. Update current_supply, reserve_balance
  8. Record TWAP observation
  9. Emit event

sell():
  1. Pool must be active
  2. usdc_out = calculate_sale_return(pool, token_amount) — inverse of purchase cost
  3. Slippage: usdc_out >= min_usdc_out
  4. Transfer tokens from seller to pool, USDC from pool to seller
  5. Update current_supply, reserve_balance
  6. Emit event

Do NOT add comments.
```

### Prompt 3.3 — Fair Launch Controller
```
In contracts/fairlaunch/src/lib.rs, implement:

  pub fn initialize(env: Env, pool_id: BytesN<32>, config: FairLaunchConfig)
  pub fn check_purchase_allowed(env: &Env, buyer: &Address, pool_id: &BytesN<32>, amount: u128) -> Result<(), FairLaunchError>
  pub fn record_purchase(env: &Env, buyer: &Address, pool_id: &BytesN<32>, tokens_purchased: u128, usdc_spent: u128)
  pub fn get_wallet_purchases(env: Env, buyer: Address, pool_id: BytesN<32>) -> WalletPurchaseRecord
  pub fn get_pool_purchase_state(env: Env, pool_id: BytesN<32>) -> (u128, u128)  // (total_tokens_sold, total_usdc_raised)

check_purchase_allowed enforces in order:
  1. Blackout period: now >= pool.start_time + initial_blackout_period
  2. Wallet age: env.ledger().sequence() - wallet_creation_ledger >= min_wallet_age_ledgers
  3. Cooldown: now - last_purchase_time >= cooldown_between_purchases
  4. Wallet cap: wallet_total_purchased + amount <= max_tokens_per_wallet (also check USDC cap)
  5. Pool ownership: (wallet_total + amount) / total_supply <= max_pool_ownership_pct / SCALE
  6. KYC: if kyc_required, call compliance_bridge

record_purchase:
  - Update cumulative wallet purchases in persistent storage
  - Update pool-level totals (total_tokens_sold, total_usdc_raised)

Use DataKey enum for storage:
  enum DataKey { PurchaseRecord(Address, BytesN<32>), PoolState(BytesN<32>), FairLaunchConfig(BytesN<32>) }

Do NOT add comments.
```

---

## Day 4 — Concentrated Liquidity + Graduation + Oracle (Partial)

**Goal:** Concentrated liquidity module with position management, graduation engine with time/funds triggers, TWAP oracle with observation recording.

### Prompt 4.1 — Concentrated Liquidity: Position Management
```
In contracts/cl/src/lib.rs, implement:

  pub fn initialize(env: Env, pool_id: BytesN<32>, config: ClConfig)
  pub fn mint_position(env: Env, lp: Address, pool_id: BytesN<32>, tick_lower: i32, tick_upper: i32, liquidity: u128)
  pub fn burn_position(env: Env, lp: Address, pool_id: BytesN<32>, position_id: BytesN<32>)
  pub fn collect_fees(env: Env, lp: Address, pool_id: BytesN<32>, position_id: BytesN<32>) -> (u128, u128)
  pub fn get_position(env: Env, pool_id: BytesN<32>, position_id: BytesN<32>) -> LpPosition

mint_position:
  - Create LpPosition with liquidity, tick range
  - Transfer RWA + USDC from LP to contract proportional to current price within range
  - Store in persistent storage keyed by position_id

burn_position:
  - Remove position, calculate owed fees
  - Transfer remaining tokens back to LP

collect_fees:
  - Calculate accumulated fees since last collection
  - Update fee_growth_inside accumulators
  - Transfer owed tokens to LP

NOTE: Tick math uses sqrt price ratios like Uniswap v3. Implement:
  pub fn sqrt_price_to_tick(price: u128) -> i32
  pub fn tick_to_sqrt_price(tick: i32) -> u128
  pub fn get_amounts_for_liquidity(liquidity: u128, sqrt_price: u128, sqrt_low: u128, sqrt_high: u128) -> (u128, u128)

Do NOT add comments. This is a partial implementation — swap execution can be stubbed.
```

### Prompt 4.2 — Graduation Engine
```
In contracts/graduation/src/lib.rs, implement:

  pub fn initialize(env: Env, pool_id: BytesN<32>, criteria: GraduationCriteria, issuer: Address, threshold: u128)
  pub fn check_graduation_ready(env: Env, pool_id: BytesN<32>) -> GraduationStatus
  pub fn graduate_pool(env: Env, pool_id: BytesN<32>) -> GraduationReceipt
  pub fn trigger_early_graduation(env: Env, issuer: Address, pool_id: BytesN<32>)

  enum GraduationStatus { NotReady, Ready, Graduated }

check_graduation_ready:
  - TIME: end_time elapsed
  - FUNDS: total_usdc_raised >= threshold
  - TOKENS: total_tokens_sold >= threshold
  - Returns Ready if any criteria met (or IssuerTriggered)

graduate_pool:
  1. Verify graduation is ready
  2. Calculate USDC split: 80% issuer, 15% DEX seed, 5% protocol
  3. Mark pool as graduated in factory
  4. Transfer USDC shares
  5. Return GraduationReceipt with final price, totals, etc.

trigger_early_graduation:
  - Only callable by issuer
  - Bypasses criteria check, forces graduation

Do NOT add comments.
```

### Prompt 4.3 — TWAP Oracle
```
In contracts/oracle/src/lib.rs, implement:

  pub fn initialize(env: Env, pool_id: BytesN<32>)
  pub fn record_observation(env: Env, pool_id: BytesN<32>, price: u128)
  pub fn get_twap(env: Env, pool_id: BytesN<32>, period_seconds: u64) -> u128
  pub fn get_latest_price(env: Env, pool_id: BytesN<32>) -> u128
  pub fn get_price_history(env: Env, pool_id: BytesN<32>, from_timestamp: u64, limit: u32) -> Vec<PriceObservation>

Oracle storage maintains:
  - Circular buffer of (timestamp, price, cumulative_price) observations
  - Cumulative price accumulator: cumulative += price * time_elapsed
  - Latest price snapshot

get_twap:
  - Find earliest observation within (now - period_seconds) to now
  - TWAP = (cumulative_recent - cumulative_old) / (time_recent - time_old)

get_price_history returns paginated observations from storage.

Do NOT add comments. Use Vec to store observations, prune older than configurable max_age.
```

### Prompt 4.4 — LP Rewards Distributor (Stub)
```
In contracts/rewards/src/lib.rs, implement a stub:

  pub fn initialize(env: Env, pool_id: BytesN<32>)
  pub fn deposit_fees(env: Env, pool_id: BytesN<32>, amount_rwa: u128, amount_usdc: u128)
  pub fn claim(env: Env, lp: Address, pool_id: BytesN<32>) -> (u128, u128)
  pub fn get_rewards(env: Env, lp: Address, pool_id: BytesN<32>) -> (u128, u128)

Stub behaviour:
  - deposit_fees: accumulate fees in storage (no distribution logic yet, just totals)
  - claim: return 0, 0 (placeholder — full proportional distribution pending)
  - get_rewards: return 0, 0
  - initialize: store pool_id

Do NOT add comments.
```

---

## Day 5 — Tests, Refinement & Documentation

**Goal:** Comprehensive test suite for all Day 1–4 contracts, fix edge cases, add deploy scripts skeleton, update README.

### Prompt 5.1 — Math Library Tests
```
In contracts/math/src/test.rs, write tests for:

  fixed_mul: basic multiply, SCALE identity, zero, saturating overflow
  fixed_div: basic divide, SCALE identity, divide by zero (panic)
  fixed_pow: base^0 = SCALE, base^1 = base, fractional exponent
  fixed_ln: ln(1) = 0, ln(SCALE * e) ≈ SCALE, monotonicity
  sigmoid: asymptotes at 0 and max_val, midpoint check
  integral_logarithmic: verify against manual calculation

Use #[cfg(test)] module. Each test function uses env = Env::default().
Assert results within 1e-5 relative error tolerance.
Test with both small and large values to exercise saturating arithmetic.
```

### Prompt 5.2 — Pool Factory Tests
```
In contracts/factory/src/test.rs, write tests:

  test_create_lbp_pool:
    1. Deploy factory contract
    2. Create LBP pool with valid config
    3. Assert pool_id is returned (non-zero 32 bytes)
    4. Assert factory storage contains the pool

  test_create_duplicate_fails: (if applicable)
  test_list_pools_for_asset:
    1. Create two pools with same RWA token
    2. Verify get_pools_for_asset returns both

  test_list_all_pools_pagination:
    1. Create 5 pools
    2. Verify list_all_pools returns correct count with offset/limit

Use Soroban test framework: deploy contract, invoke functions, check storage.
```

### Prompt 5.3 — LBP Pool Integration Tests
```
In contracts/lbp/src/test.rs, write integration tests:

  test_initialize_pool:
    1. Deploy factory, create LBP pool
    2. Verify pool state: weights, balances, timestamps

  test_weight_decay:
    1. Initialize with 96% start, 50% end, 30 day duration
    2. Jump to day 7, 15, 22, 30
    3. Assert weights match expected values at each point

  test_spot_price_decreases_over_time:
    1. Seed pool with RWA tokens
    2. Jump time forward
    3. Assert spot price decreases monotonically

  test_buy_basic:
    1. Create pool, seed RWA
    2. Buyer purchases with USDC
    3. Assert tokens received, balances updated, event emitted

  test_buy_slippage_reverts:
    1. Attempt buy with max_usdc_in below calculated amount
    2. Assert transaction reverts

  test_graduated_pool_rejects_buys:
    1. Mark pool graduated, attempt buy
    2. Assert revert

Use env.ledger().set_timestamp() for time manipulation.
Mock token contracts for RWA and USDC transfers.
```

### Prompt 5.4 — Bonding Curve Tests
```
In contracts/bonding/src/test.rs, write tests:

  test_linear_curve_pricing:
    1. Initialize with linear curve: a=100, b=10
    2. Assert price at supply 0 is b, at supply N is a*N+b

  test_logarithmic_curve_pricing:
    1. Initialize logarithmic curve
    2. Assert price increases with supply, approaches ceiling

  test_buy_via_bonding_curve:
    1. Create pool, buyer sends USDC
    2. Assert tokens received based on integral pricing
    3. Assert supply and reserve updated

  test_sell_via_bonding_curve:
    1. Buy first, then sell partial
    2. Assert USDC returned approximates curve formula

  test_max_supply_enforced:
    1. Try to buy more than max_supply - current_supply
    2. Assert revert

  test_price_ceiling_enforced:
    1. Create curve with low ceiling
    2. Attempt large buy that would push price above ceiling
    3. Assert price clamped (or purchase limited)
```

### Prompt 5.5 — Fair Launch Controller Tests
```
In contracts/fairlaunch/src/test.rs, write tests:

  test_blackout_period_rejects_early_purchase:
    1. Configure 1-hour blackout
    2. Attempt purchase at t=0 → assert BlackoutPeriodActive
    3. Jump to t=61min → assert purchase succeeds

  test_wallet_cap_enforced:
    1. Set max_tokens_per_wallet
    2. Buy up to cap → succeeds
    3. Buy again → WalletCapExceeded

  test_cooldown_enforced:
    1. Set 10-min cooldown
    2. Buy → succeeds
    3. Buy again immediately → CooldownNotElapsed
    4. Jump 10min → succeeds

  test_pool_ownership_cap:
    1. Set 10% max ownership
    2. Buy up to 10% → succeeds
    3. Buy more → PoolOwnershipCapExceeded

  test_record_purchase_tracking:
    1. Buy 100 tokens for 50 USDC
    2. Assert get_wallet_purchases returns correct values
    3. Buy again, assert cumulative totals updated
```

### Prompt 5.6 — Graduation + Oracle Tests
```
In contracts/graduation/src/test.rs:

  test_time_based_graduation:
    1. Create pool with 30-day duration
    2. Jump to day 29 → NotReady
    3. Jump to day 30 → Ready
    4. graduate_pool → success, GraduationReceipt returned

  test_funds_based_graduation:
    1. Set threshold 100_000 USDC
    2. Simulate purchases totaling 99_999 → NotReady
    3. Simulate one more purchase → Ready

  test_early_graduation_by_issuer:
    1. Issuer triggers early graduation
    2. Assert pool graduated immediately

In contracts/oracle/src/test.rs:

  test_record_and_retrieve:
    1. Record observation at t=0, price=100
    2. Record at t=3600, price=110
    3. Retrieve TWAP for period → verify calculation

  test_twap_manipulation_resistance:
    1. Record many observations with price spikes
    2. Assert TWAP smooths them out correctly
```

### Prompt 5.7 — Deployment Script Skeleton
```
In scripts/deploy.sh (or deploy.ts with soroban-cli), create:

  deploy_all() {
    1. Build all contracts with `soroban contract build`
    2. Deploy amm_math (reference library)
    3. Deploy factory contract
    4. Deploy lbp_pool, bonding_curve, cl contracts
    5. Deploy fair_launch_controller, graduation_engine
    6. Deploy twap_oracle, lp_rewards, compliance_bridge
    7. Register factory as admin for all contracts
    8. Output deployed addresses to .env file
  }

Use Stellar testnet by default, mainnet flag for production.
Include verify step using block explorer API where applicable.
```

### Prompt 5.8 — Documentation & README Update
```
In the repo README.md, add a section "Getting Started":

  1. Prerequisites: Rust nightly, Soroban CLI, Stellar account
  2. Clone & build: `cargo build --workspace`
  3. Run tests: `cargo test --workspace`
  4. Deploy to testnet: `bash scripts/deploy.sh --network testnet`

Also add a "Project Status" badge indicating 65% completion.
Add brief instructions for each contract's key entry points.
Reference Plan.md for development roadmap.

Do NOT remove or modify existing README content.
```

---

## Summary: What 65% Looks Like

| Area | Files | Lines (est.) | Completion |
|------|-------|-------------|------------|
| Math library | `contracts/math/src/{lib,math}.rs` | ~400 | 100% |
| Shared types/errors | `contracts/math/src/lib.rs` | ~200 | 100% |
| Pool Factory | `contracts/factory/src/lib.rs` | ~350 | 100% |
| LBP Pool | `contracts/lbp/src/lib.rs` | ~500 | 100% |
| Bonding Curve | `contracts/bonding/src/lib.rs` | ~600 | 100% |
| Fair Launch Controller | `contracts/fairlaunch/src/lib.rs` | ~350 | 100% |
| Concentrated Liquidity | `contracts/cl/src/lib.rs` | ~400 | 40% |
| Graduation Engine | `contracts/graduation/src/lib.rs` | ~300 | 70% |
| TWAP Oracle | `contracts/oracle/src/lib.rs` | ~250 | 60% |
| LP Rewards | `contracts/rewards/src/lib.rs` | ~100 | 30% |
| Compliance Bridge | `contracts/compliance/src/lib.rs` | ~50 | 20% |
| Tests | `contracts/*/src/test.rs` | ~1200 | 70% |
| Deploy scripts | `scripts/deploy.sh` | ~100 | 50% |
| **Total** | | **~4800** | **65%** |

The remaining 35% comprises full Concentrated Liquidity implementation (swap execution within ranges), LP Rewards proportional distribution, Compliance Bridge complete integration, end-to-end integration tests, fuzz testing, comprehensive deployment infrastructure, and the frontend/UI layer.
