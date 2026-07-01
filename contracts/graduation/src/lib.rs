use soroban_sdk::{contract, contractimpl, contracttype, token, Address, BytesN, Env, IntoVal, Symbol, Val, Vec};

use amm_math::{GraduationCriteria, GraduationReceipt};

const PROTOCOL_FEE_PCT: u128 = 5;
const DEX_SEED_PCT: u128 = 15;
const ISSUER_PCT: u128 = 80;
const SPLIT_BASE: u128 = 100;

#[contracttype]
pub enum GraduationDataKey {
    Config(BytesN<32>),
    Graduated(BytesN<32>),
    FactoryContract,
    FairLaunchContract,
    ProtocolFeeRecipient,
    DexSeedAddress,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum GraduationStatus {
    NotReady,
    Ready,
    Graduated,
}

#[contract]
pub struct GraduationEngine;

#[contractimpl]
impl GraduationEngine {
    pub fn initialize(
        env: Env,
        pool_id: BytesN<32>,
        criteria: GraduationCriteria,
        issuer: Address,
        threshold: u128,
    ) {
        if env.storage().persistent().has(&GraduationDataKey::Config(pool_id.clone())) {
            panic!("already initialized");
        }
        let config = (criteria, issuer, threshold);
        env.storage().persistent().set(&GraduationDataKey::Config(pool_id), &config);
    }

    pub fn set_factory(env: Env, factory: Address) {
        if !env.storage().instance().has(&GraduationDataKey::FactoryContract) {
            env.storage().instance().set(&GraduationDataKey::FactoryContract, &factory);
        }
    }

    pub fn set_fair_launch(env: Env, fair_launch: Address) {
        if !env.storage().instance().has(&GraduationDataKey::FairLaunchContract) {
            env.storage().instance().set(&GraduationDataKey::FairLaunchContract, &fair_launch);
        }
    }

    pub fn set_fee_recipients(env: Env, protocol: Address, dex_seed: Address) {
        env.storage().instance().set(&GraduationDataKey::ProtocolFeeRecipient, &protocol);
        env.storage().instance().set(&GraduationDataKey::DexSeedAddress, &dex_seed);
    }

    pub fn check_graduation_ready(env: Env, pool_id: BytesN<32>) -> GraduationStatus {
        if env.storage().persistent().has(&GraduationDataKey::Graduated(pool_id.clone())) {
            return GraduationStatus::Graduated;
        }

        let config: (GraduationCriteria, Address, u128) = env
            .storage()
            .persistent()
            .get(&GraduationDataKey::Config(pool_id.clone()))
            .unwrap();
        let (criteria, _, threshold) = config;

        match criteria {
            GraduationCriteria::TimeElapsed => {
                let now = env.ledger().timestamp();
                if now >= threshold as u64 {
                    GraduationStatus::Ready
                } else {
                    GraduationStatus::NotReady
                }
            }
            GraduationCriteria::FundsRaised => {
                let total_raised = Self::query_total_usdc_raised(&env, &pool_id);
                if total_raised >= threshold {
                    GraduationStatus::Ready
                } else {
                    GraduationStatus::NotReady
                }
            }
            GraduationCriteria::TokensSold => {
                let total_sold = Self::query_total_tokens_sold(&env, &pool_id);
                if total_sold >= threshold {
                    GraduationStatus::Ready
                } else {
                    GraduationStatus::NotReady
                }
            }
            GraduationCriteria::IssuerTriggered => {
                GraduationStatus::NotReady
            }
        }
    }

    pub fn graduate_pool(env: Env, pool_id: BytesN<32>) -> GraduationReceipt {
        let status = Self::check_graduation_ready(env.clone(), pool_id.clone());
        match status {
            GraduationStatus::NotReady => {
                panic!("graduation not ready");
            }
            GraduationStatus::Graduated => {
                panic!("already graduated");
            }
            GraduationStatus::Ready => {}
        }

        let config: (GraduationCriteria, Address, u128) = env
            .storage()
            .persistent()
            .get(&GraduationDataKey::Config(pool_id.clone()))
            .unwrap();
        let (_, issuer, _) = config;

        let total_usdc = Self::query_total_usdc_raised(&env, &pool_id);
        let total_tokens = Self::query_total_tokens_sold(&env, &pool_id);
        let final_price = if total_tokens > 0 {
            amm_math::fixed_div(total_usdc, total_tokens)
        } else {
            0
        };

        let usdc_to_issuer = total_usdc.saturating_mul(ISSUER_PCT) / SPLIT_BASE;
        let usdc_to_dex = total_usdc.saturating_mul(DEX_SEED_PCT) / SPLIT_BASE;
        let usdc_to_protocol = total_usdc.saturating_mul(PROTOCOL_FEE_PCT) / SPLIT_BASE;

        let factory: Address = env
            .storage()
            .instance()
            .get(&GraduationDataKey::FactoryContract)
            .unwrap();

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(pool_id.clone().into_val(&env));
        env.invoke_contract::<()>(&factory, &Symbol::new(&env, "mark_pool_graduated"), args);

        let mut pool_args: Vec<Val> = Vec::new(&env);
        pool_args.push_back(pool_id.clone().into_val(&env));
        let pool_summary: amm_math::PoolSummary = env
            .invoke_contract(&factory, &Symbol::new(&env, "get_pool_summary"), pool_args);

        let usdc_token = pool_summary.rwa_token;

        let protocol_fee_recv: Address = env
            .storage()
            .instance()
            .get(&GraduationDataKey::ProtocolFeeRecipient)
            .unwrap();
        let dex_seed_addr: Address = env
            .storage()
            .instance()
            .get(&GraduationDataKey::DexSeedAddress)
            .unwrap();

        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&env.current_contract_address(), &issuer, &(usdc_to_issuer as i128));
        token_client.transfer(&env.current_contract_address(), &dex_seed_addr, &(usdc_to_dex as i128));
        token_client.transfer(&env.current_contract_address(), &protocol_fee_recv, &(usdc_to_protocol as i128));

        env.storage()
            .persistent()
            .set(&GraduationDataKey::Graduated(pool_id.clone()), &true);

        let graduation_timestamp = env.ledger().timestamp();

        env.events().publish(
            ("GraduationEngine", "PoolGraduated"),
            (pool_id.clone(), total_usdc, total_tokens, final_price),
        );

        GraduationReceipt {
            pool_id,
            total_usdc_raised: total_usdc,
            total_tokens_sold: total_tokens,
            final_price,
            migration_destination: dex_seed_addr,
            usdc_to_issuer,
            usdc_to_lp_pool: usdc_to_dex,
            graduation_timestamp,
        }
    }

    pub fn trigger_early_graduation(env: Env, issuer: Address, pool_id: BytesN<32>) {
        issuer.require_auth();

        let config: (GraduationCriteria, Address, u128) = env
            .storage()
            .persistent()
            .get(&GraduationDataKey::Config(pool_id.clone()))
            .unwrap();
        let (_, stored_issuer, _) = config;
        if issuer != stored_issuer {
            panic!("only issuer can trigger early graduation");
        }

        if env.storage().persistent().has(&GraduationDataKey::Graduated(pool_id.clone())) {
            panic!("already graduated");
        }

        let factory: Address = env
            .storage()
            .instance()
            .get(&GraduationDataKey::FactoryContract)
            .unwrap();

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(pool_id.clone().into_val(&env));
        env.invoke_contract::<()>(&factory, &Symbol::new(&env, "mark_pool_graduated"), args);

        env.storage()
            .persistent()
            .set(&GraduationDataKey::Graduated(pool_id.clone()), &true);

        env.events().publish(
            ("GraduationEngine", "EarlyGraduation"),
            (issuer, pool_id),
        );
    }

    fn query_total_usdc_raised(env: &Env, pool_id: &BytesN<32>) -> u128 {
        let fair_launch: Address = env
            .storage()
            .instance()
            .get(&GraduationDataKey::FairLaunchContract)
            .unwrap();
        let mut args: Vec<Val> = Vec::new(env);
        args.push_back(pool_id.clone().into_val(env));
        let state: (u128, u128) = env
            .invoke_contract(&fair_launch, &Symbol::new(env, "get_pool_purchase_state"), args);
        state.1
    }

    fn query_total_tokens_sold(env: &Env, pool_id: &BytesN<32>) -> u128 {
        let fair_launch: Address = env
            .storage()
            .instance()
            .get(&GraduationDataKey::FairLaunchContract)
            .unwrap();
        let mut args: Vec<Val> = Vec::new(env);
        args.push_back(pool_id.clone().into_val(env));
        let state: (u128, u128) = env
            .invoke_contract(&fair_launch, &Symbol::new(env, "get_pool_purchase_state"), args);
        state.0
    }
}

#[cfg(test)]
mod test;
