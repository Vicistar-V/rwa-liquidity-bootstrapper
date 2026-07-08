# 🌊 RWA Liquidity Bootstrapping Protocol
### *Decentralized Launchpad & Custom AMM Infrastructure for Newly Tokenized Real-World Assets on Stellar & Soroban*

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Stellar](https://img.shields.io/badge/network-Stellar-black?logo=stellar)
![Soroban](https://img.shields.io/badge/smart--contracts-Soroban-purple)
![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)
![Status](https://img.shields.io/badge/status-75%25%20Complete-orange)
![Tests](https://img.shields.io/badge/tests-22%20passing-brightgreen)
![AMM](https://img.shields.io/badge/AMM-Custom%20Curves-green)
![RWA](https://img.shields.io/badge/asset--type-Real--World--Assets-blue)

---

## 📋 Table of Contents

1. [Overview](#-overview)
2. [Problem Statement](#-problem-statement)
3. [Solution Architecture](#-solution-architecture)
4. [How It Works](#-how-it-works)
5. [System Architecture Diagram](#-system-architecture-diagram)
6. [AMM Curve Design](#-amm-curve-design)
7. [Liquidity Bootstrapping Pool (LBP)](#-liquidity-bootstrapping-pool-lbp)
8. [Bonding Curve Engine](#-bonding-curve-engine)
9. [Concentrated Liquidity Module](#-concentrated-liquidity-module)
10. [Smart Contract Design](#-smart-contract-design)
11. [Price Discovery Mechanism](#-price-discovery-mechanism)
12. [Anti-Whale & Fair Launch Mechanics](#-anti-whale--fair-launch-mechanics)
13. [Graduation & Liquidity Migration](#-graduation--liquidity-migration)
14. [Compliance Integration](#-compliance-integration)
15. [Supported RWA Asset Classes](#-supported-rwa-asset-classes)
16. [Tokenomics & Fee Structure](#-tokenomics--fee-structure)
17. [Stellar & Soroban Integration](#-stellar--soroban-integration)
18. [Security Model](#-security-model)
19. [Tech Stack](#-tech-stack)
20. [Repository Structure](#-repository-structure)
21. [Getting Started](#-getting-started)
22. [Contract Deployment](#-contract-deployment)
23. [API Reference](#-api-reference)
24. [Issuer Integration Guide](#-issuer-integration-guide)
25. [Testing](#-testing)
26. [Mathematical Appendix](#-mathematical-appendix)
27. [Roadmap](#-roadmap)
28. [Contributing](#-contributing)
29. [License](#-license)

---

## 🌐 Overview

The **RWA Liquidity Bootstrapping Protocol (RWA-LBP)** is a decentralized launchpad and custom automated market maker (AMM) infrastructure built on Stellar's Soroban smart contract platform. It solves the most critical barrier to tokenized real-world asset adoption: **the cold-start liquidity problem**.

RWA-LBP gives issuers of newly tokenized assets — real estate, commodities, private equity, carbon credits, and debt instruments — a purpose-built set of AMM curves and bootstrapping mechanics to achieve deep, organic liquidity without requiring massive upfront capital commitment on both sides of a trading pair.

Issuers seed **only their RWA tokens**. The protocol's time-weighted price decay curves, bonding curve mechanics, and concentrated liquidity modules do the rest — gradually attracting buyers, discovering fair market price, and building sustainable on-chain liquidity depth.

> RWA-LBP is what Balancer's LBP model would look like if it were purpose-built for real-world assets, with compliance hooks, KYC gating, and Stellar-native settlement.

### Core Capabilities at a Glance

| Capability | Description |
|------------|-------------|
| 🎯 **LBP Pools** | Time-weighted price decay pools — starts high, finds fair value naturally |
| 📈 **Bonding Curves** | Continuous price discovery as supply grows; no upfront liquidity needed |
| 🎯 **Concentrated Liquidity** | Capital-efficient liquidity within expected RWA price ranges |
| 🐋 **Anti-Whale Guards** | Per-wallet purchase caps + time-locks prevent early manipulation |
| 🔄 **Graduation System** | Auto-migrates liquidity to standard DEX once bootstrapping completes |
| 🪪 **Compliance Hooks** | Optional KYC/AML gating via ARCM middleware integration |
| 📊 **Price Oracle** | On-chain TWAP oracle built into every pool |
| 🌍 **Multi-Asset Support** | Works for real estate, commodities, equity, debt, carbon credits |

---

## 🔴 Problem Statement

### The Cold-Start Liquidity Trap

Every newly tokenized RWA faces the same brutal paradox on launch:

```
No liquidity → High slippage → No buyers → No liquidity
      ▲                                          │
      └──────────────────────────────────────────┘
                   THE COLD-START TRAP
```

Standard approaches fail in predictable ways:

| Approach | Why It Fails for RWAs |
|----------|----------------------|
| **Uniswap-style AMM (x×y=k)** | Requires equal value on both sides. Issuer must lock $1M in USDC to match $1M in tokens — capital they don't have |
| **Order book DEX** | Needs market makers. New RWA tokens have no market makers and no trading history |
| **Centralized exchange listing** | Requires connections, fees ($50K–$500K), and sacrifices decentralization |
| **Manual OTC sales** | Slow, non-transparent, no price discovery, regulatory grey area |
| **Yield farming incentives** | Attracts mercenary capital that exits the moment rewards drop — leaves no real liquidity |
| **IDO/launchpad (generic)** | Built for fungible governance tokens. Assumes price speculation — wrong mental model for yield-bearing RWAs |

### RWA-Specific Complications

Beyond the standard cold-start problem, RWAs introduce additional complexity:

```
1. PRICE IS NOT SPECULATIVE
   A tokenized apartment has a known appraised value.
   Generic AMMs allow wild price swings inappropriate
   for assets with underlying collateral.

2. TRANSFER RESTRICTIONS
   Many RWAs require KYC verification before purchase.
   Standard AMMs have no KYC hooks.

3. HOLDING PERIODS
   Securities regulations often require 90-day+ hold periods.
   Standard AMMs allow instant resale.

4. ACCREDITED INVESTOR RULES
   US and many other jurisdictions restrict RWA purchase
   to accredited investors. AMMs cannot enforce this.

5. YIELD EXPECTATIONS
   RWA buyers buy for yield, not price speculation.
   Bootstrapping needs to reflect yield-adjusted pricing.
```

**RWA-LBP is designed from the ground up to solve all six of these simultaneously.**

---

## 🧩 Solution Architecture

RWA-LBP is a **four-module system** operating across three phases:

```
┌─────────────────────────────────────────────────────────────────┐
│                    RWA-LBP MODULE STACK                         │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  MODULE 1: POOL FACTORY                                   │  │
│  │  Issuers deploy customized bootstrapping pools            │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │  MODULE 2: AMM CURVE ENGINE                               │  │
│  │  LBP | Bonding Curve | Concentrated Liquidity             │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │  MODULE 3: FAIR LAUNCH CONTROLLER                         │  │
│  │  Anti-whale | Time-locks | Purchase caps | KYC gate       │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │  MODULE 4: GRADUATION & MIGRATION ENGINE                  │  │
│  │  Auto-migrates to standard AMM once bootstrapping done    │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

PHASE 1: BOOTSTRAPPING (Days 1–30)
  LBP or Bonding Curve active
  Price decays from high → fair value
  Anti-whale guards active
  KYC gating enforced

PHASE 2: DEEPENING (Days 31–90)
  Concentrated liquidity module activated
  Liquidity providers rewarded
  TWAP oracle stabilizes
  Price range narrows around fair value

PHASE 3: GRADUATION (Day 90+)
  Bootstrapping pool closed
  Liquidity auto-migrated to standard DEX pool
  Token tradeable on open market
  TWAP oracle continues for downstream DeFi
```

---

## ⚙️ How It Works

### Issuer Journey — End to End

```
STEP 1: ISSUER CONFIGURES POOL
  Issuer calls pool_factory.create_pool(config)
  Selects: LBP | Bonding Curve | Concentrated Liquidity
  Sets: start_price, end_price, duration, purchase_cap,
        kyc_required, holding_period, compliance_contract

STEP 2: ISSUER SEEDS POOL (RWA TOKENS ONLY)
  Issuer deposits RWA tokens into pool
  NO USDC required from issuer at launch
  Pool contract holds tokens in escrow

STEP 3: BOOTSTRAPPING PHASE BEGINS
  Price starts at issuer-configured ceiling (e.g., 120% of appraised value)
  Price decays over time toward floor (e.g., 90% of appraised value)
  Buyers purchase at current curve price using USDC

STEP 4: BUYERS ENTER (KYC-GATED IF REQUIRED)
  Buyer calls pool.buy(amount, max_price_slippage)
  If KYC required → ARCM middleware verifies credential
  Purchase cap enforced (anti-whale)
  USDC flows into pool; RWA tokens flow to buyer
  Holding period lock applied to buyer's tokens

STEP 5: PRICE DISCOVERY COMPLETES
  As more buyers enter, curve mechanics find equilibrium
  TWAP oracle records price history
  Issuer receives USDC proceeds from sales

STEP 6: LIQUIDITY DEEPENING
  External LPs can now add USDC to concentrated range
  LP rewards distributed from protocol fee pool
  Price range tightens as liquidity deepens

STEP 7: GRADUATION
  Bootstrapping criteria met (time elapsed OR tokens sold threshold)
  Remaining pool liquidity auto-migrated to Stellar DEX / standard AMM
  Token enters free-market trading with deep liquidity base
  TWAP oracle continues serving downstream protocols
```

---

## 🏗️ System Architecture Diagram

```
                  ┌──────────────────────────────────────────────────────────┐
                  │                    STELLAR NETWORK                       │
                  │                                                          │
  ┌──────────┐    │  ┌────────────────────────────────────────────────────┐  │
  │  ISSUER  │───▶│  │            POOL FACTORY CONTRACT                   │  │
  │          │    │  │         (factory/src/lib.rs)                       │  │
  └──────────┘    │  │                                                    │  │
  Seeds RWA       │  │  create_pool(LBPConfig | BondingConfig | CLConfig) │  │
  tokens only     │  │  Deploys customized pool instance                  │  │
                  │  │  Stores pool registry                              │  │
                  │  └──────────────────┬─────────────────────────────────┘  │
                  │                     │ deploys                            │
                  │          ┌──────────┴──────────┐                        │
                  │          │                     │                        │
                  │          ▼                     ▼                        │
                  │  ┌───────────────┐   ┌─────────────────────────────┐   │
                  │  │  LBP POOL     │   │   BONDING CURVE POOL        │   │
                  │  │  CONTRACT     │   │   CONTRACT                  │   │
                  │  │               │   │                             │   │
                  │  │ Time-weighted │   │  Continuous mint/price      │   │
                  │  │ price decay   │   │  discovery                  │   │
                  │  │ w1(t)/w2(t)   │   │  P = f(supply)              │   │
                  │  └──────┬────────┘   └──────────────┬──────────────┘   │
                  │         │                           │                  │
                  │         └──────────┬────────────────┘                  │
                  │                    │                                   │
                  │                    ▼                                   │
                  │  ┌──────────────────────────────────────────────────┐  │
                  │  │       AMM CURVE ENGINE CONTRACT                  │  │
                  │  │       (amm/src/lib.rs)                           │  │
                  │  │                                                  │  │
                  │  │  • get_spot_price(pool_id, amount)               │  │
                  │  │  • calculate_out_given_in(...)                   │  │
                  │  │  • calculate_in_given_out(...)                   │  │
                  │  │  • update_weights(pool_id, elapsed_time)         │  │
                  │  │  • get_twap(pool_id, period)                     │  │
                  │  └──────────────────┬───────────────────────────────┘  │
                  │                     │                                  │
                  │          ┌──────────┴──────────────┐                  │
                  │          │                         │                  │
                  │          ▼                         ▼                  │
                  │  ┌───────────────────┐   ┌──────────────────────────┐ │
                  │  │  FAIR LAUNCH      │   │  COMPLIANCE HOOK         │ │
                  │  │  CONTROLLER       │   │  (Optional ARCM Bridge)  │ │
                  │  │                   │   │                          │ │
                  │  │  • Purchase caps  │   │  • KYC tier check        │ │
                  │  │  • Time locks     │   │  • Jurisdiction verify   │ │
                  │  │  • Anti-whale     │   │  • Holding period lock   │ │
                  │  │  • Vesting logic  │   │  • Accredited investor   │ │
                  │  └─────────┬─────────┘   └──────────────────────────┘ │
                  │            │                                           │
                  │            ▼                                           │
                  │  ┌──────────────────────────────────────────────────┐  │
                  │  │      GRADUATION & MIGRATION ENGINE               │  │
                  │  │      (graduation/src/lib.rs)                     │  │
                  │  │                                                  │  │
                  │  │  • Monitors graduation criteria                  │  │
                  │  │  • Closes bootstrapping pool                     │  │
                  │  │  • Migrates LP to Stellar DEX / standard AMM     │  │
                  │  │  • Transfers USDC proceeds to issuer             │  │
                  │  └──────────────────────────────────────────────────┘  │
                  │                                                         │
  ┌──────────┐    │  ┌──────────────────────────────────────────────────┐  │
  │ BUYERS   │───▶│  │         TWAP ORACLE CONTRACT                     │  │
  │          │    │  │  Accumulates price × time for downstream DeFi   │  │
  └──────────┘    │  └──────────────────────────────────────────────────┘  │
  ┌──────────┐    │                                                         │
  │   LPs    │───▶│  ┌──────────────────────────────────────────────────┐  │
  │          │    │  │         LP REWARDS DISTRIBUTOR                   │  │
  └──────────┘    │  │  Fee revenue → LP token holders                  │  │
                  │  └──────────────────────────────────────────────────┘  │
                  └─────────────────────────────────────────────────────────┘
```

---

## 📐 AMM Curve Design

RWA-LBP implements three specialized AMM curve types. Issuers select the curve best suited to their asset type and distribution goals.

### Curve Comparison Matrix

```
                    LBP                 BONDING CURVE        CONCENTRATED
                    ───────────────     ─────────────────    ────────────────
BEST FOR            Token launches      Continuous issuance  Post-bootstrap LP
                    with time limit     without time limit   depth building

ISSUER CAPITAL      RWA tokens only     RWA tokens only      USDC + RWA tokens
REQUIRED

PRICE BEHAVIOR      High → decays to    Rises with demand    Stable within range
                    fair value

WHALE PROTECTION    Strong (time-       Moderate             N/A (open LP)
                    weighted decay)

PRICE DISCOVERY     Excellent           Good                 Assumes known range

DURATION            Fixed (1–90 days)   Indefinite           Indefinite

GRADUATION          Auto-migrates       Manual or triggered  Already standard AMM
```

---

## 🏊 Liquidity Bootstrapping Pool (LBP)

The LBP is the flagship bootstrapping mechanism — a time-weighted AMM where the RWA token's weight decreases continuously over the bootstrapping period, causing price to decay naturally from a high starting point toward fair market value.

### Weight Function

```
The pool maintains two assets: RWA Token (T) and USDC (U)
Each has a time-varying weight: w_T(t) and w_U(t)

At time t during bootstrapping period [t_start, t_end]:

  w_T(t) = w_T_start + (w_T_end - w_T_start) × (t - t_start) / (t_end - t_start)
  w_U(t) = 1 - w_T(t)

Example configuration:
  w_T_start = 0.96  (96% RWA weight at launch — price is very high)
  w_T_end   = 0.50  (50% RWA weight at end — balanced pool)
  Duration  = 30 days

  Day  0: w_T = 0.96, w_U = 0.04  → Price = very high (few USDC needed)
  Day  7: w_T = 0.85, w_U = 0.15  → Price decaying
  Day 15: w_T = 0.73, w_U = 0.27  → Price at ~midpoint
  Day 22: w_T = 0.61, w_U = 0.39  → Price approaching fair value
  Day 30: w_T = 0.50, w_U = 0.50  → Price at fair value equilibrium
```

### Spot Price Formula

```
At any time t, the spot price of RWA token in USDC:

  SpotPrice(t) = (balance_U / w_U(t)) / (balance_T / w_T(t))

As w_T decreases and w_U increases over time:
  → Denominator decreases
  → Price naturally falls
  → Buyers who wait get lower prices
  → But risk others buying first and price staying elevated
  → Creates natural FOMO vs patience tension → fair price discovery
```

### LBP Rust Implementation

```rust
pub struct LbpPool {
    pub pool_id: BytesN<32>,
    pub rwa_token: Address,
    pub usdc_token: Address,

    // Starting weights (scaled 1e7, must sum to 1e7)
    pub weight_rwa_start: u128,    // e.g., 9_600_000 (96%)
    pub weight_rwa_end: u128,      // e.g., 5_000_000 (50%)

    // Timestamps
    pub start_time: u64,
    pub end_time: u64,

    // Balances
    pub balance_rwa: u128,
    pub balance_usdc: u128,

    // Config
    pub swap_fee: u128,            // Basis points (e.g., 200 = 2%)
    pub purchase_cap_per_wallet: u128,
    pub kyc_required: bool,
    pub compliance_contract: Option<Address>,
    pub min_holding_period: Option<u64>,

    // State
    pub total_usdc_raised: u128,
    pub is_active: bool,
    pub graduated: bool,
}

/// Calculate current RWA weight at time t
pub fn get_current_weight_rwa(env: &Env, pool: &LbpPool) -> u128 {
    let now = env.ledger().timestamp();
    let elapsed = now.saturating_sub(pool.start_time);
    let duration = pool.end_time - pool.start_time;

    if elapsed >= duration {
        return pool.weight_rwa_end;
    }

    let weight_delta = pool.weight_rwa_start
        .saturating_sub(pool.weight_rwa_end);

    pool.weight_rwa_start
        - (weight_delta * elapsed as u128 / duration as u128)
}

/// Calculate spot price in USDC per RWA token
pub fn get_spot_price(env: &Env, pool: &LbpPool) -> u128 {
    let w_rwa = get_current_weight_rwa(env, pool);
    let w_usdc = SCALE - w_rwa;  // SCALE = 1e7

    // SpotPrice = (balance_usdc / w_usdc) / (balance_rwa / w_rwa)
    let numerator = pool.balance_usdc * SCALE / w_usdc;
    let denominator = pool.balance_rwa * SCALE / w_rwa;
    numerator * SCALE / denominator
}

/// Calculate USDC needed to buy exact amount of RWA tokens
pub fn calculate_in_given_out(
    env: &Env,
    pool: &LbpPool,
    rwa_out: u128,
) -> u128 {
    let w_rwa = get_current_weight_rwa(env, pool);
    let w_usdc = SCALE - w_rwa;

    // Balancer invariant formula:
    // usdc_in = balance_usdc × ((balance_rwa / (balance_rwa - rwa_out))^(w_rwa/w_usdc) - 1)
    let ratio = fixed_pow(
        pool.balance_rwa * SCALE / (pool.balance_rwa - rwa_out),
        w_rwa * SCALE / w_usdc,
    );

    pool.balance_usdc * (ratio - SCALE) / SCALE
}
```

---

## 📈 Bonding Curve Engine

The Bonding Curve pool enables **continuous token issuance** with automatic price discovery. Unlike LBPs (which are time-bounded), bonding curves operate indefinitely — price rises as more tokens are purchased and falls as tokens are sold back.

### Curve Types Available

```
1. LINEAR BONDING CURVE
   P(s) = m × s + b
   Where: s = current supply, m = slope, b = base price
   Use case: Predictable, simple price growth

2. POLYNOMIAL BONDING CURVE (Recommended for RWAs)
   P(s) = a × s^n + b
   Where: n = 1 (linear), 2 (quadratic), 0.5 (square root)
   Use case: Steeper growth as demand increases

3. SIGMOID BONDING CURVE
   P(s) = P_max / (1 + e^(-k(s - s_mid)))
   Use case: S-curve adoption — slow start, rapid middle, plateau
   Best for: Assets with natural adoption ceiling (e.g., capped real estate)

4. LOGARITHMIC BONDING CURVE (Most RWA-appropriate)
   P(s) = a × ln(s + 1) + b
   Use case: Rapid early price discovery that stabilizes
   Best for: Assets with known appraised value ceiling
```

### Logarithmic Curve — Recommended for RWAs

```
Price
  │
  │                    ╭──────────────── Ceiling (appraised value)
  │               ╭────╯
  │          ╭────╯
  │     ╭────╯
  │╭────╯
  └─────────────────────────────────── Supply
  0    25%  50%  75%  100% of max supply

Why logarithmic for RWAs:
  • Price discovers rapidly in early purchases (highest uncertainty)
  • Stabilizes near appraised value as supply fills (lower uncertainty)
  • Natural ceiling prevents irrational speculation above asset value
  • Early buyers rewarded without infinite price growth
```

### Bonding Curve Rust Implementation

```rust
pub struct BondingCurvePool {
    pub pool_id: BytesN<32>,
    pub rwa_token: Address,
    pub reserve_token: Address,          // USDC

    // Curve parameters
    pub curve_type: CurveType,           // Linear | Polynomial | Sigmoid | Logarithmic
    pub curve_coefficient_a: u128,       // Primary curve coefficient
    pub curve_coefficient_b: u128,       // Base price (floor)
    pub curve_exponent_n: u128,          // For polynomial: power (scaled 1e7)
    pub max_supply: u128,                // Hard cap on issuable tokens
    pub price_ceiling: u128,             // Hard price ceiling (appraised value × 1.2)

    // State
    pub current_supply: u128,
    pub reserve_balance: u128,           // USDC held in bonding curve reserve
    pub is_active: bool,
    pub graduated: bool,

    // Config
    pub purchase_cap_per_wallet: u128,
    pub kyc_required: bool,
    pub compliance_contract: Option<Address>,
}

pub enum CurveType {
    Linear,
    Polynomial,
    Sigmoid,
    Logarithmic,
}

/// Calculate current price for logarithmic curve
pub fn get_price_logarithmic(pool: &BondingCurvePool) -> u128 {
    // P(s) = a × ln(s + 1) + b
    // Using fixed-point natural log approximation
    let ln_supply = fixed_ln(pool.current_supply + SCALE);
    let price = (pool.curve_coefficient_a * ln_supply / SCALE)
        + pool.curve_coefficient_b;

    // Enforce price ceiling
    price.min(pool.price_ceiling)
}

/// Calculate reserve (USDC) needed to purchase `token_amount` RWA tokens
/// Uses integral of price curve for exact calculation
pub fn calculate_purchase_cost(
    pool: &BondingCurvePool,
    token_amount: u128,
) -> u128 {
    // Integral of P(s) from current_supply to (current_supply + token_amount)
    // For logarithmic: ∫(a·ln(s+1) + b)ds = a·((s+1)·ln(s+1) - s) + b·s
    let s1 = pool.current_supply;
    let s2 = pool.current_supply + token_amount;

    integral_logarithmic(pool.curve_coefficient_a, pool.curve_coefficient_b, s2)
        - integral_logarithmic(pool.curve_coefficient_a, pool.curve_coefficient_b, s1)
}

/// Calculate tokens received for exact USDC input (binary search approximation)
pub fn calculate_tokens_for_usdc(
    pool: &BondingCurvePool,
    usdc_in: u128,
) -> u128 {
    // Binary search: find token_amount such that calculate_purchase_cost ≈ usdc_in
    let mut low = 0u128;
    let mut high = pool.max_supply - pool.current_supply;

    for _ in 0..64 {  // 64 iterations → sufficient precision
        let mid = (low + high) / 2;
        let cost = calculate_purchase_cost(pool, mid);
        if cost < usdc_in {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    low
}
```

---

## 🎯 Concentrated Liquidity Module

After bootstrapping completes, the Concentrated Liquidity module activates — allowing liquidity providers to add capital within specific price ranges, dramatically increasing capital efficiency and reducing slippage for traders.

### How Concentrated Liquidity Works in RWA Context

```
STANDARD AMM LIQUIDITY:
  Spread across entire price range [0, ∞)
  Capital efficiency: ~0.5% at any given price point

CONCENTRATED LIQUIDITY (Uniswap v3 model):
  LPs choose a range [P_low, P_high]
  All capital deployed within that range
  Capital efficiency: 10–100× higher within range

RWA ADVANTAGE:
  Tokenized assets have KNOWN fundamental value
  (e.g., appraised at $100/token)
  LPs can confidently provide liquidity in [$85, $115]
  → Near-zero slippage for all realistic trades
  → Far higher fee revenue for LPs
  → Better trading experience for buyers/sellers
```

### Price Range Configuration

```rust
pub struct ConcentratedLiquidityPool {
    pub pool_id: BytesN<32>,
    pub rwa_token: Address,
    pub usdc_token: Address,

    // Price range
    pub price_lower: u128,       // Lower bound (e.g., $85 per token, scaled 1e7)
    pub price_upper: u128,       // Upper bound (e.g., $115 per token, scaled 1e7)
    pub current_price: u128,     // Current spot price within range

    // Tick system (like Uniswap v3)
    pub tick_spacing: u32,       // Minimum price movement granularity
    pub fee_tier: u32,           // 500 (0.05%) | 3000 (0.3%) | 10000 (1%)

    // Liquidity
    pub total_liquidity: u128,
    pub liquidity_positions: Map<Address, LpPosition>,

    // TWAP
    pub price_accumulator: u128, // Sum of price × time
    pub last_observation_time: u64,
}

pub struct LpPosition {
    pub owner: Address,
    pub liquidity: u128,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub fee_growth_inside_rwa: u128,
    pub fee_growth_inside_usdc: u128,
    pub tokens_owed_rwa: u128,
    pub tokens_owed_usdc: u128,
}
```

---

## 📜 Smart Contract Design

### Contract Inventory

| Contract | Location | Responsibility |
|----------|----------|----------------|
| `pool_factory` | `contracts/factory/` | Creates + manages all pool types |
| `lbp_pool` | `contracts/lbp/` | Time-weighted LBP logic |
| `bonding_curve` | `contracts/bonding/` | Continuous bonding curve pools |
| `concentrated_liquidity` | `contracts/cl/` | CL position management |
| `amm_math` | `contracts/math/` | Shared curve math library |
| `fair_launch_controller` | `contracts/fairlaunch/` | Anti-whale + purchase caps |
| `graduation_engine` | `contracts/graduation/` | Bootstrapping completion + migration |
| `twap_oracle` | `contracts/oracle/` | On-chain TWAP price oracle |
| `lp_rewards` | `contracts/rewards/` | LP fee collection + distribution |
| `compliance_bridge` | `contracts/compliance/` | Optional ARCM middleware hook |

---

### Pool Factory Contract (`contracts/factory`)

```rust
/// Create a new LBP bootstrapping pool
pub fn create_lbp_pool(
    env: Env,
    issuer: Address,
    config: LbpConfig,
) -> BytesN<32>;  // Returns pool_id

/// Create a new bonding curve pool
pub fn create_bonding_pool(
    env: Env,
    issuer: Address,
    config: BondingConfig,
) -> BytesN<32>;

/// Create a concentrated liquidity pool (post-bootstrap)
pub fn create_cl_pool(
    env: Env,
    issuer: Address,
    config: ClConfig,
) -> BytesN<32>;

/// List all active pools for an RWA token
pub fn get_pools_for_asset(env: Env, rwa_token: Address) -> Vec<PoolSummary>;

/// List all pools across protocol
pub fn list_all_pools(env: Env, offset: u32, limit: u32) -> Vec<PoolSummary>;

pub struct LbpConfig {
    pub rwa_token: Address,
    pub rwa_amount: u128,            // Tokens seeded by issuer
    pub weight_rwa_start: u128,      // e.g., 96% (scaled 1e7)
    pub weight_rwa_end: u128,        // e.g., 50%
    pub start_time: u64,
    pub end_time: u64,
    pub swap_fee_bps: u32,           // Basis points
    pub purchase_cap_per_wallet: Option<u128>,
    pub kyc_required: bool,
    pub min_kyc_tier: Option<u8>,
    pub compliance_contract: Option<Address>,
    pub min_holding_period: Option<u64>,
    pub graduation_threshold: u128,  // USDC raised target for early graduation
}

pub struct BondingConfig {
    pub rwa_token: Address,
    pub curve_type: CurveType,
    pub coefficient_a: u128,
    pub coefficient_b: u128,
    pub max_supply: u128,
    pub price_ceiling: u128,
    pub purchase_cap_per_wallet: Option<u128>,
    pub kyc_required: bool,
    pub compliance_contract: Option<Address>,
}
```

---

### Fair Launch Controller (`contracts/fairlaunch`)

```rust
pub struct FairLaunchConfig {
    /// Maximum USDC any single wallet can spend in the pool
    pub max_usdc_per_wallet: Option<u128>,

    /// Maximum RWA tokens any single wallet can purchase
    pub max_tokens_per_wallet: Option<u128>,

    /// Minimum time between purchases per wallet (prevents bot sniping)
    pub cooldown_between_purchases: Option<u64>,

    /// Block first N seconds from purchase (allows price to decay initially)
    pub initial_blackout_period: Option<u64>,

    /// Require wallet to have existed for min N ledgers (anti-sybil)
    pub min_wallet_age_ledgers: Option<u32>,

    /// Max % of pool any single wallet can own
    pub max_pool_ownership_pct: Option<u128>,  // Scaled 1e7; e.g., 500_000 = 5%
}

/// Check if a purchase is allowed under fair launch rules
pub fn check_purchase_allowed(
    env: &Env,
    buyer: &Address,
    pool_id: &BytesN<32>,
    amount: u128,
    config: &FairLaunchConfig,
) -> Result<(), FairLaunchError>;

/// Record a purchase (updates per-wallet tracking)
pub fn record_purchase(
    env: &Env,
    buyer: &Address,
    pool_id: &BytesN<32>,
    tokens_purchased: u128,
    usdc_spent: u128,
);

/// Get wallet's purchase history in a pool
pub fn get_wallet_purchases(
    env: Env,
    buyer: Address,
    pool_id: BytesN<32>,
) -> WalletPurchaseRecord;

pub enum FairLaunchError {
    WalletCapExceeded,
    PoolOwnershipCapExceeded,
    CooldownNotElapsed,
    BlackoutPeriodActive,
    WalletTooNew,
    KycRequired,
    InsufficientKycTier,
    JurisdictionProhibited,
}
```

---

### Graduation Engine (`contracts/graduation`)

```rust
/// Check if a pool has met graduation criteria
pub fn check_graduation_ready(env: Env, pool_id: BytesN<32>) -> GraduationStatus;

/// Execute graduation (migrate liquidity to standard DEX)
pub fn graduate_pool(env: Env, pool_id: BytesN<32>) -> GraduationReceipt;

/// Manually trigger early graduation (issuer only, if threshold met)
pub fn trigger_early_graduation(env: Env, issuer: Address, pool_id: BytesN<32>);

pub enum GraduationCriteria {
    TimeElapsed,           // Bootstrap period ended
    FundsRaised,           // USDC raised target met
    TokensSold,            // % of supply sold threshold met
    IssuerTriggered,       // Issuer manually graduates
}

pub struct GraduationReceipt {
    pub pool_id: BytesN<32>,
    pub total_usdc_raised: u128,
    pub total_tokens_sold: u128,
    pub final_price: u128,
    pub migration_destination: Address,  // Standard DEX pool address
    pub usdc_to_issuer: u128,
    pub usdc_to_lp_pool: u128,           // Seeded into DEX for initial liquidity
    pub graduation_timestamp: u64,
}

/// Graduation USDC split:
/// 80% → Issuer (asset sale proceeds)
/// 15% → Seeded into standard DEX pool for initial liquidity
///  5% → Protocol treasury
```

---

### TWAP Oracle Contract (`contracts/oracle`)

```rust
/// Record a price observation (called on every swap)
pub fn record_observation(env: Env, pool_id: BytesN<32>, price: u128);

/// Get time-weighted average price over a period
pub fn get_twap(
    env: Env,
    pool_id: BytesN<32>,
    period_seconds: u64,
) -> u128;

/// Get latest spot price
pub fn get_spot_price(env: Env, pool_id: BytesN<32>) -> u128;

/// Get price at a specific historical timestamp (if available)
pub fn get_historical_price(
    env: Env,
    pool_id: BytesN<32>,
    timestamp: u64,
) -> Option<u128>;

/// Return full price history (paginated)
pub fn get_price_history(
    env: Env,
    pool_id: BytesN<32>,
    from_timestamp: u64,
    limit: u32,
) -> Vec<PriceObservation>;

pub struct PriceObservation {
    pub timestamp: u64,
    pub price: u128,
    pub cumulative_price: u128,  // Running sum for TWAP calculation
}
```

---

## 💱 Price Discovery Mechanism

### How LBP Achieves Fair Price Discovery

The LBP's genius is in creating **asymmetric incentives** between waiting and buying early:

```
BUYER'S DILEMMA:
  ┌─────────────────────────────────────────────────────┐
  │ BUY EARLY                  │ WAIT                   │
  │  ✅ Get more tokens         │ ✅ Lower price          │
  │  ✅ Guaranteed allocation   │ ❌ Risk others buy out  │
  │  ❌ Pay higher price        │ ❌ Smaller allocation   │
  └─────────────────────────────────────────────────────┘

This tension resolves to:
  → Buyers enter throughout the bootstrapping period
  → No single block front-running is profitable
  → Price discovery happens organically over days/weeks
  → TWAP accurately reflects genuine demand
```

### TWAP Calculation

```
Time-Weighted Average Price (TWAP) over period T:

  TWAP = Σ(price_i × Δt_i) / T

Where:
  price_i  = spot price at observation i
  Δt_i     = time elapsed since previous observation
  T        = total period length

Example (24-hour TWAP):
  12:00 PM: price = $100  (held 6 hours)
  06:00 PM: price = $95   (held 3 hours)
  09:00 PM: price = $92   (held 9 hours)

  TWAP = (100×6 + 95×3 + 92×9) / 18
       = (600 + 285 + 828) / 18
       = $95.17

TWAP is manipulation-resistant because:
  → Attacker must hold a manipulated price for extended period
  → Cost of sustained manipulation >> potential profit
```

---

## 🐋 Anti-Whale & Fair Launch Mechanics

### Multi-Layer Anti-Whale Protection

```
LAYER 1: PER-WALLET PURCHASE CAP
  Hard limit: no single wallet can purchase >X% of pool
  Configurable per pool (default: 5% of total supply)

LAYER 2: TIME-WEIGHTED PRICE DECAY (LBP)
  Buying large amounts early is EXPENSIVE
  Whales must pay the highest prices
  → Early large purchases are self-penalizing

LAYER 3: PURCHASE COOLDOWN
  Minimum N seconds between purchases per wallet
  Prevents rapid successive buys to circumvent cap

LAYER 4: INITIAL BLACKOUT PERIOD
  First 30 minutes: purchases blocked
  Allows price to naturally decay before any purchases
  Prevents bots from front-running launch block

LAYER 5: WALLET AGE REQUIREMENT
  Wallet must be ≥ N ledgers old to participate
  Prevents Sybil attacks via fresh wallets

LAYER 6: SYBIL DETECTION (Optional)
  KYC-linked: one KYC identity = one wallet cap
  Via ARCM integration: same person can't split across wallets
```

### Anti-Bot Mechanics

```rust
pub fn validate_purchase(
    env: &Env,
    buyer: &Address,
    pool: &LbpPool,
    amount: u128,
) -> Result<(), PurchaseError> {

    let now = env.ledger().timestamp();

    // 1. Blackout period check
    if now < pool.start_time + pool.initial_blackout_seconds {
        return Err(PurchaseError::BlackoutPeriodActive);
    }

    // 2. Cooldown check
    let last_purchase = get_last_purchase_time(env, buyer, &pool.pool_id);
    if now - last_purchase < pool.cooldown_between_purchases {
        return Err(PurchaseError::CooldownNotElapsed);
    }

    // 3. Per-wallet cap check
    let wallet_total = get_wallet_total_purchased(env, buyer, &pool.pool_id);
    if wallet_total + amount > pool.purchase_cap_per_wallet {
        return Err(PurchaseError::WalletCapExceeded);
    }

    // 4. Pool ownership cap
    let ownership_pct = (wallet_total + amount) * SCALE / pool.balance_rwa;
    if ownership_pct > pool.max_pool_ownership_pct {
        return Err(PurchaseError::PoolOwnershipCapExceeded);
    }

    // 5. KYC check (if required)
    if pool.kyc_required {
        validate_kyc(env, buyer, pool.min_kyc_tier, &pool.compliance_contract)?;
    }

    Ok(())
}
```

---

## 🎓 Graduation & Liquidity Migration

When a bootstrapping pool graduates, ARCM automatically migrates liquidity to ensure the RWA token enters the open market with a strong liquidity foundation.

### Graduation Criteria (Configurable per Pool)

```
TYPE A: TIME-BASED (Default)
  Pool runs for configured duration (e.g., 30 days)
  At end: regardless of amount raised → graduate

TYPE B: FUNDS-BASED
  Graduate as soon as X USDC raised
  Useful for issuers with a specific fundraising target

TYPE C: SUPPLY-BASED
  Graduate when Y% of token supply sold
  Useful for assets with fixed supply targets

TYPE D: HYBRID (Recommended)
  Graduate when EITHER:
    • Bootstrapping duration elapsed, OR
    • 80% of supply sold, OR
    • Fundraising target met
  Whichever comes first
```

### Migration Flow

```
GRADUATION TRIGGERED
       │
       ▼
┌──────────────────────┐
│ 1. CLOSE POOL        │
│ No more purchases    │
│ allowed              │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ 2. CALCULATE SPLITS  │
│ 80% USDC → Issuer   │
│ 15% USDC → DEX Seed │
│  5% USDC → Protocol │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ 3. SEED DEX POOL     │
│ Create standard AMM  │
│ pool on Stellar DEX  │
│ with 15% USDC +      │
│ remaining RWA tokens │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ 4. TRANSFER PROCEEDS │
│ 80% USDC → Issuer   │
│ wallet               │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ 5. ACTIVATE CL MODULE│
│ LPs can now add      │
│ concentrated         │
│ liquidity to deepen  │
│ the DEX pool         │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ 6. TWAP CONTINUES    │
│ Oracle keeps         │
│ accumulating price   │
│ data for downstream  │
│ DeFi integrations    │
└──────────────────────┘
```

---

## 🪪 Compliance Integration

RWA-LBP integrates natively with the **ARCM (Automated RWA Compliance Middleware)** protocol via a compliance bridge contract.

### Compliance Hook Architecture

```rust
/// Called before every purchase — optional per pool
pub fn compliance_check(
    env: &Env,
    buyer: &Address,
    pool_id: &BytesN<32>,
    amount: u128,
    compliance_contract: &Address,
) -> Result<(), ComplianceError> {

    // Cross-contract call to ARCM middleware
    let decision: ComplianceDecision = env.invoke_contract(
        compliance_contract,
        &Symbol::new(env, "check_purchase"),
        vec![
            buyer.into_val(env),
            pool_id.into_val(env),
            amount.into_val(env),
        ],
    );

    match decision {
        ComplianceDecision::Approve => Ok(()),
        ComplianceDecision::Reject(reason) => Err(ComplianceError::Rejected(reason)),
        ComplianceDecision::PendingKyc => Err(ComplianceError::KycRequired),
        _ => Err(ComplianceError::Unknown),
    }
}
```

### What Compliance Integration Enforces

| Rule | Enforced By |
|------|------------|
| KYC tier requirement (accredited investor) | ARCM KYC Oracle |
| Jurisdiction check (prohibited countries) | ARCM Jurisdiction Engine |
| OFAC/sanctions check | ARCM Sanctions Oracle |
| Holding period lock post-purchase | ARCM Enforcement Engine |
| Max holdings cap per jurisdiction | ARCM Jurisdiction Engine |

---

## 🏢 Supported RWA Asset Classes

| Asset Class | Recommended Curve | Typical Duration | Price Ceiling Basis |
|-------------|------------------|-----------------|-------------------|
| **Real Estate** | Logarithmic Bonding | 60–90 days | Independent appraisal |
| **Commodities** | LBP (96→50 weights) | 7–14 days | Spot commodity price |
| **Private Equity** | Sigmoid Bonding | 30–60 days | Latest 409A valuation |
| **Debt Instruments** | Linear Bonding | 14–30 days | Par value + yield premium |
| **Carbon Credits** | LBP (90→50 weights) | 14–21 days | Registry market price |
| **Art / Collectibles** | Polynomial Bonding | 30–60 days | Auction house appraisal |
| **Infrastructure** | Logarithmic Bonding | 90–180 days | DCF valuation |

---

## 💰 Tokenomics & Fee Structure

### Protocol Fee Model

```
ON EVERY SWAP DURING BOOTSTRAPPING:
  Swap Fee = configurable (default 2%)
  Split:
    70% → Protocol Treasury (funds development)
    20% → LP Rewards Pool (incentivizes liquidity)
    10% → Buyback & Burn (if governance token exists)

ON GRADUATION:
  5% of total USDC raised → Protocol Treasury

LP REWARD DISTRIBUTION (post-graduation):
  Distributed proportionally to liquidity providers
  Based on: (LP liquidity / total liquidity) × period_fees
  Claimed via: lp_rewards.claim(pool_id)
```

### Fee Comparison vs. Alternatives

| Method | Issuer Cost | Protocol Fee | Time to Liquidity |
|--------|------------|-------------|------------------|
| **RWA-LBP** | 0 USDC upfront | 5% of raise | 7–90 days |
| CEX Listing | $50K–$500K | Ongoing | Weeks–months |
| Standard AMM | 50% of pool value in USDC | ~0.3% per swap | Immediate but shallow |
| OTC Sales | Legal fees | None | Weeks |
| Market Maker | $10K–$100K/month | None | Days |

---

## ⭐ Stellar & Soroban Integration

### Why Stellar for RWA Liquidity Bootstrapping

- **Native DEX**: Stellar has a built-in decentralized exchange — graduated pools migrate directly, no bridges needed
- **USDC as quote currency**: Circle's native USDC on Stellar is the ideal stable pricing unit for RWAs
- **Low fees**: Complex AMM math executed on Soroban costs fractions of a cent — viable for micro-purchases
- **Fast settlement**: 3–5 second finality means TWAP oracle updates are near-real-time
- **Compliance ecosystem**: ARCM middleware lives on the same chain — zero-latency compliance checks

### Stellar-Specific Implementation Notes

```rust
// Stellar DEX integration for post-graduation liquidity
// Uses Stellar's built-in AMM (constant product) for graduated pools

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, Symbol, Vec, Map,
    token::Client as TokenClient,
};

// Cross-contract token transfers use Stellar's token interface
let rwa_token = TokenClient::new(&env, &pool.rwa_token);
let usdc_token = TokenClient::new(&env, &pool.usdc_token);

// Transfer RWA tokens from buyer escrow to buyer
rwa_token.transfer(&env.current_contract_address(), &buyer, &amount);

// Transfer USDC from buyer to pool
usdc_token.transfer_from(&env.current_contract_address(), &buyer, &env.current_contract_address(), &usdc_in);
```

---

## 🔐 Security Model

### Threat Model

| Threat | Attack Scenario | Mitigation |
|--------|----------------|-----------|
| **Price Manipulation** | Whale buys large in single block to inflate TWAP | Per-wallet caps + TWAP uses time-weighted average (not spot) |
| **Bot Front-Running** | Bots snipe first block of launch | Blackout period + Soroban's deterministic execution ordering |
| **Sandwich Attacks** | Attacker sandwiches large purchases | Slippage tolerance parameter in `buy()` call; reverts if exceeded |
| **Rug Pull** | Issuer withdraws liquidity mid-bootstrap | Issuer tokens locked in contract until graduation |
| **Fake Graduation** | Attacker triggers premature graduation | Only authorized addresses or met criteria can trigger |
| **Oracle Manipulation** | Attacker manipulates TWAP with flash trades | TWAP requires sustained price over time; flash trade = 1 block |
| **Sybil Attack** | Attacker splits whale purchase across 100 wallets | KYC-linked wallets (via ARCM) enforce one identity = one cap |
| **Math Overflow** | Integer overflow in curve calculations | Saturating arithmetic throughout; extensive fuzz testing |
| **Reentrancy** | Reentrant call during swap execution | Soroban execution model prevents reentrancy natively |
| **Weight Manipulation** | Attacker calls weight update mid-swap | Weights calculated from timestamp — not caller-supplied |

### Invariant Checks

```rust
// Enforced on every swap — pool must remain mathematically valid
pub fn assert_pool_invariant(pool: &LbpPool) {
    // Weights must sum to SCALE (1e7)
    assert!(
        pool.weight_rwa + pool.weight_usdc == SCALE,
        "Weight invariant violated"
    );

    // Balances must be non-zero
    assert!(pool.balance_rwa > 0, "RWA balance cannot be zero");
    assert!(pool.balance_usdc >= 0, "USDC balance cannot be negative");

    // Price must be within configured bounds
    let spot = get_spot_price(pool);
    assert!(spot >= pool.price_floor, "Price below floor");
    assert!(spot <= pool.price_ceiling, "Price above ceiling");
}
```

---

## 🛠️ Tech Stack

| Layer | Technology |
|-------|-----------|
| Smart Contracts | Rust + Soroban SDK |
| Blockchain | Stellar (Mainnet / Testnet) |
| AMM Math Library | Custom fixed-point Rust library |
| Quote Currency | USDC (Circle, native Stellar) |
| DEX Integration | Stellar Built-in DEX (post-graduation) |
| Compliance | ARCM Protocol (optional) |
| Frontend | Next.js 14 + TypeScript |
| Wallet Support | Freighter, xBull, Albedo |
| Stellar SDK | @stellar/stellar-sdk + soroban-client |
| Price Charts | TradingView Lightweight Charts |
| Keeper Bot | Node.js + Soroban RPC |
| Testing | Soroban test framework + 60 unit tests |
| CI/CD | GitHub Actions |
| Monitoring | Datadog + Horizon event stream |

---

## 📁 Repository Structure

```
rwa-liquidity-bootstrapping-protocol/
│
├── contracts/                                  # All Soroban smart contracts
│   │
│   ├── factory/                                # Pool factory
│   │   ├── src/
│   │   │   ├── lib.rs                          # create_lbp_pool(), create_bonding_pool()
│   │   │   ├── registry.rs                     # Pool registry + discovery
│   │   │   └── validation.rs                   # Config validation
│   │   └── Cargo.toml
│   │
│   ├── lbp/                                    # Liquidity Bootstrapping Pool
│   │   ├── src/
│   │   │   ├── lib.rs                          # buy(), get_price(), pool state
│   │   │   ├── weights.rs                      # Time-varying weight calculation
│   │   │   ├── swap.rs                         # Swap math (in-given-out, out-given-in)
│   │   │   ├── storage.rs                      # Pool storage schema
│   │   │   └── errors.rs
│   │   └── Cargo.toml
│   │
│   ├── bonding/                                # Bonding curve pools
│   │   ├── src/
│   │   │   ├── lib.rs                          # buy(), sell(), get_price()
│   │   │   ├── curves/
│   │   │   │   ├── linear.rs
│   │   │   │   ├── polynomial.rs
│   │   │   │   ├── sigmoid.rs
│   │   │   │   └── logarithmic.rs              # Primary RWA curve
│   │   │   ├── integral.rs                     # Curve integrals for exact pricing
│   │   │   └── storage.rs
│   │   └── Cargo.toml
│   │
│   ├── cl/                                     # Concentrated liquidity
│   │   ├── src/
│   │   │   ├── lib.rs                          # add_liquidity(), remove(), swap()
│   │   │   ├── ticks.rs                        # Tick bitmap + management
│   │   │   ├── positions.rs                    # LP position tracking
│   │   │   ├── fees.rs                         # Fee growth tracking
│   │   │   └── math.rs                         # SqrtPrice math
│   │   └── Cargo.toml
│   │
│   ├── math/                                   # Shared math library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── fixed_point.rs                  # Fixed-point arithmetic (1e7 scaling)
│   │   │   ├── pow.rs                          # Fixed-point power function
│   │   │   ├── ln.rs                           # Fixed-point natural log
│   │   │   └── sqrt.rs                         # Fixed-point square root
│   │   └── Cargo.toml
│   │
│   ├── fairlaunch/                             # Anti-whale + fair launch
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── caps.rs                         # Per-wallet + pool caps
│   │   │   ├── cooldown.rs                     # Purchase cooldown logic
│   │   │   ├── blackout.rs                     # Initial blackout period
│   │   │   └── sybil.rs                        # Wallet age + KYC sybil guard
│   │   └── Cargo.toml
│   │
│   ├── graduation/                             # Graduation + migration
│   │   ├── src/
│   │   │   ├── lib.rs                          # check_ready(), graduate()
│   │   │   ├── criteria.rs                     # Graduation condition evaluation
│   │   │   ├── migration.rs                    # DEX pool seeding
│   │   │   └── splits.rs                       # USDC proceeds allocation
│   │   └── Cargo.toml
│   │
│   ├── oracle/                                 # TWAP price oracle
│   │   ├── src/
│   │   │   ├── lib.rs                          # record_observation(), get_twap()
│   │   │   ├── accumulator.rs                  # Cumulative price tracking
│   │   │   └── history.rs                      # Historical price storage
│   │   └── Cargo.toml
│   │
│   ├── rewards/                                # LP fee distribution
│   │   ├── src/
│   │   │   ├── lib.rs                          # claim(), distribute()
│   │   │   ├── accounting.rs                   # Fee growth per liquidity unit
│   │   │   └── vesting.rs                      # Optional LP reward vesting
│   │   └── Cargo.toml
│   │
│   └── compliance_bridge/                      # Optional ARCM integration
│       ├── src/
│       │   ├── lib.rs                          # compliance_check()
│       │   └── arcm_client.rs                  # ARCM contract caller
│       └── Cargo.toml
│
├── keeper/                                     # Automation bot
│   ├── src/
│   │   ├── index.ts
│   │   ├── weight_updater.ts                   # Updates pool weights periodically
│   │   ├── graduation_monitor.ts               # Watches graduation criteria
│   │   ├── oracle_pinger.ts                    # Triggers TWAP observations
│   │   └── lp_reward_distributor.ts
│   └── package.json
│
├── frontend/
│   ├── src/
│   │   ├── app/
│   │   │   ├── page.tsx                        # Pool discovery + launchpad
│   │   │   ├── pool/[id]/page.tsx              # Individual pool page
│   │   │   ├── launch/page.tsx                 # Issuer pool creation wizard
│   │   │   └── portfolio/page.tsx              # User purchases + vesting
│   │   ├── components/
│   │   │   ├── PriceChart.tsx                  # TradingView chart (TWAP + spot)
│   │   │   ├── WeightChart.tsx                 # LBP weight decay visualization
│   │   │   ├── BuyForm.tsx                     # Purchase widget
│   │   │   ├── PoolCard.tsx                    # Pool summary card
│   │   │   ├── BootstrappingProgress.tsx       # Visual progress tracker
│   │   │   └── GraduationCountdown.tsx         # Time to graduation
│   │   └── lib/
│   │       ├── curve_math.ts                   # Client-side price simulation
│   │       ├── pool_queries.ts                 # Contract state queries
│   │       └── wallet.ts                       # Freighter integration
│   └── package.json
│
├── scripts/
│   ├── deploy_all.sh                           # Deploy all contracts to Stellar
│   ├── initialize.sh                           # Initialize factory + templates
│   ├── create_test_pool.sh                     # Spin up demo LBP/bonding pool
│   └── optimize_all.sh                         # Optimize Wasm bytecode
│
├── docs/
│   ├── architecture.md
│   ├── curve-math.md                           # Full mathematical derivations
│   ├── lbp-guide.md                            # Issuer LBP configuration guide
│   ├── bonding-curve-guide.md
│   ├── anti-whale-guide.md
│   └── graduation-guide.md
│
├── simulations/                                # Off-chain Monte Carlo simulations
│   ├── lbp_simulation.py                       # Python: simulate LBP over time
│   ├── bonding_curve_sim.py
│   └── plots/                                  # Pre-generated curve visualizations
│
├── Cargo.toml
└── README.md
```

---

## 🚀 Getting Started

### Prerequisites

```bash
# 1. Rust + Wasm target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# 2. Soroban CLI
cargo install --locked soroban-cli --features opt

# 3. Node.js 18+
nvm install 18 && nvm use 18

# 4. Python 3.10+ (for simulations only)
pip install numpy matplotlib scipy
```

### Clone & Build

```bash
git clone https://github.com/your-org/rwa-liquidity-bootstrapping-protocol.git
cd rwa-liquidity-bootstrapping-protocol

# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Optimize Wasm
./scripts/optimize_all.sh

# Run all tests (60 unit tests across 7 contract crates)
cargo test --workspace
```

### Environment Configuration

```env
# .env
STELLAR_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Contract IDs (after deployment)
POOL_FACTORY_CONTRACT_ID=
LBP_TEMPLATE_CONTRACT_ID=
BONDING_TEMPLATE_CONTRACT_ID=
CL_CONTRACT_ID=
GRADUATION_ENGINE_CONTRACT_ID=
TWAP_ORACLE_CONTRACT_ID=
LP_REWARDS_CONTRACT_ID=
COMPLIANCE_BRIDGE_CONTRACT_ID=

# USDC on Stellar Testnet
USDC_CONTRACT_ID=CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA

# Optional ARCM integration
ARCM_GATEWAY_CONTRACT_ID=   # Leave empty to disable compliance hooks

# Admin
ADMIN_SECRET_KEY=S...
KEEPER_SECRET_KEY=S...
```

---

## 🌐 Contract Deployment

```bash
# 1. Deploy full contract suite
./scripts/deploy_all.sh

# 2. Initialize with contract addresses
./scripts/initialize.sh

# 3. Create a demo LBP pool on testnet
./scripts/create_test_pool.sh \
  --rwa-token $MY_RWA_TOKEN \
  --amount 1000000 \
  --weight-start 96 \
  --weight-end 50 \
  --duration-days 7 \
  --purchase-cap 10000
```

### Deploy & Create LBP Pool Manually

```bash
# Deploy pool factory
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/pool_factory.wasm \
  --source admin \
  --network testnet

# Create LBP pool
soroban contract invoke \
  --id $POOL_FACTORY_CONTRACT_ID \
  --source issuer-key \
  --network testnet \
  -- \
  create_lbp_pool \
  --issuer $ISSUER_ADDRESS \
  --rwa_token $RWA_TOKEN_ADDRESS \
  --rwa_amount 1000000_0000000 \
  --weight_rwa_start 9600000 \
  --weight_rwa_end 5000000 \
  --start_time $(date +%s) \
  --end_time $(($(date +%s) + 2592000)) \
  --swap_fee_bps 200 \
  --purchase_cap_per_wallet 50000_0000000 \
  --kyc_required false
```

---

## 📡 API Reference

### Pool Factory

```rust
// Create pools
pub fn create_lbp_pool(env: Env, issuer: Address, config: LbpConfig) -> BytesN<32>
pub fn create_bonding_pool(env: Env, issuer: Address, config: BondingConfig) -> BytesN<32>
pub fn create_cl_pool(env: Env, issuer: Address, config: ClConfig) -> BytesN<32>

// Discovery
pub fn get_pools_for_asset(env: Env, rwa_token: Address) -> Vec<PoolSummary>
pub fn list_all_pools(env: Env, offset: u32, limit: u32) -> Vec<PoolSummary>
pub fn get_pool_config(env: Env, pool_id: BytesN<32>) -> PoolConfig
```

### LBP Pool

```rust
// Trading
pub fn buy(env: Env, buyer: Address, usdc_in: u128, min_tokens_out: u128) -> u128
pub fn get_spot_price(env: Env) -> u128
pub fn get_price_at_time(env: Env, timestamp: u64) -> u128
pub fn simulate_buy(env: Env, usdc_in: u128) -> SwapSimulation

// State
pub fn get_pool_state(env: Env) -> LbpPool
pub fn get_current_weights(env: Env) -> (u128, u128)
pub fn time_remaining(env: Env) -> u64
pub fn get_wallet_purchases(env: Env, wallet: Address) -> WalletPurchaseRecord
```

### Bonding Curve Pool

```rust
// Trading
pub fn buy(env: Env, buyer: Address, usdc_in: u128, min_tokens_out: u128) -> u128
pub fn sell(env: Env, seller: Address, tokens_in: u128, min_usdc_out: u128) -> u128
pub fn get_current_price(env: Env) -> u128
pub fn get_price_for_supply(env: Env, supply: u128) -> u128
pub fn simulate_buy(env: Env, usdc_in: u128) -> SwapSimulation
pub fn simulate_sell(env: Env, tokens_in: u128) -> SwapSimulation
```

### TWAP Oracle

```rust
pub fn get_twap(env: Env, pool_id: BytesN<32>, period_seconds: u64) -> u128
pub fn get_spot_price(env: Env, pool_id: BytesN<32>) -> u128
pub fn get_price_history(env: Env, pool_id: BytesN<32>, from: u64, limit: u32) -> Vec<PriceObservation>
```

### Graduation Engine

```rust
pub fn check_graduation_ready(env: Env, pool_id: BytesN<32>) -> GraduationStatus
pub fn graduate_pool(env: Env, pool_id: BytesN<32>) -> GraduationReceipt
pub fn get_graduation_criteria(env: Env, pool_id: BytesN<32>) -> GraduationCriteria
```

---

## 🔌 Issuer Integration Guide

### Step 1: Simulate Your Curve First

```python
# simulations/lbp_simulation.py
from lbp_sim import simulate_lbp

results = simulate_lbp(
    rwa_amount=1_000_000,
    weight_start=0.96,
    weight_end=0.50,
    duration_days=30,
    expected_buyers=500,
    avg_purchase_usdc=2000,
)

print(f"Projected raise: ${results.total_usdc_raised:,.0f}")
print(f"Price at day 0: ${results.prices[0]:.2f}")
print(f"Price at day 30: ${results.prices[-1]:.2f}")
print(f"Estimated fair value: ${results.equilibrium_price:.2f}")
```

### Step 2: Choose Your Curve Type

```
DECISION TREE:

Do you have a fixed token supply?
  YES → Do you need time-bounded price discovery?
            YES → Use LBP (96→50 weight decay)
            NO  → Use Bonding Curve (Logarithmic)
  NO  → Use Bonding Curve (Sigmoid or Polynomial)

Is your asset yield-bearing with known appraised value?
  YES → Logarithmic Bonding Curve (stabilizes near appraised value)
  NO  → LBP (let market discover value from scratch)

Do you need KYC / accredited investor gating?
  YES → Enable compliance_contract in config → connects to ARCM
  NO  → Leave compliance_contract as None
```

### Step 3: Configure and Deploy

```typescript
import { PoolFactory } from "./lib/pool-factory";

const factory = new PoolFactory(POOL_FACTORY_CONTRACT_ID);

const poolId = await factory.createLbpPool(issuerKeypair, {
  rwaToken: "G...",                      // Your RWA token contract
  rwaAmount: BigInt("1000000_0000000"),  // 1M tokens (7 decimals)
  weightRwaStart: 9_600_000,             // 96%
  weightRwaEnd: 5_000_000,               // 50%
  startTime: Math.floor(Date.now() / 1000) + 3600,  // Start in 1 hour
  endTime: Math.floor(Date.now() / 1000) + 2592000, // 30 days
  swapFeeBps: 200,                       // 2% swap fee
  purchaseCapPerWallet: BigInt("50000_0000000"),     // Max 50K USDC per wallet
  kycRequired: true,
  minKycTier: 2,                         // Accredited investors only
  complianceContract: ARCM_GATEWAY_CONTRACT_ID,
  minHoldingPeriod: 7776000,             // 90 days in seconds
  graduationThreshold: BigInt("500000_0000000"), // Graduate at $500K raised
});

console.log(`Pool created: ${poolId}`);
```

### Step 4: Monitor Your Pool

```typescript
// Real-time pool monitoring
const pool = await factory.getPoolState(poolId);

console.log(`Current price: $${pool.spotPrice / 1e7}`);
console.log(`USDC raised: $${pool.totalUsdcRaised / 1e7}`);
console.log(`Tokens sold: ${pool.tokensSold / 1e7}`);
console.log(`Time remaining: ${pool.timeRemaining / 3600} hours`);
console.log(`Graduation ready: ${pool.graduationReady}`);
```

---

## 🧪 Testing

### Contract Unit Tests (60 tests across all contracts)

Tests live alongside each contract in `contracts/*/src/test.rs`. Run them all at once:

```bash
# Run all tests across the entire workspace
cargo test --workspace
```

Run tests for a specific contract:

```bash
# Math library (20 tests)
cargo test -p amm_math -- --nocapture

# Pool factory (7 tests)
cargo test -p contract-factory

# LBP pool (7 tests)
cargo test -p contract-lbp

# Bonding curve pool (8 tests)
cargo test -p contract-bonding

# Fair launch controller (6 tests)
cargo test -p contract-fairlaunch

# Graduation engine (6 tests)
cargo test -p contract-graduation

# TWAP oracle (6 tests)
cargo test -p contract-oracle
```

### Key Test Scenarios

| # | Scenario | Contract | Validates |
|---|----------|----------|----------|
| 1 | Fixed-point multiply, divide, pow, ln | `math` | Arithmetic correctness |
| 2 | Sigmoid asymptotes & midpoint | `math` | Curve function accuracy |
| 3 | Integral logarithmic pricing | `math` | Bonding curve integral |
| 4 | Weight decay at t=0, 7, 15, 22, 30 | `lbp` | LBP weight formula |
| 5 | Spot price decreases over time | `lbp` | LBP price monotonicity |
| 6 | Buy with slippage protection | `lbp` | Swap execution |
| 7 | Graduated pool rejects buys | `lbp` | Pool state enforcement |
| 8 | Linear, logarithmic, sigmoid pricing | `bonding` | Curve price formulas |
| 9 | Max supply enforcement | `bonding` | Supply cap |
| 10 | Sell via bonding curve | `bonding` | Reverse swap math |
| 11 | Purchase cap per wallet | `fairlaunch` | Anti-whale enforcement |
| 12 | Cooldown between purchases | `fairlaunch` | Rate limiting |
| 13 | Pool ownership cap | `fairlaunch` | Max allocation |
| 14 | Blackout period rejection | `fairlaunch` | Initial time lock |
| 15 | Time-based graduation | `graduation` | Criteria evaluation |
| 16 | Early graduation by issuer | `graduation` | Admin override |
| 17 | TWAP record and query | `oracle` | Oracle accumulation |
| 18 | TWAP manipulation resistance | `oracle` | Time-weighted smoothing |

---

## 📐 Mathematical Appendix

### A. Balancer-Style Invariant (LBP)

```
Value function: V = Π(balance_i ^ weight_i)

For RWA-LBP with two tokens (T = RWA, U = USDC):
  V = balance_T^w_T × balance_U^w_U

This invariant is preserved across all swaps:
  V_before_swap = V_after_swap

Spot price:
  P(T→U) = (balance_U / w_U) / (balance_T / w_T)

Out-given-in (selling USDC for RWA):
  tokens_out = balance_T × (1 - (balance_U / (balance_U + usdc_in))^(w_U/w_T))

In-given-out (buying exact RWA amount with USDC):
  usdc_in = balance_U × ((balance_T / (balance_T - tokens_out))^(w_T/w_U) - 1)
```

### B. Logarithmic Bonding Curve Integral

```
Price function:    P(s) = a·ln(s + 1) + b
Integral:          ∫P(s)ds = a·((s+1)·ln(s+1) - s) + b·s + C

Cost to buy from supply s1 to s2:
  Cost = ∫[s1→s2] P(s)ds
       = [a·((s+1)·ln(s+1) - s) + b·s] evaluated from s1 to s2
       = a·((s2+1)·ln(s2+1) - s2) + b·s2
         - a·((s1+1)·ln(s1+1) - s1) - b·s1

This gives EXACT cost without binary search approximation.
```

### C. TWAP Calculation

```
Cumulative price at time t:
  CP(t) = CP(t-1) + SpotPrice(t) × (t - t_prev)

TWAP over period [t_start, t_end]:
  TWAP = (CP(t_end) - CP(t_start)) / (t_end - t_start)

All arithmetic in fixed-point 1e7 scaling.
Overflow protection: CP stored as u256 (via multi-word arithmetic).
```

---

## 🗺️ Roadmap

### Phase 1 — Core AMM (100% Complete)
- [x] Protocol design + curve selection
- [x] LBP pool contract (Balancer-style)
- [x] Logarithmic bonding curve contract
- [x] AMM math library (fixed-point)
- [x] TWAP oracle contract
- [x] Comprehensive test suite (60 tests)
- [x] Pool factory contract
- [x] Deployment scripts skeleton

### Phase 2 — Fair Launch + Compliance (70% Complete)
- [x] Fair Launch Controller (anti-whale, blackout, cooldown)
- [x] Compliance bridge (ARCM integration, stub)
- [x] Graduation engine + DEX migration
- [ ] LP rewards distributor (full proportional distribution)
- [ ] Frontend launchpad (Alpha)

### Phase 3 — Advanced Curves (20% Complete)
- [x] Concentrated liquidity module (position management)
- [ ] Concentrated liquidity swap execution
- [ ] Sigmoid + polynomial bonding curves (full)
- [ ] Multi-asset pools (3-token LBP)
- [ ] External LP incentives (gauge system)
- [ ] Issuer dashboard v2

### Phase 4 — Mainnet 
- [ ] Security audit
- [ ] Mainnet deployment
- [ ] Real estate token launchpad pilots
- [ ] Commodity token launchpad pilots
- [ ] Bug bounty

### Phase 5 — Ecosystem 
- [ ] RWA-LBP SDK for third-party launchpads
- [ ] Cross-chain price oracle bridge
- [ ] Governance token + DAO
- [ ] Insurance integration (IL protection for LPs)
- [ ] Institutional API for large issuers

---

## 🤝 Contributing

Contributions are welcome! Please read our full guide in [CONTRIBUTING.md](./CONTRIBUTING.md).

This project follows a [Code of Conduct](./CODE_OF_CONDUCT.md) — by participating, you agree to uphold it.

### Quick Start

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/rwa-liquidity-bootstrapping-protocol.git

# Feature branch
git checkout -b feature/your-feature

# Test before PR
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# PR against main
```

### Priority Contribution Areas

- 🔢 **Curve math** — Implement polynomial and sigmoid curves in `contracts/bonding/src/curves/`
- 🧪 **Fuzz testing** — Expand fuzz test coverage for edge cases in curve math
- 📊 **Simulations** — Add Python simulations for new curve types
- 🌍 **Issuer guides** — Write configuration guides for specific asset classes
- 🔌 **Integrations** — Build adapters for other Stellar DeFi protocols

Browse the [issue tracker](https://github.com/your-org/rwa-liquidity-bootstrapping-protocol/issues) for open work — look for `good first issue` labels to get started.

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](./LICENSE) file for details.

---

## 🙏 Acknowledgements

- [Balancer Finance](https://balancer.fi) — Liquidity Bootstrapping Pool design inspiration
- [Uniswap](https://uniswap.org) — Concentrated liquidity model reference (v3)
- [Stellar Development Foundation](https://stellar.org) — Soroban + native DEX infrastructure
- [Circle](https://circle.com) — Native USDC on Stellar
- The broader RWA tokenization and DeFi research community

---

<div align="center">

**Built with 🌊 for the tokenized asset economy on Stellar**

[Website](#) · [Documentation](#) · [Discord](#) · [Twitter](#) · [Simulations](#)

*Deep liquidity for every real-world asset. From day one.*

</div>
