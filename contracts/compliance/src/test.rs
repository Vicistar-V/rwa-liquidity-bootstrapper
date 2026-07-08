use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};
use soroban_sdk::testutils::Address as _;

use amm_math::{ComplianceDecision, PoolComplianceConfig};

#[contract]
pub struct MockRejectCompliance;

#[contractimpl]
impl MockRejectCompliance {
    pub fn check_purchase(
        _env: Env,
        _buyer: Address,
        _pool_id: BytesN<32>,
        _amount: u128,
    ) -> ComplianceDecision {
        ComplianceDecision::Reject(BytesN::from_array(&_env, &[0x03; 32]))
    }
}

pub mod mock_approve {
    use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};
    use amm_math::ComplianceDecision;

    #[contract]
    pub struct MockApproveCompliance;

    #[contractimpl]
    impl MockApproveCompliance {
        pub fn check_purchase(
            _env: Env,
            _buyer: Address,
            _pool_id: BytesN<32>,
            _amount: u128,
        ) -> ComplianceDecision {
            ComplianceDecision::Approve
        }
    }
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let config: PoolComplianceConfig = client.get_pool_config(&pool_id).unwrap();
    assert!(config.kyc_required);
    assert_eq!(config.min_kyc_tier, 1);
}

#[test]
fn test_set_and_get_pool_config() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let config = PoolComplianceConfig {
        kyc_required: false,
        min_kyc_tier: 0,
    };
    client.set_pool_config(&pool_id, &config);

    let retrieved = client.get_pool_config(&pool_id).unwrap();
    assert!(!retrieved.kyc_required);
    assert_eq!(retrieved.min_kyc_tier, 0);
}

#[test]
fn test_get_pool_config_none() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let unknown_pool = BytesN::from_array(&env, &[99; 32]);
    let config = client.get_pool_config(&unknown_pool);
    assert!(config.is_none());
}

#[test]
fn test_set_and_get_wallet_kyc() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let wallet = Address::generate(&env);
    client.set_wallet_kyc(&wallet, &2u32, &true);

    let kyc = client.get_wallet_kyc(&wallet).unwrap();
    assert_eq!(kyc.tier, 2);
    assert!(kyc.verified);
    assert_eq!(kyc.wallet, wallet);
}

#[test]
fn test_get_wallet_kyc_none() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let wallet = Address::generate(&env);
    let kyc = client.get_wallet_kyc(&wallet);
    assert!(kyc.is_none());
}

#[test]
fn test_set_blacklisted_and_check() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let wallet = Address::generate(&env);
    assert!(!client.is_blacklisted(&wallet));

    client.set_blacklisted(&wallet, &true);
    assert!(client.is_blacklisted(&wallet));

    client.set_blacklisted(&wallet, &false);
    assert!(!client.is_blacklisted(&wallet));
}

#[test]
fn test_check_purchase_approves_no_pool_config() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let buyer = Address::generate(&env);
    let unknown_pool = BytesN::from_array(&env, &[99; 32]);
    let decision = client.check_purchase(&buyer, &unknown_pool, &1000u128);
    assert_eq!(decision, ComplianceDecision::Approve);
}

#[test]
fn test_check_purchase_rejects_blacklisted() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let buyer = Address::generate(&env);
    client.set_blacklisted(&buyer, &true);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert!(matches!(decision, ComplianceDecision::Reject(_)));
}

#[test]
fn test_check_purchase_pending_kyc_when_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let buyer = Address::generate(&env);
    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert_eq!(decision, ComplianceDecision::PendingKyc);
}

#[test]
fn test_check_purchase_approves_with_kyc() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let buyer = Address::generate(&env);
    client.set_wallet_kyc(&buyer, &1u32, &true);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert_eq!(decision, ComplianceDecision::Approve);
}

#[test]
fn test_check_purchase_rejects_insufficient_kyc_tier() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let buyer = Address::generate(&env);
    client.set_wallet_kyc(&buyer, &0u32, &true);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert!(matches!(decision, ComplianceDecision::Reject(_)));
}

#[test]
fn test_check_purchase_rejects_unverified_kyc() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let buyer = Address::generate(&env);
    client.set_wallet_kyc(&buyer, &1u32, &false);

    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert!(matches!(decision, ComplianceDecision::Reject(_)));
}

#[test]
fn test_check_purchase_with_kyc_disabled() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let buyer = Address::generate(&env);
    let config = PoolComplianceConfig {
        kyc_required: false,
        min_kyc_tier: 0,
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
    let compliance_id = env.register_contract(None, crate::ComplianceBridge);
    let external_id = env.register_contract(None, MockRejectCompliance);

    let client = crate::ComplianceBridgeClient::new(&env, &compliance_id);
    let pool_id = BytesN::from_array(&env, &[5; 32]);
    let dummy = Address::generate(&env);
    client.initialize(&admin, &pool_id, &dummy);
    client.set_compliance_contract(&pool_id, &external_id);

    let buyer = Address::generate(&env);
    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);

    assert!(matches!(decision, ComplianceDecision::Reject(_)));
}

#[test]
fn test_set_compliance_contract_updates_external() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let compliance_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &compliance_id);

    let pool_id = BytesN::from_array(&env, &[5; 32]);
    let dummy = Address::generate(&env);
    client.initialize(&admin, &pool_id, &dummy);

    let approve_id = env.register_contract(None, mock_approve::MockApproveCompliance);
    client.set_compliance_contract(&pool_id, &approve_id);

    let buyer = Address::generate(&env);
    let decision = client.check_purchase(&buyer, &pool_id, &1000u128);
    assert_eq!(decision, ComplianceDecision::Approve);
}
#[test]
fn test_check_purchase_bypasses_kyc_when_no_config() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, crate::ComplianceBridge);
    let client = crate::ComplianceBridgeClient::new(&env, &contract_id);

    let pool_id = BytesN::from_array(&env, &[1; 32]);
    let compliance_contract = Address::generate(&env);
    client.initialize(&admin, &pool_id, &compliance_contract);

    let buyer = Address::generate(&env);
    let unknown_pool = BytesN::from_array(&env, &[42; 32]);
    let decision = client.check_purchase(&buyer, &unknown_pool, &500u128);
    assert_eq!(decision, ComplianceDecision::Approve);
}
