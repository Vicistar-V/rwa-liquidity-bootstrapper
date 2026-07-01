use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};
use soroban_sdk::testutils::{Address as _, Ledger as _};

use crate::GraduationEngine;
use amm_math::{GraduationCriteria, GraduationReceipt, PoolSummary, PoolType};

#[contracttype]
pub enum MockFactoryDataKey {
    Token,
}

#[contract]
pub struct MockFactoryForGraduation;

#[contractimpl]
impl MockFactoryForGraduation {
    pub fn set_token(env: Env, token: Address) {
        env.storage().instance().set(&MockFactoryDataKey::Token, &token);
    }

    pub fn mark_pool_graduated(_env: Env, _pool_id: BytesN<32>) {
    }

    pub fn get_pool_summary(env: Env, pool_id: BytesN<32>) -> PoolSummary {
        let token: Address = env.storage().instance().get(&MockFactoryDataKey::Token).unwrap();
        PoolSummary {
            pool_id,
            pool_type: PoolType::Lbp,
            rwa_token: token.clone(),
            usdc_token: token,
            is_active: false,
            graduated: true,
            total_usdc_raised: 0,
        }
    }
}

#[contract]
pub struct MockFairLaunchForGraduation;

#[contractimpl]
impl MockFairLaunchForGraduation {
    pub fn get_pool_purchase_state(_env: Env, _pool_id: BytesN<32>) -> (u128, u128) {
        (100_000_0000000, 1_000_000_0000000)
    }
}

fn setup_time_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let grad_id = env.register_contract(None, GraduationEngine);
    let factory_id = env.register_contract(None, MockFactoryForGraduation);
    let fl_id = env.register_contract(None, MockFairLaunchForGraduation);

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(admin.clone());
    let sac = token::StellarAssetClient::new(&env, &token_id);
    sac.mint(&grad_id, &i128::MAX);

    let client = crate::GraduationEngineClient::new(&env, &grad_id);
    client.set_admin(&admin);
    client.set_factory(&factory_id);
    client.set_fair_launch(&fl_id);
    client.set_fee_recipients(
        &Address::generate(&env),
        &Address::generate(&env),
    );

    (env, grad_id, factory_id, fl_id)
}

#[test]
fn test_time_based_graduation() {
    let (env, grad_id, _, _) = setup_time_env();
    let client = crate::GraduationEngineClient::new(&env, &grad_id);
    let pool_id = BytesN::from_array(&env, &[10; 32]);
    let issuer = Address::generate(&env);

    let start_time: u64 = 1000;
    let duration: u64 = 86400 * 30;
    let threshold = start_time + duration;

    client.initialize(
        &pool_id,
        &GraduationCriteria::TimeElapsed,
        &issuer,
        &(threshold as u128),
    );

    env.ledger().with_mut(|info| info.timestamp = start_time + duration - 1);
    let status = client.check_graduation_ready(&pool_id);
    assert!(matches!(status, crate::GraduationStatus::NotReady));

    env.ledger().with_mut(|info| info.timestamp = threshold);
    let status = client.check_graduation_ready(&pool_id);
    assert!(matches!(status, crate::GraduationStatus::Ready));
}

#[test]
fn test_funds_based_graduation() {
    let (env, grad_id, _, _) = setup_time_env();
    let client = crate::GraduationEngineClient::new(&env, &grad_id);
    let pool_id = BytesN::from_array(&env, &[11; 32]);
    let issuer = Address::generate(&env);

    client.initialize(
        &pool_id,
        &GraduationCriteria::FundsRaised,
        &issuer,
        &500_000_0000000u128,
    );

    let status = client.check_graduation_ready(&pool_id);
    assert!(matches!(status, crate::GraduationStatus::Ready));
}

#[test]
fn test_early_graduation_by_issuer() {
    let (env, grad_id, _, _) = setup_time_env();
    let client = crate::GraduationEngineClient::new(&env, &grad_id);
    let pool_id = BytesN::from_array(&env, &[12; 32]);
    let issuer = Address::generate(&env);

    client.initialize(
        &pool_id,
        &GraduationCriteria::IssuerTriggered,
        &issuer,
        &0u128,
    );

    client.trigger_early_graduation(&issuer, &pool_id);

    let status = client.check_graduation_ready(&pool_id);
    assert!(matches!(status, crate::GraduationStatus::Graduated));
}

#[test]
fn test_graduate_pool_full_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let grad_id = env.register_contract(None, GraduationEngine);
    let factory_id = env.register_contract(None, MockFactoryForGraduation);
    let fl_id = env.register_contract(None, MockFairLaunchForGraduation);

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(admin.clone());

    let client = crate::GraduationEngineClient::new(&env, &grad_id);
    client.set_admin(&admin);
    client.set_factory(&factory_id);
    client.set_fair_launch(&fl_id);

    let mut args: Vec<Val> = Vec::new(&env);
    args.push_back(token_id.clone().into_val(&env));
    env.invoke_contract::<()>(&factory_id, &Symbol::new(&env, "set_token"), args);

    let protocol_recv = Address::generate(&env);
    let dex_seed = Address::generate(&env);
    client.set_fee_recipients(&protocol_recv, &dex_seed);

    let pool_id = BytesN::from_array(&env, &[20; 32]);
    let issuer = Address::generate(&env);

    let sac = token::StellarAssetClient::new(&env, &token_id);
    sac.mint(&grad_id, &(2_000_000_0000000i128));

    client.initialize(
        &pool_id,
        &GraduationCriteria::FundsRaised,
        &issuer,
        &1_000_000_0000000u128,
    );

    env.ledger().with_mut(|info| info.timestamp = 5000);
    let receipt = client.graduate_pool(&pool_id);

    assert!(receipt.total_usdc_raised > 0);
    assert!(receipt.total_tokens_sold > 0);
    assert!(receipt.usdc_to_issuer > 0);
    assert!(receipt.usdc_to_lp_pool > 0);
    assert!(receipt.graduation_timestamp > 0);
}

#[test]
fn test_graduate_pool_not_ready_caught() {
    let (env, grad_id, _, _) = setup_time_env();
    let client = crate::GraduationEngineClient::new(&env, &grad_id);
    let pool_id = BytesN::from_array(&env, &[30; 32]);
    let issuer = Address::generate(&env);

    client.initialize(
        &pool_id,
        &GraduationCriteria::TimeElapsed,
        &issuer,
        &(u64::MAX as u128),
    );

    let status = client.check_graduation_ready(&pool_id);
    assert!(matches!(status, crate::GraduationStatus::NotReady));
}

#[test]
fn test_early_graduation_issuer_check() {
    let (env, grad_id, _, _) = setup_time_env();
    let client = crate::GraduationEngineClient::new(&env, &grad_id);
    let pool_id = BytesN::from_array(&env, &[40; 32]);
    let real_issuer = Address::generate(&env);
    let fake_issuer = Address::generate(&env);

    client.initialize(
        &pool_id,
        &GraduationCriteria::IssuerTriggered,
        &real_issuer,
        &0u128,
    );

    let status = client.check_graduation_ready(&pool_id);
    assert!(matches!(status, crate::GraduationStatus::NotReady));

    client.trigger_early_graduation(&real_issuer, &pool_id);
    let status = client.check_graduation_ready(&pool_id);
    assert!(matches!(status, crate::GraduationStatus::Graduated));
}
