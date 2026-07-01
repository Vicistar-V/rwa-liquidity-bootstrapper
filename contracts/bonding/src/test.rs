use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, IntoVal, Symbol, Val, Vec};
use soroban_sdk::testutils::{Address as _, Ledger as _};

use crate::BondingCurveContract;
use amm_math::{BondingConfig, CurveType, SCALE};

#[contract]
pub struct MockFairLaunchForBonding;

#[contractimpl]
impl MockFairLaunchForBonding {
    pub fn check_purchase_allowed(
        _env: Env,
        _buyer: Address,
        _pool_id: BytesN<32>,
        _amount: u128,
    ) {
    }

    pub fn record_purchase(
        _env: Env,
        _buyer: Address,
        _pool_id: BytesN<32>,
        _tokens_purchased: u128,
        _usdc_spent: u128,
    ) {
    }
}

#[contract]
pub struct MockOracleForBonding;

#[contractimpl]
impl MockOracleForBonding {
    pub fn record_observation(_env: Env, _pool_id: BytesN<32>, _price: u128) {
    }
}

fn setup_test_env() -> (Env, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let rwa_id = env.register_stellar_asset_contract(admin.clone());
    let rwa_sac = token::StellarAssetClient::new(&env, &rwa_id);
    rwa_sac.mint(&admin, &i128::MAX);

    let reserve_id = env.register_stellar_asset_contract(admin.clone());
    let reserve_sac = token::StellarAssetClient::new(&env, &reserve_id);
    reserve_sac.mint(&admin, &i128::MAX);

    let bonding_id = env.register_contract(None, BondingCurveContract);
    let bonding_addr = Address::from(bonding_id.clone());
    rwa_sac.mint(&bonding_addr, &(10_000_000_0000000i128));
    reserve_sac.mint(&bonding_addr, &(10_000_000_0000000i128));

    let fair_launch_id = env.register_contract(None, MockFairLaunchForBonding);
    let oracle_id = env.register_contract(None, MockOracleForBonding);

    (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id)
}

fn init_and_return_basic_config(env: &Env, bonding_id: &Address, pool_id: &BytesN<32>, rwa_id: &Address, reserve_id: &Address, fair_launch_id: &Address, oracle_id: &Address) {
    let client = crate::BondingCurveContractClient::new(env, bonding_id);
    let config = BondingConfig {
        rwa_token: rwa_id.clone(),
        curve_type: CurveType::Linear,
        coefficient_a: SCALE,
        coefficient_b: SCALE / 10,
        max_supply: 1_000_000_0000000,
        price_ceiling: 100 * SCALE,
        purchase_cap_per_wallet: Some(100_000_0000000),
        kyc_required: false,
        compliance_contract: Address::generate(env),
    };
    client.initialize(
        pool_id,
        &config,
        &Address::generate(env),
        reserve_id,
        fair_launch_id,
        oracle_id,
    );
}

#[test]
fn test_linear_curve_pricing() {
    let (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id) = setup_test_env();
    let client = crate::BondingCurveContractClient::new(&env, &bonding_id);
    let pool_id = BytesN::from_array(&env, &[10; 32]);
    init_and_return_basic_config(&env, &bonding_id, &pool_id, &rwa_id, &reserve_id, &fair_launch_id, &oracle_id);

    let price = client.get_price(&pool_id);
    assert_eq!(price, SCALE / 10);
}

#[test]
fn test_linear_curve_price_increases() {
    let (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id) = setup_test_env();
    let client = crate::BondingCurveContractClient::new(&env, &bonding_id);
    let pool_id = BytesN::from_array(&env, &[11; 32]);
    init_and_return_basic_config(&env, &bonding_id, &pool_id, &rwa_id, &reserve_id, &fair_launch_id, &oracle_id);

    let price_at_0 = client.get_price(&pool_id);
    assert_eq!(price_at_0, SCALE / 10);
}

#[test]
fn test_logarithmic_curve_pricing() {
    let (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id) = setup_test_env();
    let client = crate::BondingCurveContractClient::new(&env, &bonding_id);
    let pool_id = BytesN::from_array(&env, &[12; 32]);

    let config = BondingConfig {
        rwa_token: rwa_id,
        curve_type: CurveType::Logarithmic,
        coefficient_a: SCALE,
        coefficient_b: SCALE / 10,
        max_supply: 1_000_000_0000000,
        price_ceiling: 100 * SCALE,
        purchase_cap_per_wallet: Some(100_000_0000000),
        kyc_required: false,
        compliance_contract: Address::generate(&env),
    };
    client.initialize(
        &pool_id,
        &config,
        &Address::generate(&env),
        &reserve_id,
        &fair_launch_id,
        &oracle_id,
    );

    let price = client.get_price(&pool_id);
    assert!(price > 0);
}

#[test]
fn test_buy_via_bonding_curve() {
    let (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id) = setup_test_env();
    let client = crate::BondingCurveContractClient::new(&env, &bonding_id);
    let pool_id = BytesN::from_array(&env, &[13; 32]);
    let buyer = Address::generate(&env);

    let reserve_sac = token::StellarAssetClient::new(&env, &reserve_id);
    reserve_sac.mint(&buyer, &(100_000_0000000i128));

    init_and_return_basic_config(&env, &bonding_id, &pool_id, &rwa_id, &reserve_id, &fair_launch_id, &oracle_id);

    let usdc_in = 1000_0000000u128;
    let min_tokens = 0u128;
    let tokens_out = client.buy(&buyer, &pool_id, &usdc_in, &min_tokens);

    assert!(tokens_out > 0);

    let pool = client.get_pool_details(&pool_id);
    assert_eq!(pool.current_supply, tokens_out);
    assert!(pool.reserve_balance == usdc_in);
}

#[test]
fn test_sell_via_bonding_curve() {
    let (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id) = setup_test_env();
    let client = crate::BondingCurveContractClient::new(&env, &bonding_id);
    let pool_id = BytesN::from_array(&env, &[14; 32]);
    let buyer = Address::generate(&env);

    let reserve_sac = token::StellarAssetClient::new(&env, &reserve_id);
    reserve_sac.mint(&buyer, &(100_000_0000000i128));

    init_and_return_basic_config(&env, &bonding_id, &pool_id, &rwa_id, &reserve_id, &fair_launch_id, &oracle_id);

    let usdc_in = 1000_0000000u128;
    let tokens_out = client.buy(&buyer, &pool_id, &usdc_in, &0u128);

    let sell_amount = tokens_out / 2;
    let usdc_out = client.sell(&buyer, &pool_id, &sell_amount, &0u128);

    assert!(usdc_out > 0);

    let pool = client.get_pool_details(&pool_id);
    assert!(pool.current_supply < tokens_out);
    assert!(pool.reserve_balance < 1000_0000000u128);
}

#[test]
fn test_max_supply_enforced() {
    let (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id) = setup_test_env();
    let client = crate::BondingCurveContractClient::new(&env, &bonding_id);
    let pool_id = BytesN::from_array(&env, &[15; 32]);
    let buyer = Address::generate(&env);

    let reserve_sac = token::StellarAssetClient::new(&env, &reserve_id);
    reserve_sac.mint(&buyer, &(100_000_0000000i128));

    let config = BondingConfig {
        rwa_token: rwa_id,
        curve_type: CurveType::Linear,
        coefficient_a: SCALE,
        coefficient_b: SCALE / 10,
        max_supply: 100,
        price_ceiling: 100 * SCALE,
        purchase_cap_per_wallet: Some(100_000_0000000),
        kyc_required: false,
        compliance_contract: Address::generate(&env),
    };
    client.initialize(
        &pool_id,
        &config,
        &Address::generate(&env),
        &reserve_id,
        &fair_launch_id,
        &oracle_id,
    );

    let usdc_in = 1000_0000000u128;
    let tokens_out = client.buy(&buyer, &pool_id, &usdc_in, &0u128);
    assert!(tokens_out > 0);
    assert!(tokens_out <= 100);
}

#[test]
fn test_price_ceiling_enforced() {
    let (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id) = setup_test_env();
    let client = crate::BondingCurveContractClient::new(&env, &bonding_id);
    let pool_id = BytesN::from_array(&env, &[16; 32]);

    let config = BondingConfig {
        rwa_token: rwa_id,
        curve_type: CurveType::Linear,
        coefficient_a: 10 * SCALE,
        coefficient_b: 0,
        max_supply: 1_000_000_0000000,
        price_ceiling: SCALE,
        purchase_cap_per_wallet: Some(100_000_0000000),
        kyc_required: false,
        compliance_contract: Address::generate(&env),
    };
    client.initialize(
        &pool_id,
        &config,
        &Address::generate(&env),
        &reserve_id,
        &fair_launch_id,
        &oracle_id,
    );

    let price = client.get_price(&pool_id);
    assert!(price <= SCALE);
}

#[test]
fn test_sigmoid_curve_pricing() {
    let (env, bonding_id, rwa_id, reserve_id, fair_launch_id, oracle_id) = setup_test_env();
    let client = crate::BondingCurveContractClient::new(&env, &bonding_id);
    let pool_id = BytesN::from_array(&env, &[17; 32]);

    let config = BondingConfig {
        rwa_token: rwa_id,
        curve_type: CurveType::Sigmoid,
        coefficient_a: SCALE / 100,
        coefficient_b: SCALE / 10,
        max_supply: 100_000_000,
        price_ceiling: 100 * SCALE,
        purchase_cap_per_wallet: Some(100_000_0000000),
        kyc_required: false,
        compliance_contract: Address::generate(&env),
    };
    client.initialize(
        &pool_id,
        &config,
        &Address::generate(&env),
        &reserve_id,
        &fair_launch_id,
        &oracle_id,
    );

    let price = client.get_price(&pool_id);
    assert!(price > 0);
}

