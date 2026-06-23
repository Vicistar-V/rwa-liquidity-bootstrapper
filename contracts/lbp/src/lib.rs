use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env,
};

use amm_math::{LbpConfig, LbpPool, SCALE};

#[contracttype]
pub enum LbpDataKey {
    Pool(BytesN<32>),
    FairLaunchContract,
    OracleContract,
}

#[contract]
pub struct LbpPoolContract;

#[contractimpl]
impl LbpPoolContract {
    pub fn initialize(
        env: Env,
        pool_id: BytesN<32>,
        config: LbpConfig,
        issuer: Address,
        fair_launch_contract: Address,
        oracle_contract: Address,
    ) {
        if !env.storage().instance().has(&LbpDataKey::FairLaunchContract) {
            env.storage()
                .instance()
                .set(&LbpDataKey::FairLaunchContract, &fair_launch_contract);
            env.storage()
                .instance()
                .set(&LbpDataKey::OracleContract, &oracle_contract);
        }

        if env.storage().persistent().has(&LbpDataKey::Pool(pool_id.clone())) {
            panic!("pool already exists");
        }

        let pool = LbpPool {
            pool_id: pool_id.clone(),
            rwa_token: config.rwa_token.clone(),
            usdc_token: config.rwa_token.clone(),
            weight_rwa_start: config.weight_rwa_start,
            weight_rwa_end: config.weight_rwa_end,
            start_time: config.start_time,
            end_time: config.end_time,
            balance_rwa: config.rwa_amount,
            balance_usdc: 0,
            swap_fee: config.swap_fee_bps as u128,
            purchase_cap_per_wallet: config.purchase_cap_per_wallet.unwrap_or(u128::MAX),
            kyc_required: config.kyc_required,
            compliance_contract: config.compliance_contract,
            min_holding_period: config.min_holding_period,
            total_usdc_raised: 0,
            is_active: true,
            graduated: false,
        };

        env.storage()
            .persistent()
            .set(&LbpDataKey::Pool(pool_id), &pool);
    }

    pub fn get_current_weight_rwa(env: Env, pool_id: BytesN<32>) -> u128 {
        let pool: LbpPool = env
            .storage()
            .persistent()
            .get(&LbpDataKey::Pool(pool_id.clone()))
            .unwrap();
        Self::compute_current_weight_rwa(&env, &pool)
    }

    pub fn get_spot_price(env: Env, pool_id: BytesN<32>) -> u128 {
        let pool: LbpPool = env
            .storage()
            .persistent()
            .get(&LbpDataKey::Pool(pool_id.clone()))
            .unwrap();
        Self::compute_spot_price(&env, &pool)
    }

    pub fn get_pool_details(env: Env, pool_id: BytesN<32>) -> LbpPool {
        env.storage()
            .persistent()
            .get(&LbpDataKey::Pool(pool_id))
            .unwrap()
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

    fn compute_current_weight_rwa(env: &Env, pool: &LbpPool) -> u128 {
        let now = env.ledger().timestamp();
        let duration = pool.end_time.saturating_sub(pool.start_time);

        if duration == 0 || now >= pool.end_time {
            return pool.weight_rwa_end;
        }
        if now <= pool.start_time {
            return pool.weight_rwa_start;
        }

        let elapsed = now.saturating_sub(pool.start_time);
        let weight_diff = pool.weight_rwa_start.saturating_sub(pool.weight_rwa_end);
        let decay = amm_math::fixed_mul(
            weight_diff,
            amm_math::fixed_div(elapsed as u128, duration as u128),
        );
        pool.weight_rwa_start.saturating_sub(decay)
    }

    fn compute_spot_price(env: &Env, pool: &LbpPool) -> u128 {
        let w_rwa = Self::compute_current_weight_rwa(env, pool);
        let w_usdc = SCALE.saturating_sub(w_rwa);

        if pool.balance_rwa == 0 || w_usdc == 0 {
            return 0;
        }

        let numerator = amm_math::fixed_mul(pool.balance_usdc, w_rwa);
        let denominator = amm_math::fixed_mul(pool.balance_rwa, w_usdc);
        if denominator == 0 {
            return 0;
        }
        amm_math::fixed_div(numerator, denominator)
    }
}
