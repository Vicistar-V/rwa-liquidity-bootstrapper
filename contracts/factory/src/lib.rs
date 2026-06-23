use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

use amm_math::{
    BondingConfig, BondingCurvePool, ClConfig, ConcentratedLiquidityPool, LbpConfig, LbpPool,
    PoolSummary, PoolType,
};

#[contracttype]
pub enum DataKey {
    PoolCounter,
    AllPoolIds,
    PoolSummary(BytesN<32>),
    LbpPool(BytesN<32>),
    BondingPool(BytesN<32>),
    ClPool(BytesN<32>),
    IssuerPools(Address),
    Initialized,
    LbpContract,
    BondingContract,
    ClContract,
    FairLaunchContract,
    OracleContract,
}

#[contract]
pub struct PoolFactory;

#[contractimpl]
impl PoolFactory {
    pub fn init(
        env: Env,
        admin: Address,
        lbp_contract: Address,
        bonding_contract: Address,
        cl_contract: Address,
        fair_launch_contract: Address,
        oracle_contract: Address,
    ) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&DataKey::LbpContract, &lbp_contract);
        env.storage()
            .instance()
            .set(&DataKey::BondingContract, &bonding_contract);
        env.storage()
            .instance()
            .set(&DataKey::ClContract, &cl_contract);
        env.storage()
            .instance()
            .set(&DataKey::FairLaunchContract, &fair_launch_contract);
        env.storage()
            .instance()
            .set(&DataKey::OracleContract, &oracle_contract);
    }

    pub fn create_lbp_pool(env: Env, issuer: Address, config: LbpConfig) -> BytesN<32> {
        unimplemented!()
    }

    pub fn create_bonding_pool(env: Env, issuer: Address, config: BondingConfig) -> BytesN<32> {
        unimplemented!()
    }

    pub fn create_cl_pool(env: Env, issuer: Address, config: ClConfig) -> BytesN<32> {
        unimplemented!()
    }

    pub fn get_pools_for_asset(env: Env, rwa_token: Address) -> Vec<PoolSummary> {
        unimplemented!()
    }

    pub fn list_all_pools(env: Env, offset: u32, limit: u32) -> Vec<PoolSummary> {
        unimplemented!()
    }

    pub fn get_pool_summary(env: Env, pool_id: BytesN<32>) -> PoolSummary {
        unimplemented!()
    }

    pub fn get_lbp_pool(env: Env, pool_id: BytesN<32>) -> LbpPool {
        unimplemented!()
    }

    pub fn get_bonding_pool(env: Env, pool_id: BytesN<32>) -> BondingCurvePool {
        unimplemented!()
    }

    pub fn mark_pool_graduated(env: Env, pool_id: BytesN<32>) {
        unimplemented!()
    }

    pub fn get_fair_launch_contract(env: Env) -> Address {
        unimplemented!()
    }

    pub fn get_oracle_contract(env: Env) -> Address {
        unimplemented!()
    }

    fn generate_pool_id(env: &Env) -> BytesN<32> {
        unimplemented!()
    }
}
