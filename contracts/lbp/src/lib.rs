use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

use amm_math::{fixed_div, fixed_mul, fixed_pow, LbpConfig, LbpPool, SCALE};

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
        _issuer: Address,
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
            compliance_contract: config.compliance_contract.clone(),
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
        let pool: LbpPool = env
            .storage()
            .persistent()
            .get(&LbpDataKey::Pool(pool_id.clone()))
            .unwrap();
        Self::compute_in_given_out(&pool, rwa_out)
    }

    pub fn calculate_out_given_in(env: Env, pool_id: BytesN<32>, usdc_in: u128) -> u128 {
        let pool: LbpPool = env
            .storage()
            .persistent()
            .get(&LbpDataKey::Pool(pool_id.clone()))
            .unwrap();
        Self::compute_out_given_in(&pool, usdc_in)
    }

    pub fn buy(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        min_rwa_out: u128,
        max_usdc_in: u128,
    ) -> u128 {
        buyer.require_auth();

        let mut pool: LbpPool = env
            .storage()
            .persistent()
            .get(&LbpDataKey::Pool(pool_id.clone()))
            .unwrap();

        if !pool.is_active || pool.graduated {
            panic!("pool is not active");
        }

        let rwa_out = Self::compute_out_given_in(&pool, max_usdc_in);

        if rwa_out < min_rwa_out {
            panic!("slippage: rwa_out below minimum");
        }

        let fair_launch: Address = env
            .storage()
            .instance()
            .get(&LbpDataKey::FairLaunchContract)
            .unwrap();
        let mut fl_args: Vec<Val> = Vec::new(&env);
        fl_args.push_back(buyer.clone().into_val(&env));
        fl_args.push_back(pool_id.clone().into_val(&env));
        fl_args.push_back(rwa_out.into_val(&env));
        env.invoke_contract::<()>(
            &fair_launch,
            &Symbol::new(&env, "check_purchase_allowed"),
            fl_args,
        );

        let mut fl_record_args: Vec<Val> = Vec::new(&env);
        fl_record_args.push_back(buyer.clone().into_val(&env));
        fl_record_args.push_back(pool_id.clone().into_val(&env));
        fl_record_args.push_back(rwa_out.into_val(&env));
        fl_record_args.push_back(max_usdc_in.into_val(&env));
        env.invoke_contract::<()>(
            &fair_launch,
            &Symbol::new(&env, "record_purchase"),
            fl_record_args,
        );

        let usdc_client = token::Client::new(&env, &pool.rwa_token);
        usdc_client.transfer(&buyer, &env.current_contract_address(), &(max_usdc_in as i128));

        let rwa_client = token::Client::new(&env, &pool.rwa_token);
        rwa_client.transfer(
            &env.current_contract_address(),
            &buyer,
            &(rwa_out as i128),
        );

        pool.balance_usdc = pool.balance_usdc.saturating_add(max_usdc_in);
        pool.balance_rwa = pool.balance_rwa.saturating_sub(rwa_out);
        pool.total_usdc_raised = pool.total_usdc_raised.saturating_add(max_usdc_in);
        env.storage()
            .persistent()
            .set(&LbpDataKey::Pool(pool_id.clone()), &pool);

        let oracle: Address = env.storage().instance().get(&LbpDataKey::OracleContract).unwrap();
        let spot_price = Self::compute_spot_price(&env, &pool);
        let mut obs_args: Vec<Val> = Vec::new(&env);
        obs_args.push_back(pool_id.clone().into_val(&env));
        obs_args.push_back(spot_price.into_val(&env));
        env.invoke_contract::<()>(&oracle, &Symbol::new(&env, "record_observation"), obs_args);

        env.events().publish(
            ("LbpPool", "Swap"),
            (buyer, pool_id, rwa_out, max_usdc_in, spot_price),
        );

        rwa_out
    }

    pub fn get_balance(env: Env, pool_id: BytesN<32>) -> (u128, u128) {
        let pool: LbpPool = env
            .storage()
            .persistent()
            .get(&LbpDataKey::Pool(pool_id))
            .unwrap();
        (pool.balance_rwa, pool.balance_usdc)
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
        let decay = fixed_mul(weight_diff, fixed_div(elapsed as u128, duration as u128));
        pool.weight_rwa_start.saturating_sub(decay)
    }

    fn compute_spot_price(env: &Env, pool: &LbpPool) -> u128 {
        let w_rwa = Self::compute_current_weight_rwa(env, pool);
        let w_usdc = SCALE.saturating_sub(w_rwa);

        if pool.balance_rwa == 0 || w_usdc == 0 {
            return 0;
        }

        let numerator = fixed_mul(pool.balance_usdc, w_rwa);
        let denominator = fixed_mul(pool.balance_rwa, w_usdc);
        if denominator == 0 {
            return 0;
        }
        fixed_div(numerator, denominator)
    }

    fn compute_in_given_out(pool: &LbpPool, rwa_out: u128) -> u128 {
        if rwa_out >= pool.balance_rwa {
            panic!("insufficient liquidity");
        }

        let w_rwa = pool.weight_rwa_start;
        let w_usdc = SCALE.saturating_sub(w_rwa);

        let ratio = fixed_div(pool.balance_rwa, pool.balance_rwa.saturating_sub(rwa_out));
        let exponent = fixed_div(w_rwa, w_usdc);
        let power = fixed_pow(ratio, exponent);

        let usdc_in = fixed_mul(pool.balance_usdc, power.saturating_sub(SCALE));
        let fee = fixed_mul(usdc_in, pool.swap_fee) / 10000;
        usdc_in.saturating_add(fee)
    }

    fn compute_out_given_in(pool: &LbpPool, usdc_in: u128) -> u128 {
        let w_rwa = pool.weight_rwa_start;
        let w_usdc = SCALE.saturating_sub(w_rwa);

        let fee = fixed_mul(usdc_in, pool.swap_fee) / 10000;
        let usdc_in_after_fee = usdc_in.saturating_sub(fee);

        let ratio = fixed_div(pool.balance_usdc, pool.balance_usdc.saturating_add(usdc_in_after_fee));
        let exponent = fixed_div(w_usdc, w_rwa);
        let power = fixed_pow(ratio, exponent);

        let out = fixed_mul(pool.balance_rwa, SCALE.saturating_sub(power));
        if out >= pool.balance_rwa {
            pool.balance_rwa
        } else {
            out
        }
    }
}

#[cfg(test)]
mod test;
