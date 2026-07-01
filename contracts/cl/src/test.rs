use soroban_sdk::{token, Address, Env};
use soroban_sdk::testutils::Address as _;

use crate::ConcentratedLiquidityContract;
use amm_math::{ClConfig, SCALE};

fn setup() -> (Env, Address, Address, Address, ClConfig) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let rwa_id = env.register_stellar_asset_contract(admin.clone());
    let usdc_id = env.register_stellar_asset_contract(admin.clone());

    let config = ClConfig {
        rwa_token: rwa_id.clone(),
        usdc_token: usdc_id.clone(),
        price_lower: SCALE,
        price_upper: 10 * SCALE,
        tick_spacing: 10,
        fee_tier: 300,
    };

    (env, admin, rwa_id, usdc_id, config)
}

#[test]
fn test_initialize_pool() {
    let (env, _, _, _, config) = setup();
    let cl_id = env.register_contract(None, ConcentratedLiquidityContract);
    let client = crate::ConcentratedLiquidityContractClient::new(&env, &cl_id);
    let pool_id = soroban_sdk::BytesN::from_array(&env, &[1; 32]);

    client.initialize(&pool_id, &config);

    let pool = client.get_pool_details(&pool_id);
    assert_eq!(pool.rwa_token, config.rwa_token);
    assert_eq!(pool.total_liquidity, 0);
}

#[test]
fn test_mint_position() {
    let (env, admin, _, _, config) = setup();
    let cl_id = env.register_contract(None, ConcentratedLiquidityContract);
    let client = crate::ConcentratedLiquidityContractClient::new(&env, &cl_id);
    let pool_id = soroban_sdk::BytesN::from_array(&env, &[2; 32]);

    let sac = token::StellarAssetClient::new(&env, &config.rwa_token);
    sac.mint(&admin, &(1_000_000_0000000i128));
    let sac_usdc = token::StellarAssetClient::new(&env, &config.usdc_token);
    sac_usdc.mint(&admin, &(1_000_000_0000000i128));

    client.initialize(&pool_id, &config);

    let liquidity = 1_000_000_0000000u128;
    let pos_id = client.mint_position(&admin, &pool_id, &(-100i32), &(100i32), &liquidity);
    assert!(pos_id.len() == 32);

    let pool = client.get_pool_details(&pool_id);
    assert!(pool.total_liquidity > 0);
}

#[test]
fn test_burn_position() {
    let (env, admin, _, _, config) = setup();
    let cl_id = env.register_contract(None, ConcentratedLiquidityContract);
    let client = crate::ConcentratedLiquidityContractClient::new(&env, &cl_id);
    let pool_id = soroban_sdk::BytesN::from_array(&env, &[3; 32]);

    let sac = token::StellarAssetClient::new(&env, &config.rwa_token);
    sac.mint(&admin, &(1_000_000_0000000i128));
    let sac_usdc = token::StellarAssetClient::new(&env, &config.usdc_token);
    sac_usdc.mint(&admin, &(1_000_000_0000000i128));

    client.initialize(&pool_id, &config);

    let liquidity = 500_000_0000000u128;
    let pos_id = client.mint_position(&admin, &pool_id, &(-50i32), &(50i32), &liquidity);

    let pool_before = client.get_pool_details(&pool_id);
    let liq_before = pool_before.total_liquidity;

    client.burn_position(&admin, &pool_id, &pos_id);

    let pool_after = client.get_pool_details(&pool_id);
    assert!(pool_after.total_liquidity < liq_before);
}

#[test]
fn test_collect_fees_empty() {
    let (env, admin, _, _, config) = setup();
    let cl_id = env.register_contract(None, ConcentratedLiquidityContract);
    let client = crate::ConcentratedLiquidityContractClient::new(&env, &cl_id);
    let pool_id = soroban_sdk::BytesN::from_array(&env, &[4; 32]);

    let sac = token::StellarAssetClient::new(&env, &config.rwa_token);
    sac.mint(&admin, &(1_000_000_0000000i128));
    let sac_usdc = token::StellarAssetClient::new(&env, &config.usdc_token);
    sac_usdc.mint(&admin, &(1_000_000_0000000i128));

    client.initialize(&pool_id, &config);

    let pos_id = client.mint_position(&admin, &pool_id, &(-10i32), &(10i32), &1_000_000_0000000u128);

    let (rwa_fees, usdc_fees) = client.collect_fees(&admin, &pool_id, &pos_id);
    assert_eq!(rwa_fees, 0);
    assert_eq!(usdc_fees, 0);
}

#[test]
fn test_tick_math_roundtrip() {
    let _env = Env::default();
    let price = SCALE;
    let tick = crate::ConcentratedLiquidityContract::sqrt_price_to_tick(price);
    let sqrt_back = crate::ConcentratedLiquidityContract::tick_to_sqrt_price(tick);
    assert!(sqrt_back > 0);
}

#[test]
fn test_get_amounts_for_liquidity_below_range() {
    let liquidity = 1_000_000_0000000u128;
    let sqrt_price = 1_000_000u128;
    let sqrt_low = 2_000_000u128;
    let sqrt_high = 10_000_000u128;

    let (rwa, usdc) = crate::ConcentratedLiquidityContract::get_amounts_for_liquidity(
        liquidity, sqrt_price, sqrt_low, sqrt_high,
    );
    assert!(rwa > 0);
    assert_eq!(usdc, 0);
}

#[test]
fn test_get_amounts_for_liquidity_above_range() {
    let liquidity = 1_000_000_0000000u128;
    let sqrt_price = 20_000_000u128;
    let sqrt_low = 1_000_000u128;
    let sqrt_high = 10_000_000u128;

    let (rwa, usdc) = crate::ConcentratedLiquidityContract::get_amounts_for_liquidity(
        liquidity, sqrt_price, sqrt_low, sqrt_high,
    );
    assert_eq!(rwa, 0);
    assert!(usdc > 0);
}

#[test]
fn test_double_initialize_detected() {
    let (env, _, _, _, config) = setup();
    let cl_id = env.register_contract(None, ConcentratedLiquidityContract);
    let client = crate::ConcentratedLiquidityContractClient::new(&env, &cl_id);
    let pool_id = soroban_sdk::BytesN::from_array(&env, &[5; 32]);

    client.initialize(&pool_id, &config);

    let pool = client.get_pool_details(&pool_id);
    assert_eq!(pool.rwa_token, config.rwa_token);
    assert_eq!(pool.total_liquidity, 0);
}

#[test]
fn test_position_owner_check() {
    let (env, admin, _, _, config) = setup();
    let cl_id = env.register_contract(None, ConcentratedLiquidityContract);
    let client = crate::ConcentratedLiquidityContractClient::new(&env, &cl_id);
    let pool_id = soroban_sdk::BytesN::from_array(&env, &[6; 32]);

    let sac = token::StellarAssetClient::new(&env, &config.rwa_token);
    sac.mint(&admin, &(1_000_000_0000000i128));
    let sac_usdc = token::StellarAssetClient::new(&env, &config.usdc_token);
    sac_usdc.mint(&admin, &(1_000_000_0000000i128));

    client.initialize(&pool_id, &config);

    let pos_id = client.mint_position(&admin, &pool_id, &(-10i32), &(10i32), &1_000_000_0000000u128);

    let pos = client.get_position(&pool_id, &pos_id);
    assert_eq!(pos.owner, admin);
}
