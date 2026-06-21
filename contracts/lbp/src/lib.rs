use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use amm_math::LbpConfig;
use amm_math::LbpPool;

#[contract]
pub struct LbpPoolContract;

#[contractimpl]
impl LbpPoolContract {
    pub fn initialize(env: Env, pool_id: BytesN<32>, config: LbpConfig, issuer: Address) {
        unimplemented!()
    }

    pub fn get_current_weight_rwa(env: Env, pool_id: BytesN<32>) -> u128 {
        unimplemented!()
    }

    pub fn get_spot_price(env: Env, pool_id: BytesN<32>) -> u128 {
        unimplemented!()
    }

    pub fn get_pool_details(env: Env, pool_id: BytesN<32>) -> LbpPool {
        unimplemented!()
    }

    pub fn calculate_in_given_out(env: Env, pool_id: BytesN<32>, rwa_out: u128) -> u128 {
        unimplemented!()
    }

    pub fn calculate_out_given_in(env: Env, pool_id: BytesN<32>, usdc_in: u128) -> u128 {
        unimplemented!()
    }

    pub fn buy(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        min_rwa_out: u128,
        max_usdc_in: u128,
    ) -> u128 {
        unimplemented!()
    }

    pub fn get_balance(env: Env, pool_id: BytesN<32>) -> (u128, u128) {
        unimplemented!()
    }
}
