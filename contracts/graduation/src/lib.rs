use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

use amm_math::{GraduationCriteria, GraduationReceipt};

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
        unimplemented!()
    }

    pub fn check_graduation_ready(env: Env, pool_id: BytesN<32>) -> GraduationStatus {
        unimplemented!()
    }

    pub fn graduate_pool(env: Env, pool_id: BytesN<32>) -> GraduationReceipt {
        unimplemented!()
    }

    pub fn trigger_early_graduation(env: Env, issuer: Address, pool_id: BytesN<32>) {
        unimplemented!()
    }
}
