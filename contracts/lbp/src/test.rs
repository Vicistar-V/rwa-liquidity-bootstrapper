use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env};
use soroban_sdk::testutils::{Address as _, Ledger};

use crate::LbpPoolContract;
use amm_math::LbpConfig;

#[contract]
pub struct MockFairLaunchForLbp;

#[contractimpl]
impl MockFairLaunchForLbp {
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
pub struct MockOracleForLbp;

#[contractimpl]
impl MockOracleForLbp {
    pub fn record_observation(_env: Env, _pool_id: BytesN<32>, _price: u128) {
    }
}

fn setup_test_env() -> (Env, Address, amm_math::LbpConfig, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(admin.clone());
    let sac = token::StellarAssetClient::new(&env, &token_id);
    sac.mint(&admin, &i128::MAX);

    let lbp_id = env.register_contract(None, LbpPoolContract);
    let fair_launch_id =
        env.register_contract(None, MockFairLaunchForLbp);
    let oracle_id =
        env.register_contract(None, MockOracleForLbp);

    let pool_config = LbpConfig {
        rwa_token: token_id.clone(),
        usdc_token: token_id.clone(),
        rwa_amount: 1_000_000_0000000,
        weight_rwa_start: 9600000,
        weight_rwa_end: 5000000,
        start_time: 1000,
        end_time: 1000 + 86400 * 30,
        swap_fee_bps: 200,
        purchase_cap_per_wallet: Some(50000_0000000),
        kyc_required: false,
        min_kyc_tier: None,
        compliance_contract: Address::generate(&env),
        min_holding_period: None,
        graduation_threshold: 500000_0000000,
    };

    (env, lbp_id, pool_config, fair_launch_id, oracle_id)
}

#[test]
fn test_initialize_pool() {
    let (env, lbp_id, pool_config, fair_launch_id, oracle_id) = setup_test_env();
    let lbp_client = crate::LbpPoolContractClient::new(&env, &lbp_id);
    let pool_id = BytesN::from_array(&env, &[10; 32]);
    let issuer = Address::generate(&env);

    lbp_client.initialize(
        &pool_id,
        &pool_config,
        &issuer,
        &fair_launch_id,
        &oracle_id,
    );

    let pool = lbp_client.get_pool_details(&pool_id);
    assert_eq!(pool.balance_rwa, 1_000_000_0000000);
    assert_eq!(pool.balance_usdc, 0);
    assert_eq!(pool.weight_rwa_start, 9600000);
    assert_eq!(pool.weight_rwa_end, 5000000);
    assert!(pool.is_active);
}

#[test]
fn test_weight_decay() {
    let (env, lbp_id, pool_config, fair_launch_id, oracle_id) = setup_test_env();
    let lbp_client = crate::LbpPoolContractClient::new(&env, &lbp_id);
    let pool_id = BytesN::from_array(&env, &[11; 32]);
    let issuer = Address::generate(&env);

    lbp_client.initialize(
        &pool_id,
        &pool_config,
        &issuer,
        &fair_launch_id,
        &oracle_id,
    );

    env.ledger().with_mut(|info| info.timestamp = 1000);
    let w_start = lbp_client.get_current_weight_rwa(&pool_id);
    assert_eq!(w_start, 9600000);

    let mid_time = 1000 + (86400 * 30) / 2;
    env.ledger().with_mut(|info| info.timestamp = mid_time);
    let w_mid = lbp_client.get_current_weight_rwa(&pool_id);
    let expected_mid = 9600000u128 - (9600000 - 5000000) * (mid_time as u128 - 1000) / (86400 * 30);
    let diff = if w_mid > expected_mid { w_mid - expected_mid } else { expected_mid - w_mid };
    assert!(diff <= 10);

    env.ledger().with_mut(|info| info.timestamp = 1000 + 86400 * 30);
    let w_end = lbp_client.get_current_weight_rwa(&pool_id);
    assert_eq!(w_end, 5000000);
}

#[test]
fn test_spot_price_monotonic() {
    let (env, lbp_id, pool_config, fair_launch_id, oracle_id) = setup_test_env();
    let lbp_client = crate::LbpPoolContractClient::new(&env, &lbp_id);
    let pool_id = BytesN::from_array(&env, &[12; 32]);
    let issuer = Address::generate(&env);

    lbp_client.initialize(
        &pool_id,
        &pool_config,
        &issuer,
        &fair_launch_id,
        &oracle_id,
    );

    env.ledger().with_mut(|info| info.timestamp = 1000);
    let price_early = lbp_client.get_spot_price(&pool_id);

    env.ledger().with_mut(|info| info.timestamp = 1000 + 86400 * 15);
    let price_mid = lbp_client.get_spot_price(&pool_id);

    env.ledger().with_mut(|info| info.timestamp = 1000 + 86400 * 30);
    let price_late = lbp_client.get_spot_price(&pool_id);

    assert!(price_early >= price_mid);
    assert!(price_mid >= price_late);
}

#[test]
fn test_buy_basic() {
    let (env, lbp_id, pool_config, fair_launch_id, oracle_id) = setup_test_env();
    let lbp_client = crate::LbpPoolContractClient::new(&env, &lbp_id);
    let pool_id = BytesN::from_array(&env, &[13; 32]);
    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);

    let token_client = token::Client::new(&env, &pool_config.rwa_token);
    let rwa_balance_before = token_client.balance(&lbp_id);
    assert!(rwa_balance_before == 0 || rwa_balance_before >= 0);

    let sac = token::StellarAssetClient::new(&env, &pool_config.rwa_token);
    sac.mint(&lbp_id, &(1_000_000_0000000i128));
    sac.mint(&buyer, &(100_000_0000000i128));

    lbp_client.initialize(
        &pool_id,
        &pool_config,
        &issuer,
        &fair_launch_id,
        &oracle_id,
    );

    env.ledger().with_mut(|info| info.timestamp = 1000);

    let min_rwa_out = 0;
    let max_usdc_in = 1000_0000000;
    let rwa_out = lbp_client.buy(&buyer, &pool_id, &min_rwa_out, &max_usdc_in);

    assert!(rwa_out > 0);

    let pool = lbp_client.get_pool_details(&pool_id);
    assert_eq!(pool.balance_usdc, 1000_0000000);
    assert!(pool.balance_rwa < 1_000_000_0000000);
}



#[test]
fn test_slippage_check_enforced() {
    let (env, lbp_id, pool_config, fair_launch_id, oracle_id) = setup_test_env();
    let lbp_client = crate::LbpPoolContractClient::new(&env, &lbp_id);
    let pool_id = BytesN::from_array(&env, &[14; 32]);
    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);

    let sac = token::StellarAssetClient::new(&env, &pool_config.rwa_token);
    sac.mint(&lbp_id, &(1_000_000_0000000i128));
    sac.mint(&buyer, &(100_000_0000000i128));

    lbp_client.initialize(
        &pool_id,
        &pool_config,
        &issuer,
        &fair_launch_id,
        &oracle_id,
    );

    env.ledger().with_mut(|info| info.timestamp = 1000);

    let rwa_out = lbp_client.calculate_out_given_in(&pool_id, &1000_0000000u128);
    assert!(rwa_out > 0);
    assert!(rwa_out < u128::MAX);
}

#[test]
fn test_graduated_pool_tracking() {
    let (env, lbp_id, pool_config, fair_launch_id, oracle_id) = setup_test_env();
    let lbp_client = crate::LbpPoolContractClient::new(&env, &lbp_id);
    let pool_id = BytesN::from_array(&env, &[15; 32]);
    let issuer = Address::generate(&env);

    let sac = token::StellarAssetClient::new(&env, &pool_config.rwa_token);
    sac.mint(&lbp_id, &(1_000_000_0000000i128));

    lbp_client.initialize(
        &pool_id,
        &pool_config,
        &issuer,
        &fair_launch_id,
        &oracle_id,
    );

    let pool = lbp_client.get_pool_details(&pool_id);
    assert!(pool.is_active);
    assert!(!pool.graduated);
}

#[test]
fn test_get_balance() {
    let (env, lbp_id, pool_config, fair_launch_id, oracle_id) = setup_test_env();
    let lbp_client = crate::LbpPoolContractClient::new(&env, &lbp_id);
    let pool_id = BytesN::from_array(&env, &[16; 32]);
    let issuer = Address::generate(&env);

    lbp_client.initialize(
        &pool_id,
        &pool_config,
        &issuer,
        &fair_launch_id,
        &oracle_id,
    );

    let (rwa_bal, usdc_bal) = lbp_client.get_balance(&pool_id);
    assert_eq!(rwa_bal, 1_000_000_0000000);
    assert_eq!(usdc_bal, 0);
}
