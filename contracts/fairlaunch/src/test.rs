use soroban_sdk::{Address, BytesN, Env};
use soroban_sdk::testutils::{Address as _, Ledger as _};

use crate::FairLaunchController;
use amm_math::FairLaunchConfig;

fn setup_test_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let fl_id = env.register_contract(None, FairLaunchController);
    let rwa_token = Address::generate(&env);
    let reserve_token = Address::generate(&env);

    (env, fl_id, rwa_token, reserve_token)
}

#[test]
fn test_blackout_period_rejects_early_purchase() {
    let (env, fl_id, rwa_token, reserve_token) = setup_test_env();
    let client = crate::FairLaunchControllerClient::new(&env, &fl_id);
    let pool_id = BytesN::from_array(&env, &[10; 32]);
    let buyer = Address::generate(&env);

    let config = FairLaunchConfig {
        max_usdc_per_wallet: None,
        max_tokens_per_wallet: Some(1000_0000000),
        cooldown_between_purchases: None,
        initial_blackout_period: Some(3600),
        min_wallet_age_ledgers: None,
        max_pool_ownership_pct: None,
    };

    env.ledger().with_mut(|info| info.timestamp = 0);
    client.initialize(&pool_id, &config, &1000u64, &rwa_token, &reserve_token);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.check_purchase_allowed(&buyer, &pool_id, &100_0000000);
    }));
    assert!(result.is_err());

    env.ledger().with_mut(|info| info.timestamp = 1000 + 3601);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.check_purchase_allowed(&buyer, &pool_id, &100_0000000);
    }));
    assert!(result.is_ok());
}

#[test]
fn test_wallet_cap_enforced() {
    let (env, fl_id, rwa_token, reserve_token) = setup_test_env();
    let client = crate::FairLaunchControllerClient::new(&env, &fl_id);
    let pool_id = BytesN::from_array(&env, &[11; 32]);
    let buyer = Address::generate(&env);

    let config = FairLaunchConfig {
        max_usdc_per_wallet: None,
        max_tokens_per_wallet: Some(500_0000000),
        cooldown_between_purchases: None,
        initial_blackout_period: None,
        min_wallet_age_ledgers: None,
        max_pool_ownership_pct: None,
    };

    client.initialize(&pool_id, &config, &0u64, &rwa_token, &reserve_token);

    let result = client.check_purchase_allowed(&buyer, &pool_id, &300_0000000);
    assert!(result);

    client.record_purchase(&buyer, &pool_id, &300_0000000, &300_0000000);

    let result = client.check_purchase_allowed(&buyer, &pool_id, &200_0000000);
    assert!(result);

    client.record_purchase(&buyer, &pool_id, &200_0000000, &200_0000000);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.check_purchase_allowed(&buyer, &pool_id, &1_0000000);
    }));
    assert!(result.is_err());
}

#[test]
fn test_cooldown_enforced() {
    let (env, fl_id, rwa_token, reserve_token) = setup_test_env();
    let client = crate::FairLaunchControllerClient::new(&env, &fl_id);
    let pool_id = BytesN::from_array(&env, &[12; 32]);
    let buyer = Address::generate(&env);

    let config = FairLaunchConfig {
        max_usdc_per_wallet: None,
        max_tokens_per_wallet: Some(10_000_0000000),
        cooldown_between_purchases: Some(600),
        initial_blackout_period: None,
        min_wallet_age_ledgers: None,
        max_pool_ownership_pct: None,
    };

    env.ledger().with_mut(|info| info.timestamp = 1000);
    client.initialize(&pool_id, &config, &1000u64, &rwa_token, &reserve_token);

    client.check_purchase_allowed(&buyer, &pool_id, &100_0000000);
    client.record_purchase(&buyer, &pool_id, &100_0000000, &100_0000000);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.check_purchase_allowed(&buyer, &pool_id, &100_0000000);
    }));
    assert!(result.is_err());

    env.ledger().with_mut(|info| info.timestamp = 1000 + 601);
    let result = client.check_purchase_allowed(&buyer, &pool_id, &100_0000000);
    assert!(result);
}

#[test]
fn test_pool_ownership_cap() {
    let (env, fl_id, rwa_token, reserve_token) = setup_test_env();
    let client = crate::FairLaunchControllerClient::new(&env, &fl_id);
    let pool_id = BytesN::from_array(&env, &[13; 32]);
    let buyer = Address::generate(&env);

    let config = FairLaunchConfig {
        max_usdc_per_wallet: None,
        max_tokens_per_wallet: None,
        cooldown_between_purchases: None,
        initial_blackout_period: None,
        min_wallet_age_ledgers: None,
        max_pool_ownership_pct: Some(amm_math::SCALE / 10),
    };

    client.initialize(&pool_id, &config, &0u64, &rwa_token, &reserve_token);

    client.record_purchase(&buyer, &pool_id, &1000_0000000, &1000_0000000);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.check_purchase_allowed(&buyer, &pool_id, &1_000_000_0000000);
    }));
    assert!(result.is_err());
}

#[test]
fn test_record_purchase_tracking() {
    let (env, fl_id, rwa_token, reserve_token) = setup_test_env();
    let client = crate::FairLaunchControllerClient::new(&env, &fl_id);
    let pool_id = BytesN::from_array(&env, &[14; 32]);
    let buyer = Address::generate(&env);

    let config = FairLaunchConfig {
        max_usdc_per_wallet: None,
        max_tokens_per_wallet: None,
        cooldown_between_purchases: None,
        initial_blackout_period: None,
        min_wallet_age_ledgers: None,
        max_pool_ownership_pct: None,
    };

    client.initialize(&pool_id, &config, &0u64, &rwa_token, &reserve_token);

    client.record_purchase(&buyer, &pool_id, &100_0000000, &50_0000000);

    let record = client.get_wallet_purchases(&buyer, &pool_id);
    assert_eq!(record.total_tokens_purchased, 100_0000000);
    assert_eq!(record.total_usdc_spent, 50_0000000);
    assert_eq!(record.purchase_count, 1);

    client.record_purchase(&buyer, &pool_id, &200_0000000, &150_0000000);

    let record = client.get_wallet_purchases(&buyer, &pool_id);
    assert_eq!(record.total_tokens_purchased, 300_0000000);
    assert_eq!(record.total_usdc_spent, 200_0000000);
    assert_eq!(record.purchase_count, 2);

    let (total_tokens, total_usdc) = client.get_pool_purchase_state(&pool_id);
    assert_eq!(total_tokens, 300_0000000);
    assert_eq!(total_usdc, 200_0000000);
}

#[test]
fn test_usdc_cap_enforced() {
    let (env, fl_id, rwa_token, reserve_token) = setup_test_env();
    let client = crate::FairLaunchControllerClient::new(&env, &fl_id);
    let pool_id = BytesN::from_array(&env, &[15; 32]);
    let buyer = Address::generate(&env);

    let config = FairLaunchConfig {
        max_usdc_per_wallet: Some(1000_0000000),
        max_tokens_per_wallet: None,
        cooldown_between_purchases: None,
        initial_blackout_period: None,
        min_wallet_age_ledgers: None,
        max_pool_ownership_pct: None,
    };

    client.initialize(&pool_id, &config, &0u64, &rwa_token, &reserve_token);

    let result = client.check_purchase_allowed(&buyer, &pool_id, &100_0000000);
    assert!(result);

    client.record_purchase(&buyer, &pool_id, &100_0000000, &100_0000000);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.check_purchase_allowed(&buyer, &pool_id, &1000_0000000);
    }));
    assert!(result.is_err());
}
