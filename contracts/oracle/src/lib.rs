use soroban_sdk::{contract, contractimpl, contracttype, BytesN, Env, Vec};

use amm_math::PriceObservation;

const MAX_OBSERVATIONS: u32 = 1000;
const DEFAULT_MAX_AGE: u64 = 86400;

#[contracttype]
pub enum OracleDataKey {
    Observations(BytesN<32>),
    LatestPrice(BytesN<32>),
    MaxAge(BytesN<32>),
}

#[contract]
pub struct TwapOracle;

#[contractimpl]
impl TwapOracle {
    pub fn initialize(env: Env, pool_id: BytesN<32>) {
        if env.storage().persistent().has(&OracleDataKey::Observations(pool_id.clone())) {
            panic!("already initialized");
        }
        env.storage().persistent().set(&OracleDataKey::Observations(pool_id.clone()), &Vec::<PriceObservation>::new(&env));
        env.storage().persistent().set(&OracleDataKey::LatestPrice(pool_id.clone()), &0u128);
        env.storage().persistent().set(&OracleDataKey::MaxAge(pool_id), &DEFAULT_MAX_AGE);
    }

    pub fn record_observation(env: Env, pool_id: BytesN<32>, price: u128) {
        let now = env.ledger().timestamp();
        let mut observations: Vec<PriceObservation> = env
            .storage()
            .persistent()
            .get(&OracleDataKey::Observations(pool_id.clone()))
            .unwrap();

        let cumulative = if observations.len() > 0 {
            let last = observations.get(observations.len() - 1).unwrap();
            let time_elapsed = (now.saturating_sub(last.timestamp)) as u128;
            last.cumulative_price.saturating_add(price.saturating_mul(time_elapsed))
        } else {
            0
        };

        let obs = PriceObservation {
            timestamp: now,
            price,
            cumulative_price: cumulative,
        };
        observations.push_back(obs);

        let max_age: u64 = env
            .storage()
            .persistent()
            .get(&OracleDataKey::MaxAge(pool_id.clone()))
            .unwrap_or(DEFAULT_MAX_AGE);
        let cutoff = now.saturating_sub(max_age);
        while observations.len() > 1 {
            let first = observations.get(0).unwrap();
            if first.timestamp < cutoff {
                observations.remove(0);
            } else {
                break;
            }
        }

        while observations.len() > MAX_OBSERVATIONS {
            observations.remove(0);
        }

        env.storage()
            .persistent()
            .set(&OracleDataKey::Observations(pool_id.clone()), &observations);
        env.storage()
            .persistent()
            .set(&OracleDataKey::LatestPrice(pool_id), &price);
    }

    pub fn get_twap(env: Env, pool_id: BytesN<32>, period_seconds: u64) -> u128 {
        let observations: Vec<PriceObservation> = env
            .storage()
            .persistent()
            .get(&OracleDataKey::Observations(pool_id))
            .unwrap_or(Vec::new(&env));
        let now = env.ledger().timestamp();
        let cutoff = now.saturating_sub(period_seconds);

        if observations.len() < 2 {
            return if observations.len() == 1 {
                observations.get(0).unwrap().price
            } else {
                0
            };
        }

        let latest = observations.get(observations.len() - 1).unwrap();

        let mut earliest_idx = 0u32;
        for i in 0..observations.len() {
            let obs = observations.get(i).unwrap();
            if obs.timestamp >= cutoff {
                break;
            }
            earliest_idx = i;
        }

        let earliest = observations.get(earliest_idx).unwrap();
        let time_diff = latest.timestamp.saturating_sub(earliest.timestamp);
        if time_diff == 0 {
            return latest.price;
        }

        let cum_diff = latest
            .cumulative_price
            .saturating_sub(earliest.cumulative_price);
        cum_diff / (time_diff as u128)
    }

    pub fn get_latest_price(env: Env, pool_id: BytesN<32>) -> u128 {
        env.storage()
            .persistent()
            .get(&OracleDataKey::LatestPrice(pool_id))
            .unwrap_or(0)
    }

    pub fn get_price_history(
        env: Env,
        pool_id: BytesN<32>,
        from_timestamp: u64,
        limit: u32,
    ) -> Vec<PriceObservation> {
        let observations: Vec<PriceObservation> = env
            .storage()
            .persistent()
            .get(&OracleDataKey::Observations(pool_id))
            .unwrap_or(Vec::new(&env));
        let mut result: Vec<PriceObservation> = Vec::new(&env);
        let mut count = 0u32;
        for i in 0..observations.len() {
            let obs = observations.get(i).unwrap();
            if obs.timestamp >= from_timestamp && count < limit {
                result.push_back(obs);
                count += 1;
            }
        }
        result
    }
}
