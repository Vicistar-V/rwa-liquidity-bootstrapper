use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

use amm_math::{BondingConfig, ClConfig, LbpConfig, PoolSummary};

#[contract]
pub struct PoolFactory;

#[contractimpl]
impl PoolFactory {
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
}
