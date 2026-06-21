use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use amm_math::{ClConfig, LpPosition};

#[contract]
pub struct ConcentratedLiquidityContract;

#[contractimpl]
impl ConcentratedLiquidityContract {
    pub fn initialize(env: Env, pool_id: BytesN<32>, config: ClConfig) {
        unimplemented!()
    }

    pub fn mint_position(
        env: Env,
        lp: Address,
        pool_id: BytesN<32>,
        tick_lower: i32,
        tick_upper: i32,
        liquidity: u128,
    ) {
        unimplemented!()
    }

    pub fn burn_position(
        env: Env,
        lp: Address,
        pool_id: BytesN<32>,
        position_id: BytesN<32>,
    ) {
        unimplemented!()
    }

    pub fn collect_fees(
        env: Env,
        lp: Address,
        pool_id: BytesN<32>,
        position_id: BytesN<32>,
    ) -> (u128, u128) {
        unimplemented!()
    }

    pub fn get_position(
        env: Env,
        pool_id: BytesN<32>,
        position_id: BytesN<32>,
    ) -> LpPosition {
        unimplemented!()
    }

    pub fn sqrt_price_to_tick(price: u128) -> i32 {
        unimplemented!()
    }

    pub fn tick_to_sqrt_price(tick: i32) -> u128 {
        unimplemented!()
    }

    pub fn get_amounts_for_liquidity(
        liquidity: u128,
        sqrt_price: u128,
        sqrt_low: u128,
        sqrt_high: u128,
    ) -> (u128, u128) {
        unimplemented!()
    }
}
