use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

use amm_math::{
    BondingConfig, BondingCurvePool, ClConfig, ConcentratedLiquidityPool, LbpConfig, LbpPool,
    PoolSummary, PoolType,
};

#[contracttype]
pub enum DataKey {
    PoolCounter,
    AllPoolIds,
    PoolSummary(BytesN<32>),
    LbpPool(BytesN<32>),
    BondingPool(BytesN<32>),
    ClPool(BytesN<32>),
    IssuerPools(Address),
    Initialized,
    LbpContract,
    BondingContract,
    ClContract,
    FairLaunchContract,
    OracleContract,
}

#[contract]
pub struct PoolFactory;

#[contractimpl]
impl PoolFactory {
    pub fn init(
        env: Env,
        admin: Address,
        lbp_contract: Address,
        bonding_contract: Address,
        cl_contract: Address,
        fair_launch_contract: Address,
        oracle_contract: Address,
    ) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&DataKey::LbpContract, &lbp_contract);
        env.storage()
            .instance()
            .set(&DataKey::BondingContract, &bonding_contract);
        env.storage()
            .instance()
            .set(&DataKey::ClContract, &cl_contract);
        env.storage()
            .instance()
            .set(&DataKey::FairLaunchContract, &fair_launch_contract);
        env.storage()
            .instance()
            .set(&DataKey::OracleContract, &oracle_contract);
    }

    pub fn create_lbp_pool(env: Env, issuer: Address, config: LbpConfig) -> BytesN<32> {
        issuer.require_auth();
        let pool_id = Self::generate_pool_id(&env);
        let now = env.ledger().timestamp();

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

        let summary = PoolSummary {
            pool_id: pool_id.clone(),
            pool_type: PoolType::Lbp,
            rwa_token: config.rwa_token.clone(),
            is_active: true,
            graduated: false,
            total_usdc_raised: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PoolSummary(pool_id.clone()), &summary);
        env.storage()
            .persistent()
            .set(&DataKey::LbpPool(pool_id.clone()), &pool);

        let mut all_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::AllPoolIds)
            .unwrap_or_else(|| Vec::new(&env));
        all_ids.push_back(pool_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::AllPoolIds, &all_ids);

        let mut issuer_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::IssuerPools(issuer.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        issuer_ids.push_back(pool_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::IssuerPools(issuer.clone()), &issuer_ids);

        let token_client = token::Client::new(&env, &config.rwa_token);
        token_client.transfer(
            &issuer,
            &env.current_contract_address(),
            &(config.rwa_amount as i128),
        );

        let lbp_contract: Address = env.storage().instance().get(&DataKey::LbpContract).unwrap();
        let fair_launch: Address = env
            .storage()
            .instance()
            .get(&DataKey::FairLaunchContract)
            .unwrap();
        let oracle: Address = env.storage().instance().get(&DataKey::OracleContract).unwrap();

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(pool_id.clone().into_val(&env));
        args.push_back(config.into_val(&env));
        args.push_back(issuer.into_val(&env));
        args.push_back(fair_launch.into_val(&env));
        args.push_back(oracle.into_val(&env));
        env.invoke_contract::<()>(
            &lbp_contract,
            &Symbol::new(&env, "initialize"),
            args,
        );

        env.events().publish(
            ("PoolFactory", "PoolCreated"),
            (Symbol::new(&env, "LBP"), pool_id.clone(), config.rwa_token, now),
        );

        pool_id
    }

    pub fn create_bonding_pool(env: Env, issuer: Address, config: BondingConfig) -> BytesN<32> {
        issuer.require_auth();
        let pool_id = Self::generate_pool_id(&env);
        let now = env.ledger().timestamp();

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

        let summary = PoolSummary {
            pool_id: pool_id.clone(),
            pool_type: PoolType::BondingCurve,
            rwa_token: config.rwa_token.clone(),
            is_active: true,
            graduated: false,
            total_usdc_raised: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PoolSummary(pool_id.clone()), &summary);
        env.storage()
            .persistent()
            .set(&DataKey::BondingPool(pool_id.clone()), &pool);

        let mut all_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::AllPoolIds)
            .unwrap_or_else(|| Vec::new(&env));
        all_ids.push_back(pool_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::AllPoolIds, &all_ids);

        let mut issuer_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::IssuerPools(issuer.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        issuer_ids.push_back(pool_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::IssuerPools(issuer.clone()), &issuer_ids);

        let bonding_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::BondingContract)
            .unwrap();
        let fair_launch: Address = env
            .storage()
            .instance()
            .get(&DataKey::FairLaunchContract)
            .unwrap();
        let oracle: Address = env.storage().instance().get(&DataKey::OracleContract).unwrap();

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(pool_id.clone().into_val(&env));
        args.push_back(config.into_val(&env));
        args.push_back(issuer.into_val(&env));
        args.push_back(fair_launch.into_val(&env));
        args.push_back(oracle.into_val(&env));
        env.invoke_contract::<()>(
            &bonding_contract,
            &Symbol::new(&env, "initialize"),
            args,
        );

        env.events().publish(
            ("PoolFactory", "PoolCreated"),
            (
                Symbol::new(&env, "Bonding"),
                pool_id.clone(),
                config.rwa_token,
                now,
            ),
        );

        pool_id
    }

    pub fn create_cl_pool(env: Env, issuer: Address, config: ClConfig) -> BytesN<32> {
        issuer.require_auth();
        let pool_id = Self::generate_pool_id(&env);
        let now = env.ledger().timestamp();

        let pool = ConcentratedLiquidityPool {
            pool_id: pool_id.clone(),
            rwa_token: config.rwa_token.clone(),
            usdc_token: config.usdc_token.clone(),
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

        let summary = PoolSummary {
            pool_id: pool_id.clone(),
            pool_type: PoolType::ConcentratedLiquidity,
            rwa_token: config.rwa_token.clone(),
            is_active: true,
            graduated: false,
            total_usdc_raised: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::PoolSummary(pool_id.clone()), &summary);
        env.storage()
            .persistent()
            .set(&DataKey::ClPool(pool_id.clone()), &pool);

        let mut all_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::AllPoolIds)
            .unwrap_or_else(|| Vec::new(&env));
        all_ids.push_back(pool_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::AllPoolIds, &all_ids);

        let mut issuer_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::IssuerPools(issuer.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        issuer_ids.push_back(pool_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::IssuerPools(issuer), &issuer_ids);

        let cl_contract: Address = env.storage().instance().get(&DataKey::ClContract).unwrap();

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(pool_id.clone().into_val(&env));
        args.push_back(config.into_val(&env));
        env.invoke_contract::<()>(
            &cl_contract,
            &Symbol::new(&env, "initialize"),
            args,
        );

        env.events().publish(
            ("PoolFactory", "PoolCreated"),
            (Symbol::new(&env, "CL"), pool_id.clone(), config.rwa_token, now),
        );

        pool_id
    }

    pub fn get_pools_for_asset(env: Env, rwa_token: Address) -> Vec<PoolSummary> {
        let all_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::AllPoolIds)
            .unwrap_or_else(|| Vec::new(&env));
        let mut result: Vec<PoolSummary> = Vec::new(&env);
        for i in 0..all_ids.len() {
            let id = all_ids.get(i).unwrap();
            let summary: PoolSummary = env
                .storage()
                .persistent()
                .get(&DataKey::PoolSummary(id.clone()))
                .unwrap();
            if summary.rwa_token == rwa_token {
                result.push_back(summary);
            }
        }
        result
    }

    pub fn list_all_pools(env: Env, offset: u32, limit: u32) -> Vec<PoolSummary> {
        let all_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::AllPoolIds)
            .unwrap_or_else(|| Vec::new(&env));
        let mut result: Vec<PoolSummary> = Vec::new(&env);
        let start = offset;
        let end = (start + limit).min(all_ids.len());
        for i in start..end {
            let id = all_ids.get(i).unwrap();
            let summary: PoolSummary = env
                .storage()
                .persistent()
                .get(&DataKey::PoolSummary(id.clone()))
                .unwrap();
            result.push_back(summary);
        }
        result
    }

    pub fn get_pool_summary(env: Env, pool_id: BytesN<32>) -> PoolSummary {
        env.storage()
            .persistent()
            .get(&DataKey::PoolSummary(pool_id))
            .unwrap()
    }

    pub fn get_lbp_pool(env: Env, pool_id: BytesN<32>) -> LbpPool {
        env.storage()
            .persistent()
            .get(&DataKey::LbpPool(pool_id))
            .unwrap()
    }

    pub fn get_bonding_pool(env: Env, pool_id: BytesN<32>) -> BondingCurvePool {
        env.storage()
            .persistent()
            .get(&DataKey::BondingPool(pool_id))
            .unwrap()
    }

    pub fn mark_pool_graduated(env: Env, pool_id: BytesN<32>) {
        let mut summary: PoolSummary = env
            .storage()
            .persistent()
            .get(&DataKey::PoolSummary(pool_id.clone()))
            .unwrap();
        summary.graduated = true;
        summary.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::PoolSummary(pool_id.clone()), &summary);

        if env
            .storage()
            .persistent()
            .has(&DataKey::LbpPool(pool_id.clone()))
        {
            let mut pool: LbpPool = env
                .storage()
                .persistent()
                .get(&DataKey::LbpPool(pool_id.clone()))
                .unwrap();
            pool.is_active = false;
            pool.graduated = true;
            env.storage()
                .persistent()
                .set(&DataKey::LbpPool(pool_id), &pool);
        } else if env
            .storage()
            .persistent()
            .has(&DataKey::BondingPool(pool_id.clone()))
        {
            let mut pool: BondingCurvePool = env
                .storage()
                .persistent()
                .get(&DataKey::BondingPool(pool_id.clone()))
                .unwrap();
            pool.is_active = false;
            pool.graduated = true;
            env.storage()
                .persistent()
                .set(&DataKey::BondingPool(pool_id), &pool);
        }
    }

    pub fn get_fair_launch_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::FairLaunchContract)
            .unwrap()
    }

    pub fn get_oracle_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::OracleContract)
            .unwrap()
    }

    fn generate_pool_id(env: &Env) -> BytesN<32> {
        let counter: u32 = env.storage().instance().get(&DataKey::PoolCounter).unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::PoolCounter, &(counter + 1));
        let mut arr = [0u8; 32];
        arr[..4].copy_from_slice(&counter.to_be_bytes());
        BytesN::from_array(env, &arr)
    }
}

#[cfg(test)]
mod test;


