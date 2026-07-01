use soroban_sdk::{token, Address, Env};
use soroban_sdk::testutils::Address as _;

use crate::PoolFactory;
use amm_math::{BondingConfig, ClConfig, CurveType, LbpConfig, PoolType, SCALE};

mod lbp_mock {
    use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};
    use amm_math::LbpConfig;

    #[contract]
    pub struct MockLbpForFactory;

    #[contractimpl]
    impl MockLbpForFactory {
        pub fn initialize(
            _env: Env,
            _pool_id: BytesN<32>,
            _config: LbpConfig,
            _issuer: Address,
            _fair_launch: Address,
            _oracle: Address,
        ) {
        }
    }
}

mod bonding_mock {
    use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};
    use amm_math::BondingConfig;

    #[contract]
    pub struct MockBondingForFactory;

    #[contractimpl]
    impl MockBondingForFactory {
        pub fn initialize(
            _env: Env,
            _pool_id: BytesN<32>,
            _config: BondingConfig,
            _issuer: Address,
            _reserve_token: Address,
            _fair_launch: Address,
            _oracle: Address,
        ) {
        }
    }
}

mod cl_mock {
    use soroban_sdk::{contract, contractimpl, BytesN, Env};
    use amm_math::ClConfig;

    #[contract]
    pub struct MockClForFactory;

    #[contractimpl]
    impl MockClForFactory {
        pub fn initialize(_env: Env, _pool_id: BytesN<32>, _config: ClConfig) {
        }
    }
}

fn create_test_token(env: &Env, admin: &Address) -> Address {
    let token_id = env.register_stellar_asset_contract(admin.clone());
    let sac = token::StellarAssetClient::new(env, &token_id);
    sac.mint(admin, &(1_000_000_000_000_000i128));
    token_id
}

fn deploy_factory(env: &Env) -> (Address, Address, Address, Address, Address, Address) {
    let admin = Address::generate(env);
    let factory_id = env.register_contract(None, PoolFactory);
    let factory_client = crate::PoolFactoryClient::new(env, &factory_id);

    let lbp_id = env.register_contract(None, lbp_mock::MockLbpForFactory);
    let bonding_id = env.register_contract(None, bonding_mock::MockBondingForFactory);
    let cl_id = env.register_contract(None, cl_mock::MockClForFactory);
    let fl_id = Address::generate(env);
    let oracle_id = Address::generate(env);

    factory_client.init(&admin, &lbp_id, &bonding_id, &cl_id, &fl_id, &oracle_id);
    (factory_id, lbp_id, bonding_id, cl_id, fl_id, oracle_id)
}

#[test]
fn test_factory_initialize_with_valid_addresses() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, _, _, _, _) = deploy_factory(&env);
}

#[test]
fn test_create_lbp_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let token_id = create_test_token(&env, &admin);

    let sac = token::StellarAssetClient::new(&env, &token_id);
    sac.mint(&issuer, &(100_000_0000000i128));

    let factory_id = env.register_contract(None, PoolFactory);
    let factory_client = crate::PoolFactoryClient::new(&env, &factory_id);

    let lbp_id = env.register_contract(None, lbp_mock::MockLbpForFactory);
    let bonding_id = env.register_contract(None, bonding_mock::MockBondingForFactory);
    let cl_id = env.register_contract(None, cl_mock::MockClForFactory);
    let fl_id = Address::generate(&env);
    let oracle_id = Address::generate(&env);

    factory_client.init(&admin, &lbp_id, &bonding_id, &cl_id, &fl_id, &oracle_id);

    let config = LbpConfig {
        rwa_token: token_id.clone(),
        usdc_token: token_id.clone(),
        rwa_amount: 1000_0000000,
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

    let pool_id = factory_client.create_lbp_pool(&issuer, &config);
    let summary = factory_client.get_pool_summary(&pool_id);
    assert_eq!(summary.pool_type, PoolType::Lbp);
    assert!(summary.is_active);
    assert!(!summary.graduated);
}

#[test]
fn test_create_bonding_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let token_id = create_test_token(&env, &admin);

    let factory_id = env.register_contract(None, PoolFactory);
    let factory_client = crate::PoolFactoryClient::new(&env, &factory_id);

    let lbp_id = env.register_contract(None, lbp_mock::MockLbpForFactory);
    let bonding_id = env.register_contract(None, bonding_mock::MockBondingForFactory);
    let cl_id = env.register_contract(None, cl_mock::MockClForFactory);
    let fl_id = Address::generate(&env);
    let oracle_id = Address::generate(&env);

    factory_client.init(&admin, &lbp_id, &bonding_id, &cl_id, &fl_id, &oracle_id);

    let config = BondingConfig {
        rwa_token: token_id.clone(),
        reserve_token: token_id,
        curve_type: CurveType::Logarithmic,
        coefficient_a: SCALE,
        coefficient_b: SCALE / 10,
        max_supply: 1_000_000_0000000,
        price_ceiling: 100 * SCALE,
        purchase_cap_per_wallet: Some(100_000_0000000),
        kyc_required: false,
        compliance_contract: Address::generate(&env),
    };

    let pool_id = factory_client.create_bonding_pool(&issuer, &config);
    let summary = factory_client.get_pool_summary(&pool_id);
    assert_eq!(summary.pool_type, PoolType::BondingCurve);
    assert!(!summary.graduated);
}

#[test]
fn test_create_cl_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let factory_id = env.register_contract(None, PoolFactory);
    let factory_client = crate::PoolFactoryClient::new(&env, &factory_id);

    let lbp_id = env.register_contract(None, lbp_mock::MockLbpForFactory);
    let bonding_id = env.register_contract(None, bonding_mock::MockBondingForFactory);
    let cl_id = env.register_contract(None, cl_mock::MockClForFactory);
    let fl_id = Address::generate(&env);
    let oracle_id = Address::generate(&env);

    factory_client.init(&admin, &lbp_id, &bonding_id, &cl_id, &fl_id, &oracle_id);

    let config = ClConfig {
        rwa_token: Address::generate(&env),
        usdc_token: Address::generate(&env),
        price_lower: SCALE,
        price_upper: 10 * SCALE,
        tick_spacing: 10,
        fee_tier: 300,
    };

    let pool_id = factory_client.create_cl_pool(&admin, &config);
    let summary = factory_client.get_pool_summary(&pool_id);
    assert_eq!(summary.pool_type, PoolType::ConcentratedLiquidity);
}

#[test]
fn test_list_all_pools_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let token_id = create_test_token(&env, &admin);

    let sac = token::StellarAssetClient::new(&env, &token_id);
    sac.mint(&issuer, &(100_000_0000000i128));

    let factory_id = env.register_contract(None, PoolFactory);
    let factory_client = crate::PoolFactoryClient::new(&env, &factory_id);

    let lbp_id = env.register_contract(None, lbp_mock::MockLbpForFactory);
    let bonding_id = env.register_contract(None, bonding_mock::MockBondingForFactory);
    let cl_id = env.register_contract(None, cl_mock::MockClForFactory);
    let fl_id = Address::generate(&env);
    let oracle_id = Address::generate(&env);

    factory_client.init(&admin, &lbp_id, &bonding_id, &cl_id, &fl_id, &oracle_id);

    let config = LbpConfig {
        rwa_token: token_id.clone(),
        usdc_token: token_id.clone(),
        rwa_amount: 1000_0000000,
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

    for _ in 0..5 {
        factory_client.create_lbp_pool(&issuer, &config);
    }

    let all = factory_client.list_all_pools(&0u32, &10u32);
    assert_eq!(all.len(), 5);

    let page1 = factory_client.list_all_pools(&0u32, &2u32);
    assert_eq!(page1.len(), 2);

    let page2 = factory_client.list_all_pools(&2u32, &2u32);
    assert_eq!(page2.len(), 2);

    let page3 = factory_client.list_all_pools(&4u32, &2u32);
    assert_eq!(page3.len(), 1);
}

#[test]
fn test_get_pools_for_asset() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let token_a = create_test_token(&env, &admin);
    let token_b = create_test_token(&env, &admin);

    let sac_a = token::StellarAssetClient::new(&env, &token_a);
    sac_a.mint(&issuer, &(100_000_0000000i128));

    let sac_b = token::StellarAssetClient::new(&env, &token_b);
    sac_b.mint(&issuer, &(100_000_0000000i128));

    let factory_id = env.register_contract(None, PoolFactory);
    let factory_client = crate::PoolFactoryClient::new(&env, &factory_id);

    let lbp_id = env.register_contract(None, lbp_mock::MockLbpForFactory);
    let bonding_id = env.register_contract(None, bonding_mock::MockBondingForFactory);
    let cl_id = env.register_contract(None, cl_mock::MockClForFactory);
    let fl_id = Address::generate(&env);
    let oracle_id = Address::generate(&env);

    factory_client.init(&admin, &lbp_id, &bonding_id, &cl_id, &fl_id, &oracle_id);

    let make_config = |token: Address| LbpConfig {
        rwa_token: token.clone(),
        usdc_token: token,
        rwa_amount: 1000_0000000,
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

    factory_client.create_lbp_pool(&issuer, &make_config(token_a.clone()));
    factory_client.create_lbp_pool(&issuer, &make_config(token_a.clone()));
    factory_client.create_lbp_pool(&issuer, &make_config(token_b.clone()));

    let pools_for_a = factory_client.get_pools_for_asset(&token_a);
    assert_eq!(pools_for_a.len(), 2);

    let pools_for_b = factory_client.get_pools_for_asset(&token_b);
    assert_eq!(pools_for_b.len(), 1);
}

#[test]
fn test_mark_pool_graduated() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let token_id = create_test_token(&env, &admin);

    let sac = token::StellarAssetClient::new(&env, &token_id);
    sac.mint(&issuer, &(100_000_0000000i128));

    let factory_id = env.register_contract(None, PoolFactory);
    let factory_client = crate::PoolFactoryClient::new(&env, &factory_id);

    let lbp_id = env.register_contract(None, lbp_mock::MockLbpForFactory);
    let bonding_id = env.register_contract(None, bonding_mock::MockBondingForFactory);
    let cl_id = env.register_contract(None, cl_mock::MockClForFactory);
    let fl_id = Address::generate(&env);
    let oracle_id = Address::generate(&env);

    factory_client.init(&admin, &lbp_id, &bonding_id, &cl_id, &fl_id, &oracle_id);

    let config = LbpConfig {
        rwa_token: token_id.clone(),
        usdc_token: token_id,
        rwa_amount: 1000_0000000,
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

    let pool_id = factory_client.create_lbp_pool(&issuer, &config);
    let mut summary = factory_client.get_pool_summary(&pool_id);
    assert!(!summary.graduated);

    factory_client.mark_pool_graduated(&pool_id);
    summary = factory_client.get_pool_summary(&pool_id);
    assert!(summary.graduated);
    assert!(!summary.is_active);
}
