use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

use amm_math::ComplianceDecision;

#[contracttype]
pub enum ComplianceDataKey {
    Admin,
    Initialized,
    PoolConfig(BytesN<32>),
}

#[contracttype]
#[derive(Clone)]
pub struct PoolComplianceConfig {
    pub kyc_required: bool,
    pub min_kyc_tier: u32,
    pub external_compliance_contract: Option<Address>,
}

#[contract]
pub struct ComplianceBridge;

#[contractimpl]
impl ComplianceBridge {
    pub fn initialize(env: Env, admin: Address, pool_id: BytesN<32>, compliance_contract: Address) {
        if env.storage().instance().has(&ComplianceDataKey::Initialized) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&ComplianceDataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&ComplianceDataKey::Admin, &admin);

        let config = PoolComplianceConfig {
            kyc_required: true,
            min_kyc_tier: 1,
            external_compliance_contract: Some(compliance_contract),
        };
        env.storage()
            .persistent()
            .set(&ComplianceDataKey::PoolConfig(pool_id), &config);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ComplianceDataKey::Admin)
            .unwrap();
        admin.require_auth();
        env.storage()
            .instance()
            .set(&ComplianceDataKey::Admin, &new_admin);
    }

    pub fn set_pool_config(env: Env, pool_id: BytesN<32>, config: PoolComplianceConfig) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ComplianceDataKey::Admin)
            .unwrap();
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&ComplianceDataKey::PoolConfig(pool_id), &config);
    }

    pub fn get_pool_config(env: Env, pool_id: BytesN<32>) -> Option<PoolComplianceConfig> {
        env.storage()
            .persistent()
            .get(&ComplianceDataKey::PoolConfig(pool_id))
    }

    pub fn check_purchase(
        _env: Env,
        _buyer: Address,
        _pool_id: BytesN<32>,
        _amount: u128,
    ) -> ComplianceDecision {
        ComplianceDecision::Approve
    }

    pub fn set_compliance_contract(_env: Env, _pool_id: BytesN<32>, _contract: Address) {
        let admin: Address = _env
            .storage()
            .instance()
            .get(&ComplianceDataKey::Admin)
            .unwrap();
        admin.require_auth();
    }
}

#[cfg(test)]
mod test;
