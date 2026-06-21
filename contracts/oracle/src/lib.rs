use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

use amm_math::PriceObservation;

#[contract]
pub struct TwapOracle;

#[contractimpl]
impl TwapOracle {
    pub fn initialize(env: Env, pool_id: BytesN<32>) {
        unimplemented!()
    }

    pub fn record_observation(env: Env, pool_id: BytesN<32>, price: u128) {
        unimplemented!()
    }

    pub fn get_twap(env: Env, pool_id: BytesN<32>, period_seconds: u64) -> u128 {
        unimplemented!()
    }

    pub fn get_latest_price(env: Env, pool_id: BytesN<32>) -> u128 {
        unimplemented!()
    }

    pub fn get_price_history(
        env: Env,
        pool_id: BytesN<32>,
        from_timestamp: u64,
        limit: u32,
    ) -> Vec<PriceObservation> {
        unimplemented!()
    }
}
