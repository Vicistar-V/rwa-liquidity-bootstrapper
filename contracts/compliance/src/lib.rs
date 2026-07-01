use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use amm_math::ComplianceDecision;

#[contract]
pub struct ComplianceBridge;

#[contractimpl]
impl ComplianceBridge {
    pub fn initialize(_env: Env, _pool_id: BytesN<32>, _compliance_contract: Address) {
        unimplemented!()
    }

    pub fn check_purchase(
        _env: Env,
        _buyer: Address,
        _pool_id: BytesN<32>,
        _amount: u128,
    ) -> ComplianceDecision {
        unimplemented!()
    }

    pub fn set_compliance_contract(_env: Env, _pool_id: BytesN<32>, _contract: Address) {
        unimplemented!()
    }
}
