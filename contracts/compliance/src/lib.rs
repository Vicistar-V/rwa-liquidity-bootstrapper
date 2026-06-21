use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use amm_math::ComplianceDecision;

#[contract]
pub struct ComplianceBridge;

#[contractimpl]
impl ComplianceBridge {
    pub fn initialize(env: Env, pool_id: BytesN<32>, compliance_contract: Address) {
        unimplemented!()
    }

    pub fn check_purchase(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        amount: u128,
    ) -> ComplianceDecision {
        unimplemented!()
    }

    pub fn set_compliance_contract(env: Env, pool_id: BytesN<32>, contract: Address) {
        unimplemented!()
    }
}
