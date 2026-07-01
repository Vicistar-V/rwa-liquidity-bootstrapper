use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, BytesN, Env,
};

use amm_math::{
    fixed_div, fixed_ln, fixed_mul, fixed_pow, ClConfig, ConcentratedLiquidityPool, LpPosition,
    SCALE,
};

const SQRT_1_0001: u128 = 10_000_500;
const ONE_POINT_0001: u128 = 10_001_000;

#[contracttype]
pub enum ClDataKey {
    Pool(BytesN<32>),
    Position(BytesN<32>, BytesN<32>),
    PositionCounter(BytesN<32>),
}

fn sqrt_int(x: u128) -> u128 {
    if x == 0 {
        return 0;
    }
    let mut z = x.saturating_add(1) / 2;
    for _ in 0..200 {
        let next = z.saturating_add(x / z) / 2;
        if next >= z {
            break;
        }
        z = next;
    }
    z
}

fn price_to_sqrt_price(price: u128) -> u128 {
    sqrt_int(price.saturating_mul(SCALE))
}

#[contract]
pub struct ConcentratedLiquidityContract;

#[contractimpl]
impl ConcentratedLiquidityContract {
    pub fn initialize(env: Env, pool_id: BytesN<32>, config: ClConfig) {
        if env
            .storage()
            .persistent()
            .has(&ClDataKey::Pool(pool_id.clone()))
        {
            panic!("already initialized");
        }

        let now = env.ledger().timestamp();
        let pool = ConcentratedLiquidityPool {
            pool_id: pool_id.clone(),
            rwa_token: config.rwa_token.clone(),
            usdc_token: config.usdc_token,
            price_lower: config.price_lower,
            price_upper: config.price_upper,
            current_price: config.price_lower,
            tick_spacing: config.tick_spacing,
            fee_tier: config.fee_tier,
            total_liquidity: 0,
            liquidity_positions: soroban_sdk::Map::new(&env),
            price_accumulator: 0,
            last_observation_time: now,
        };

        env.storage()
            .persistent()
            .set(&ClDataKey::Pool(pool_id), &pool);
    }

    pub fn mint_position(
        env: Env,
        lp: Address,
        pool_id: BytesN<32>,
        tick_lower: i32,
        tick_upper: i32,
        liquidity: u128,
    ) -> BytesN<32> {
        lp.require_auth();

        let mut pool: ConcentratedLiquidityPool = env
            .storage()
            .persistent()
            .get(&ClDataKey::Pool(pool_id.clone()))
            .unwrap();

        let sqrt_price = price_to_sqrt_price(pool.current_price);
        let sqrt_low = Self::tick_to_sqrt_price(tick_lower);
        let sqrt_high = Self::tick_to_sqrt_price(tick_upper);
        let (amount_rwa, amount_usdc) =
            Self::get_amounts_for_liquidity(liquidity, sqrt_price, sqrt_low, sqrt_high);

        let counter: u64 = env
            .storage()
            .persistent()
            .get(&ClDataKey::PositionCounter(pool_id.clone()))
            .unwrap_or(0);
        let new_counter = counter.saturating_add(1);
        env.storage()
            .persistent()
            .set(&ClDataKey::PositionCounter(pool_id.clone()), &new_counter);

        let mut pos_id_arr = [0u8; 32];
        pos_id_arr[..8].copy_from_slice(&counter.to_be_bytes());
        pos_id_arr[8..16].copy_from_slice(&new_counter.to_be_bytes());
        let position_id = BytesN::from_array(&env, &pos_id_arr);

        let position = LpPosition {
            owner: lp.clone(),
            liquidity,
            tick_lower,
            tick_upper,
            fee_growth_inside_rwa: 0,
            fee_growth_inside_usdc: 0,
            tokens_owed_rwa: 0,
            tokens_owed_usdc: 0,
        };

        env.storage().persistent().set(
            &ClDataKey::Position(pool_id.clone(), position_id.clone()),
            &position,
        );

        pool.total_liquidity = pool.total_liquidity.saturating_add(liquidity);
        env.storage()
            .persistent()
            .set(&ClDataKey::Pool(pool_id.clone()), &pool);

        let rwa_client = token::Client::new(&env, &pool.rwa_token);
        if amount_rwa > 0 {
            rwa_client.transfer(
                &lp,
                &env.current_contract_address(),
                &(amount_rwa as i128),
            );
        }

        let usdc_client = token::Client::new(&env, &pool.usdc_token);
        if amount_usdc > 0 {
            usdc_client.transfer(
                &lp,
                &env.current_contract_address(),
                &(amount_usdc as i128),
            );
        }

        env.events().publish(
            ("CL", "PositionMinted"),
            (
                lp, pool_id, position_id.clone(), tick_lower, tick_upper,
                liquidity, amount_rwa, amount_usdc,
            ),
        );

        position_id
    }

    pub fn burn_position(env: Env, lp: Address, pool_id: BytesN<32>, position_id: BytesN<32>) {
        lp.require_auth();

        let mut pool: ConcentratedLiquidityPool = env
            .storage()
            .persistent()
            .get(&ClDataKey::Pool(pool_id.clone()))
            .unwrap();

        let position: LpPosition = env
            .storage()
            .persistent()
            .get(&ClDataKey::Position(pool_id.clone(), position_id.clone()))
            .unwrap();

        if position.owner != lp {
            panic!("not the position owner");
        }

        let sqrt_price = price_to_sqrt_price(pool.current_price);
        let sqrt_low = Self::tick_to_sqrt_price(position.tick_lower);
        let sqrt_high = Self::tick_to_sqrt_price(position.tick_upper);
        let (amount_rwa, amount_usdc) =
            Self::get_amounts_for_liquidity(position.liquidity, sqrt_price, sqrt_low, sqrt_high);

        let total_owed_rwa = position.tokens_owed_rwa.saturating_add(amount_rwa);
        let total_owed_usdc = position.tokens_owed_usdc.saturating_add(amount_usdc);

        env.storage().persistent().remove(&ClDataKey::Position(
            pool_id.clone(),
            position_id.clone(),
        ));

        pool.total_liquidity = pool.total_liquidity.saturating_sub(position.liquidity);
        env.storage()
            .persistent()
            .set(&ClDataKey::Pool(pool_id.clone()), &pool);

        let rwa_client = token::Client::new(&env, &pool.rwa_token);
        if total_owed_rwa > 0 {
            rwa_client.transfer(
                &env.current_contract_address(),
                &lp,
                &(total_owed_rwa as i128),
            );
        }

        let usdc_client = token::Client::new(&env, &pool.usdc_token);
        if total_owed_usdc > 0 {
            usdc_client.transfer(
                &env.current_contract_address(),
                &lp,
                &(total_owed_usdc as i128),
            );
        }

        env.events().publish(
            ("CL", "PositionBurned"),
            (lp, pool_id, position_id, total_owed_rwa, total_owed_usdc),
        );
    }

    pub fn collect_fees(
        env: Env,
        lp: Address,
        pool_id: BytesN<32>,
        position_id: BytesN<32>,
    ) -> (u128, u128) {
        lp.require_auth();

        let mut position: LpPosition = env
            .storage()
            .persistent()
            .get(&ClDataKey::Position(pool_id.clone(), position_id.clone()))
            .unwrap();

        if position.owner != lp {
            panic!("not the position owner");
        }

        let rwa_fees = position.tokens_owed_rwa;
        let usdc_fees = position.tokens_owed_usdc;
        position.tokens_owed_rwa = 0;
        position.tokens_owed_usdc = 0;

        env.storage().persistent().set(
            &ClDataKey::Position(pool_id.clone(), position_id),
            &position,
        );

        let pool: ConcentratedLiquidityPool = env
            .storage()
            .persistent()
            .get(&ClDataKey::Pool(pool_id))
            .unwrap();

        let rwa_client = token::Client::new(&env, &pool.rwa_token);
        if rwa_fees > 0 {
            rwa_client.transfer(
                &env.current_contract_address(),
                &lp,
                &(rwa_fees as i128),
            );
        }

        let usdc_client = token::Client::new(&env, &pool.usdc_token);
        if usdc_fees > 0 {
            usdc_client.transfer(
                &env.current_contract_address(),
                &lp,
                &(usdc_fees as i128),
            );
        }

        (rwa_fees, usdc_fees)
    }

    pub fn get_pool_details(env: Env, pool_id: BytesN<32>) -> ConcentratedLiquidityPool {
        env.storage()
            .persistent()
            .get(&ClDataKey::Pool(pool_id))
            .unwrap()
    }

    pub fn get_position(
        env: Env,
        pool_id: BytesN<32>,
        position_id: BytesN<32>,
    ) -> LpPosition {
        env.storage()
            .persistent()
            .get(&ClDataKey::Position(pool_id, position_id))
            .unwrap()
    }

    pub fn sqrt_price_to_tick(price: u128) -> i32 {
        if price == 0 {
            return 0;
        }
        let ln_price = fixed_ln(price);
        let ln_10001 = fixed_ln(ONE_POINT_0001);
        if ln_10001 == 0 {
            return 0;
        }
        let tick_raw = 2u128.saturating_mul(ln_price) / ln_10001;
        tick_raw as i32
    }

    pub fn tick_to_sqrt_price(tick: i32) -> u128 {
        if tick >= 0 {
            fixed_pow(SQRT_1_0001, (tick as u128).saturating_mul(SCALE))
        } else {
            let n = ((-tick) as u128).saturating_mul(SCALE);
            let pow_val = fixed_pow(SQRT_1_0001, n);
            if pow_val == 0 {
                u128::MAX
            } else {
                SCALE.saturating_mul(SCALE) / pow_val
            }
        }
    }

    pub fn get_amounts_for_liquidity(
        liquidity: u128,
        sqrt_price: u128,
        sqrt_low: u128,
        sqrt_high: u128,
    ) -> (u128, u128) {
        if sqrt_price <= sqrt_low {
            let amount0 = fixed_div(liquidity, sqrt_low)
                .saturating_sub(fixed_div(liquidity, sqrt_high));
            (amount0, 0)
        } else if sqrt_price >= sqrt_high {
            let amount1 = fixed_mul(liquidity, sqrt_high)
                .saturating_sub(fixed_mul(liquidity, sqrt_low));
            (0, amount1)
        } else {
            let amount0 = fixed_div(liquidity, sqrt_price)
                .saturating_sub(fixed_div(liquidity, sqrt_high));
            let amount1 = fixed_mul(liquidity, sqrt_price)
                .saturating_sub(fixed_mul(liquidity, sqrt_low));
            (amount0, amount1)
        }
    }
}

#[cfg(test)]
mod test;
