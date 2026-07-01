mod math;
pub use math::*;

use soroban_sdk::{contracttype, Address, BytesN, Map};

pub const SCALE: u128 = 10_000_000;

#[contracttype]
#[derive(Clone, Debug)]
pub struct LbpPool {
    pub pool_id: BytesN<32>,
    pub rwa_token: Address,
    pub usdc_token: Address,
    pub weight_rwa_start: u128,
    pub weight_rwa_end: u128,
    pub start_time: u64,
    pub end_time: u64,
    pub balance_rwa: u128,
    pub balance_usdc: u128,
    pub swap_fee: u128,
    pub purchase_cap_per_wallet: u128,
    pub kyc_required: bool,
    pub compliance_contract: Address,
    pub min_holding_period: Option<u64>,
    pub total_usdc_raised: u128,
    pub is_active: bool,
    pub graduated: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BondingCurvePool {
    pub pool_id: BytesN<32>,
    pub rwa_token: Address,
    pub reserve_token: Address,
    pub curve_type: CurveType,
    pub curve_coefficient_a: u128,
    pub curve_coefficient_b: u128,
    pub curve_exponent_n: u128,
    pub max_supply: u128,
    pub price_ceiling: u128,
    pub current_supply: u128,
    pub reserve_balance: u128,
    pub is_active: bool,
    pub graduated: bool,
    pub purchase_cap_per_wallet: u128,
    pub kyc_required: bool,
    pub compliance_contract: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct ConcentratedLiquidityPool {
    pub pool_id: BytesN<32>,
    pub rwa_token: Address,
    pub usdc_token: Address,
    pub price_lower: u128,
    pub price_upper: u128,
    pub current_price: u128,
    pub tick_spacing: u32,
    pub fee_tier: u32,
    pub total_liquidity: u128,
    pub liquidity_positions: Map<Address, LpPosition>,
    pub price_accumulator: u128,
    pub last_observation_time: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
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

#[contracttype]
#[derive(Clone, Debug)]
pub struct FairLaunchConfig {
    pub max_usdc_per_wallet: Option<u128>,
    pub max_tokens_per_wallet: Option<u128>,
    pub cooldown_between_purchases: Option<u64>,
    pub initial_blackout_period: Option<u64>,
    pub min_wallet_age_ledgers: Option<u32>,
    pub max_pool_ownership_pct: Option<u128>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct WalletPurchaseRecord {
    pub wallet: Address,
    pub pool_id: BytesN<32>,
    pub total_tokens_purchased: u128,
    pub total_usdc_spent: u128,
    pub last_purchase_time: u64,
    pub purchase_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GraduationReceipt {
    pub pool_id: BytesN<32>,
    pub total_usdc_raised: u128,
    pub total_tokens_sold: u128,
    pub final_price: u128,
    pub migration_destination: Address,
    pub usdc_to_issuer: u128,
    pub usdc_to_lp_pool: u128,
    pub graduation_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PriceObservation {
    pub timestamp: u64,
    pub price: u128,
    pub cumulative_price: u128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolSummary {
    pub pool_id: BytesN<32>,
    pub pool_type: PoolType,
    pub rwa_token: Address,
    pub is_active: bool,
    pub graduated: bool,
    pub total_usdc_raised: u128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LbpConfig {
    pub rwa_token: Address,
    pub rwa_amount: u128,
    pub weight_rwa_start: u128,
    pub weight_rwa_end: u128,
    pub start_time: u64,
    pub end_time: u64,
    pub swap_fee_bps: u32,
    pub purchase_cap_per_wallet: Option<u128>,
    pub kyc_required: bool,
    pub min_kyc_tier: Option<u32>,
    pub compliance_contract: Address,
    pub min_holding_period: Option<u64>,
    pub graduation_threshold: u128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BondingConfig {
    pub rwa_token: Address,
    pub curve_type: CurveType,
    pub coefficient_a: u128,
    pub coefficient_b: u128,
    pub max_supply: u128,
    pub price_ceiling: u128,
    pub purchase_cap_per_wallet: Option<u128>,
    pub kyc_required: bool,
    pub compliance_contract: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ClConfig {
    pub rwa_token: Address,
    pub usdc_token: Address,
    pub price_lower: u128,
    pub price_upper: u128,
    pub tick_spacing: u32,
    pub fee_tier: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum CurveType {
    Linear,
    Polynomial,
    Sigmoid,
    Logarithmic,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum GraduationCriteria {
    TimeElapsed,
    FundsRaised,
    TokensSold,
    IssuerTriggered,
}

#[contracttype]
#[derive(Clone, Debug)]
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

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum PoolType {
    Lbp,
    BondingCurve,
    ConcentratedLiquidity,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum ComplianceDecision {
    Approve,
    Reject(BytesN<32>),
    PendingKyc,
}

#[cfg(test)]
mod test;
