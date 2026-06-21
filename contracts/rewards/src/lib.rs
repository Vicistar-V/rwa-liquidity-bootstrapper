use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

#[contract]
pub struct LpRewards;

#[contractimpl]
impl LpRewards {
    pub fn initialize(env: Env, pool_id: BytesN<32>) {
        unimplemented!()
    }

    pub fn deposit_fees(env: Env, pool_id: BytesN<32>, amount_rwa: u128, amount_usdc: u128) {
        unimplemented!()
    }

    pub fn claim(env: Env, lp: Address, pool_id: BytesN<32>) -> (u128, u128) {
        unimplemented!()
    }

    pub fn get_rewards(env: Env, lp: Address, pool_id: BytesN<32>) -> (u128, u128) {
        unimplemented!()
    }
}
