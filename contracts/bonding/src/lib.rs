use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

use amm_math::{
    fixed_div, fixed_ln, fixed_mul, fixed_pow, integral_logarithmic, sigmoid,
    BondingConfig, BondingCurvePool, CurveType, SCALE,
};

#[contracttype]
pub enum BondingDataKey {
    Pool(BytesN<32>),
    FairLaunchContract,
    OracleContract,
}

fn get_price_linear(pool: &BondingCurvePool) -> u128 {
    let price = pool
        .curve_coefficient_a
        .saturating_mul(pool.current_supply)
        / SCALE
        + pool.curve_coefficient_b;
    if price > pool.price_ceiling {
        pool.price_ceiling
    } else {
        price
    }
}

fn get_price_polynomial(pool: &BondingCurvePool) -> u128 {
    let s_pow_n = fixed_pow(pool.current_supply, pool.curve_exponent_n);
    let price = fixed_mul(pool.curve_coefficient_a, s_pow_n).saturating_add(pool.curve_coefficient_b);
    if price > pool.price_ceiling {
        pool.price_ceiling
    } else {
        price
    }
}

fn get_price_sigmoid(pool: &BondingCurvePool) -> u128 {
    let mid = pool.max_supply / 2;
    let price = sigmoid(
        pool.current_supply,
        pool.curve_coefficient_a,
        mid,
        pool.price_ceiling,
    )
    .saturating_add(pool.curve_coefficient_b);
    if price > pool.price_ceiling {
        pool.price_ceiling
    } else {
        price
    }
}

fn get_price_logarithmic(pool: &BondingCurvePool) -> u128 {
    let ln_val = fixed_ln(pool.current_supply.saturating_add(SCALE));
    let price = fixed_mul(pool.curve_coefficient_a, ln_val).saturating_add(pool.curve_coefficient_b);
    if price > pool.price_ceiling {
        pool.price_ceiling
    } else {
        price
    }
}

fn get_price(pool: &BondingCurvePool) -> u128 {
    match pool.curve_type {
        CurveType::Linear => get_price_linear(pool),
        CurveType::Polynomial => get_price_polynomial(pool),
        CurveType::Sigmoid => get_price_sigmoid(pool),
        CurveType::Logarithmic => get_price_logarithmic(pool),
    }
}

fn calculate_purchase_cost(pool: &BondingCurvePool, token_amount: u128) -> u128 {
    let s1 = pool.current_supply;
    let s2 = s1.saturating_add(token_amount);

    match pool.curve_type {
        CurveType::Linear => {
            let integral = fixed_mul(pool.curve_coefficient_a, (s2.saturating_mul(s2) - s1.saturating_mul(s1)) / 2 / SCALE);
            let linear = fixed_mul(pool.curve_coefficient_b, token_amount);
            integral.saturating_add(linear)
        }
        CurveType::Polynomial => {
            let n = pool.curve_exponent_n;
            let s2_pow = fixed_pow(s2, n.saturating_add(SCALE));
            let s1_pow = fixed_pow(s1, n.saturating_add(SCALE));
            let integral_coeff = fixed_div(pool.curve_coefficient_a, n.saturating_add(SCALE));
            let integral = fixed_mul(integral_coeff, s2_pow.saturating_sub(s1_pow));
            let linear = fixed_mul(pool.curve_coefficient_b, token_amount);
            integral.saturating_add(linear)
        }
        CurveType::Logarithmic => {
            let int1 = integral_logarithmic(pool.curve_coefficient_a, pool.curve_coefficient_b, s2);
            let int2 = integral_logarithmic(pool.curve_coefficient_a, pool.curve_coefficient_b, s1);
            int1.saturating_sub(int2)
        }
        CurveType::Sigmoid => {
            let steps = 100u128;
            let step_size = token_amount / steps;
            let mut cost: u128 = 0;
            for _i in 0..100 {
                let price = get_price_sigmoid(pool);
                cost = cost.saturating_add(fixed_mul(price, step_size));
            }
            let remainder = token_amount.saturating_sub(step_size.saturating_mul(100));
            if remainder > 0 {
                let price = get_price_sigmoid(pool);
                cost = cost.saturating_add(fixed_mul(price, remainder));
            }
            cost
        }
    }
}

fn calculate_tokens_for_usdc(pool: &BondingCurvePool, usdc_in: u128) -> u128 {
    let max_buyable = pool.max_supply.saturating_sub(pool.current_supply);
    let mut low: u128 = 0;
    let mut high: u128 = max_buyable;

    for _ in 0..64 {
        let mid = low.saturating_add(high) / 2;
        let cost = calculate_purchase_cost(pool, mid);
        if cost < usdc_in {
            low = mid;
        } else {
            high = mid;
        }
    }

    let final_cost = calculate_purchase_cost(pool, high);
    if final_cost <= usdc_in {
        high
    } else {
        low
    }
}

fn calculate_sale_return(pool: &BondingCurvePool, token_amount: u128) -> u128 {
    let sell_amount = if token_amount > pool.current_supply {
        pool.current_supply
    } else {
        token_amount
    };

    let mut modified_pool = pool.clone();
    modified_pool.current_supply = modified_pool.current_supply.saturating_sub(sell_amount);
    let cost = calculate_purchase_cost(&modified_pool, sell_amount);

    cost
}

#[contract]
pub struct BondingCurveContract;

#[contractimpl]
impl BondingCurveContract {
    pub fn initialize(
        env: Env,
        pool_id: BytesN<32>,
        config: BondingConfig,
        _issuer: Address,
        fair_launch_contract: Address,
        oracle_contract: Address,
    ) {
        if !env
            .storage()
            .instance()
            .has(&BondingDataKey::FairLaunchContract)
        {
            env.storage()
                .instance()
                .set(&BondingDataKey::FairLaunchContract, &fair_launch_contract);
            env.storage()
                .instance()
                .set(&BondingDataKey::OracleContract, &oracle_contract);
        }

        if env
            .storage()
            .persistent()
            .has(&BondingDataKey::Pool(pool_id.clone()))
        {
            panic!("pool already exists");
        }

        let pool = BondingCurvePool {
            pool_id: pool_id.clone(),
            rwa_token: config.rwa_token.clone(),
            reserve_token: config.rwa_token.clone(),
            curve_type: config.curve_type.clone(),
            curve_coefficient_a: config.coefficient_a,
            curve_coefficient_b: config.coefficient_b,
            curve_exponent_n: 1,
            max_supply: config.max_supply,
            price_ceiling: config.price_ceiling,
            current_supply: 0,
            reserve_balance: 0,
            is_active: true,
            graduated: false,
            purchase_cap_per_wallet: config.purchase_cap_per_wallet.unwrap_or(u128::MAX),
            kyc_required: config.kyc_required,
            compliance_contract: config.compliance_contract.clone(),
        };

        env.storage()
            .persistent()
            .set(&BondingDataKey::Pool(pool_id), &pool);
    }

    pub fn buy(env: Env, buyer: Address, pool_id: BytesN<32>, usdc_in: u128, min_tokens: u128) -> u128 {
        buyer.require_auth();

        let mut pool: BondingCurvePool = env
            .storage()
            .persistent()
            .get(&BondingDataKey::Pool(pool_id.clone()))
            .unwrap();

        if !pool.is_active || pool.graduated {
            panic!("pool is not active");
        }

        let tokens_out = calculate_tokens_for_usdc(&pool, usdc_in);

        if tokens_out < min_tokens {
            panic!("slippage: tokens below minimum");
        }

        if pool.current_supply.saturating_add(tokens_out) > pool.max_supply {
            panic!("max supply exceeded");
        }

        let fair_launch: Address = env
            .storage()
            .instance()
            .get(&BondingDataKey::FairLaunchContract)
            .unwrap();
        let mut fl_args: Vec<Val> = Vec::new(&env);
        fl_args.push_back(buyer.clone().into_val(&env));
        fl_args.push_back(pool_id.clone().into_val(&env));
        fl_args.push_back(tokens_out.into_val(&env));
        env.invoke_contract::<()>(
            &fair_launch,
            &Symbol::new(&env, "check_purchase_allowed"),
            fl_args,
        );

        let mut fl_record_args: Vec<Val> = Vec::new(&env);
        fl_record_args.push_back(buyer.clone().into_val(&env));
        fl_record_args.push_back(pool_id.clone().into_val(&env));
        fl_record_args.push_back(tokens_out.into_val(&env));
        fl_record_args.push_back(usdc_in.into_val(&env));
        env.invoke_contract::<()>(
            &fair_launch,
            &Symbol::new(&env, "record_purchase"),
            fl_record_args,
        );

        let reserve_client = token::Client::new(&env, &pool.rwa_token);
        reserve_client.transfer(&buyer, &env.current_contract_address(), &(usdc_in as i128));

        let rwa_client = token::Client::new(&env, &pool.rwa_token);
        rwa_client.transfer(
            &env.current_contract_address(),
            &buyer,
            &(tokens_out as i128),
        );

        pool.current_supply = pool.current_supply.saturating_add(tokens_out);
        pool.reserve_balance = pool.reserve_balance.saturating_add(usdc_in);
        env.storage()
            .persistent()
            .set(&BondingDataKey::Pool(pool_id.clone()), &pool);

        let oracle: Address = env
            .storage()
            .instance()
            .get(&BondingDataKey::OracleContract)
            .unwrap();
        let spot_price = get_price(&pool);
        let mut obs_args: Vec<Val> = Vec::new(&env);
        obs_args.push_back(pool_id.clone().into_val(&env));
        obs_args.push_back(spot_price.into_val(&env));
        env.invoke_contract::<()>(
            &oracle,
            &Symbol::new(&env, "record_observation"),
            obs_args,
        );

        env.events().publish(
            ("BondingCurve", "Buy"),
            (buyer, pool_id, tokens_out, usdc_in, spot_price),
        );

        tokens_out
    }

    pub fn sell(
        env: Env,
        seller: Address,
        pool_id: BytesN<32>,
        token_amount: u128,
        min_usdc_out: u128,
    ) -> u128 {
        seller.require_auth();

        let mut pool: BondingCurvePool = env
            .storage()
            .persistent()
            .get(&BondingDataKey::Pool(pool_id.clone()))
            .unwrap();

        if !pool.is_active || pool.graduated {
            panic!("pool is not active");
        }

        let usdc_out = calculate_sale_return(&pool, token_amount);

        if usdc_out < min_usdc_out {
            panic!("slippage: usdc below minimum");
        }

        if usdc_out > pool.reserve_balance {
            panic!("insufficient reserve");
        }

        let rwa_client = token::Client::new(&env, &pool.rwa_token);
        rwa_client.transfer(
            &seller,
            &env.current_contract_address(),
            &(token_amount as i128),
        );

        let reserve_client = token::Client::new(&env, &pool.rwa_token);
        reserve_client.transfer(
            &env.current_contract_address(),
            &seller,
            &(usdc_out as i128),
        );

        pool.current_supply = pool.current_supply.saturating_sub(token_amount);
        pool.reserve_balance = pool.reserve_balance.saturating_sub(usdc_out);
        env.storage()
            .persistent()
            .set(&BondingDataKey::Pool(pool_id.clone()), &pool);

        let oracle: Address = env
            .storage()
            .instance()
            .get(&BondingDataKey::OracleContract)
            .unwrap();
        let spot_price = get_price(&pool);
        let mut obs_args: Vec<Val> = Vec::new(&env);
        obs_args.push_back(pool_id.clone().into_val(&env));
        obs_args.push_back(spot_price.into_val(&env));
        env.invoke_contract::<()>(
            &oracle,
            &Symbol::new(&env, "record_observation"),
            obs_args,
        );

        env.events().publish(
            ("BondingCurve", "Sell"),
            (seller, pool_id, token_amount, usdc_out, spot_price),
        );

        usdc_out
    }

    pub fn get_price(env: Env, pool_id: BytesN<32>) -> u128 {
        let pool: BondingCurvePool = env
            .storage()
            .persistent()
            .get(&BondingDataKey::Pool(pool_id))
            .unwrap();
        get_price(&pool)
    }

    pub fn get_pool_details(env: Env, pool_id: BytesN<32>) -> BondingCurvePool {
        env.storage()
            .persistent()
            .get(&BondingDataKey::Pool(pool_id))
            .unwrap()
    }
}
