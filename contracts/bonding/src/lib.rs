use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use amm_math::BondingConfig;
use amm_math::BondingCurvePool;

fn get_price_linear(pool: &BondingCurvePool) -> u128 {
    unimplemented!()
}

fn get_price_polynomial(pool: &BondingCurvePool) -> u128 {
    unimplemented!()
}

fn get_price_sigmoid(pool: &BondingCurvePool) -> u128 {
    unimplemented!()
}

fn get_price_logarithmic(pool: &BondingCurvePool) -> u128 {
    unimplemented!()
}

fn get_price(pool: &BondingCurvePool) -> u128 {
    unimplemented!()
}

fn calculate_purchase_cost(pool: &BondingCurvePool, token_amount: u128) -> u128 {
    unimplemented!()
}

fn calculate_tokens_for_usdc(pool: &BondingCurvePool, usdc_in: u128) -> u128 {
    unimplemented!()
}

#[contract]
pub struct BondingCurveContract;

#[contractimpl]
impl BondingCurveContract {
    pub fn initialize(env: Env, pool_id: BytesN<32>, config: BondingConfig, issuer: Address) {
        unimplemented!()
    }

    pub fn buy(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        usdc_in: u128,
        min_tokens: u128,
    ) -> u128 {
        unimplemented!()
    }

    pub fn sell(
        env: Env,
        seller: Address,
        pool_id: BytesN<32>,
        token_amount: u128,
        min_usdc_out: u128,
    ) -> u128 {
        unimplemented!()
    }

    pub fn get_pool_details(env: Env, pool_id: BytesN<32>) -> BondingCurvePool {
        unimplemented!()
    }
}
