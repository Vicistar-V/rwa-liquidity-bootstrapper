use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, IntoVal, Symbol, Val, Vec};
use soroban_sdk::testutils::Address as _;

use crate::{ComplianceBridge, KycStatus, PoolComplianceConfig};
use amm_math::ComplianceDecision;

#[contract]
pub struct MockExternalCompliance;

#[contractimpl]
impl MockExternalCompliance {
    pub fn check_purchase(
        _env: Env,
        _buyer: Address,
        _pool_id: BytesN<32>,
        _amount: u128,
    ) -> ComplianceDecision {
        ComplianceDecision::Reject(BytesN::from_array(&_env, &[0x03; 32]))
    }
}

fn setup() -> (Env, Address, BytesN<32>, crate::ComplianceBridgeClient) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);

    client.initialize(&admin, &pool_id, &compliance_contract);

    (env, admin, pool_id, client)
}

#[test]
fn test_initialize() {
    let (env, admin, pool_id, _) = setup();

    let config = env
        .as_contract(&env.register_contract(None, ComplianceBridge), || {
            env.storage()
                .persistent()
                .get::<_, PoolComplianceConfig>(&crate::ComplianceDataKey::PoolConfig(
                    pool_id.clone(),
                ))
        })
        .unwrap();

    assert!(config.kyc_required);
    assert_eq!(config.min_kyc_tier, 1);
}

#[test]
fn test_initialize_panics_on_double_init() {
    let (_env, admin, pool_id, client) = setup();
    let other = Address::generate(&_env);

    client.initialize(&admin, &pool_id, &other);
}

#[test]
fn test_set_admin() {
    let (env, admin, _pool_id, client) = setup();
    let new_admin = Address::generate(&env);

    client.set_admin(&new_admin);
}

#[test]
fn test_set_and_get_pool_config() {
    let (env, _admin, pool_id, client) = setup();

    let config = PoolComplianceConfig {
        kyc_required: false,
        min_kyc_tier: 0,
        external_compliance_contract: None,
    };
    client.set_pool_config(&pool_id, &config);

    let retrieved = client.get_pool_config(&pool_id).unwrap();
    assert!(!retrieved.kyc_required);
    assert_eq!(retrieved.min_kyc_tier, 0);
    assert!(retrieved.external_compliance_contract.is_none());
}

#[test]
fn test_get_pool_config_none() {
    let (_env, _admin, _pool_id, client) = setup();
    let unknown_pool = BytesN::from_array(&_env, &[99; 32]);

    let config = client.get_pool_config(&unknown_pool);
    assert!(config.is_none());
}

#[test]
fn test_set_and_get_wallet_kyc() {
    let (_env, _admin, _pool_id, client) = setup();
    let wallet = Address::generate(&_env);

    client.set_wallet_kyc(&wallet, &2u32, &true);

    let kyc = client.get_wallet_kyc(&wallet).unwrap();
    assert_eq!(kyc.tier, 2);
    assert!(kyc.verified);
    assert!(kyc.timestamp > 0);
}

#[test]
fn test_get_wallet_kyc_none() {
    let (_env, _admin, _pool_id, client) = setup();
    let wallet = Address::generate(&_env);

    let kyc = client.get_wallet_kyc(&wallet);
    assert!(kyc.is_none());
}

#[test]
fn test_set_blacklisted_and_check() {
    let (_env, _admin, _pool_id, client) = setup();
    let wallet = Address::generate(&_env);

    assert!(!client.is_blacklisted(&wallet));

    client.set_blacklisted(&wallet, &true);
    assert!(client.is_blacklisted(&wallet));

    client.set_blacklisted(&wallet, &false);
    assert!(!client.is_blacklisted(&wallet));
}

#[test]
fn test_check_purchase_approves_no_pool_config() {
    let (_env, _admin, _pool_id, client) = setup();
    let buyer = Address::generate(&_env);
    let unknown_pool = BytesN::from_array(&_env, &[99; 32]);

    let decision = client.check_purchase(&buyer, &unknown_pool, &1000u128);
    assert_eq!(decision, ComplianceDecision::Approve);
}

#[test]
fn test_check_purchase_rejects_blacklisted() {
    let (_env, _admin, pool_id, client) = setup();
    let buyer = Address::generate(&_env);

    client.set_blacklisted(&buyer, &true);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert!(matches!(decision, ComplianceDecision::Reject(_)));
}

#[test]
fn test_check_purchase_pending_kyc_when_not_set() {
    let (_env, _admin, pool_id, client) = setup();
    let buyer = Address::generate(&_env);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert_eq!(decision, ComplianceDecision::PendingKyc);
}

#[test]
fn test_check_purchase_approves_with_kyc() {
    let (_env, _admin, pool_id, client) = setup();
    let buyer = Address::generate(&_env);

    client.set_wallet_kyc(&buyer, &1u32, &true);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert_eq!(decision, ComplianceDecision::Approve);
}

#[test]
fn test_check_purchase_rejects_insufficient_kyc_tier() {
    let (_env, _admin, pool_id, client) = setup();
    let buyer = Address::generate(&_env);

    client.set_wallet_kyc(&buyer, &0u32, &true);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert!(matches!(decision, ComplianceDecision::Reject(_)));
}

#[test]
fn test_check_purchase_rejects_unverified_kyc() {
    let (_env, _admin, pool_id, client) = setup();
    let buyer = Address::generate(&_env);

    client.set_wallet_kyc(&buyer, &1u32, &false);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert!(matches!(decision, ComplianceDecision::Reject(_)));
}

#[test]
fn test_check_purchase_with_kyc_disabled() {
    let (_env, _admin, pool_id, client) = setup();
    let buyer = Address::generate(&_env);

    let config = PoolComplianceConfig {
        kyc_required: false,
        min_kyc_tier: 0,
        external_compliance_contract: None,
    };
    client.set_pool_config(&pool_id, &config);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert_eq!(decision, ComplianceDecision::Approve);
}

#[test]
fn test_check_purchase_delegates_to_external_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let compliance_id = env.register_contract(None, ComplianceBridge);
    let external_id = env.register_contract(None, MockExternalCompliance);

    let client = crate::ComplianceBridgeClient::new(&env, &compliance_id);

    let pool_id = BytesN::from_array(&env, &[5; 32]);

    client.initialize(&admin, &pool_id, &external_id);

    let buyer = Address::generate(&env);
    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);

    assert!(matches!(decision, ComplianceDecision::Reject(_)));
}

#[test]
fn test_set_compliance_contract_updates_external() {
    let (_env, _admin, pool_id, client) = setup();
    let new_external = Address::generate(&_env);

    client.set_compliance_contract(&pool_id, &new_external);

    let config = client.get_pool_config(&pool_id).unwrap();
    assert_eq!(config.external_compliance_contract.unwrap(), new_external);
}

#[test]
fn test_check_purchase_bypasses_kyc_when_no_config() {
    let (_env, _admin, _pool_id, client) = setup();
    let buyer = Address::generate(&_env);
    let unknown_pool = BytesN::from_array(&_env, &[42; 32]);

    let decision = client.check_purchase(&buyer, &unknown_pool, &500u128);
    assert_eq!(decision, ComplianceDecision::Approve);
}
