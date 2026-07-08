use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, IntoVal, Symbol, Val, Vec};

use amm_math::{ComplianceDecision, KycStatus, PoolComplianceConfig};

#[contracttype]
pub enum ComplianceDataKey {
    Admin,
    Initialized,
    PoolConfig(BytesN<32>),
    WalletKyc(Address),
    Blacklisted(Address),
    ExternalContract(BytesN<32>),
}

#[contract]
pub struct ComplianceBridge;

#[contractimpl]
impl ComplianceBridge {
    pub fn initialize(env: Env, admin: Address, pool_id: BytesN<32>, _compliance_contract: Address) {
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

    pub fn set_wallet_kyc(env: Env, wallet: Address, tier: u32, verified: bool) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ComplianceDataKey::Admin)
            .unwrap();
        admin.require_auth();
        let status = KycStatus {
            wallet: wallet.clone(),
            tier,
            verified,
            timestamp: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&ComplianceDataKey::WalletKyc(wallet), &status);
    }

    pub fn get_wallet_kyc(env: Env, wallet: Address) -> Option<KycStatus> {
        env.storage()
            .persistent()
            .get(&ComplianceDataKey::WalletKyc(wallet))
    }

    pub fn set_blacklisted(env: Env, wallet: Address, blacklisted: bool) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ComplianceDataKey::Admin)
            .unwrap();
        admin.require_auth();
        if blacklisted {
            env.storage()
                .persistent()
                .set(&ComplianceDataKey::Blacklisted(wallet), &true);
        } else {
            env.storage()
                .persistent()
                .remove(&ComplianceDataKey::Blacklisted(wallet));
        }
    }

    pub fn is_blacklisted(env: Env, wallet: Address) -> bool {
        env.storage()
            .persistent()
            .has(&ComplianceDataKey::Blacklisted(wallet))
    }

    pub fn check_purchase(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        _amount: u128,
    ) -> ComplianceDecision {
        if env
            .storage()
            .persistent()
            .has(&ComplianceDataKey::Blacklisted(buyer.clone()))
        {
            return ComplianceDecision::Reject(BytesN::from_array(&env, &[0x01; 32]));
        }

        let config: PoolComplianceConfig =
            match env
                .storage()
                .persistent()
                .get(&ComplianceDataKey::PoolConfig(pool_id.clone()))
            {
                Some(cfg) => cfg,
                None => return ComplianceDecision::Approve,
            };

        if let Some(external) = env
            .storage()
            .persistent()
            .get::<_, Address>(&ComplianceDataKey::ExternalContract(pool_id.clone()))
        {
            let mut args: Vec<Val> = Vec::new(&env);
            args.push_back(buyer.into_val(&env));
            args.push_back(pool_id.into_val(&env));
            args.push_back(_amount.into_val(&env));
            return env
                .invoke_contract(&external, &Symbol::new(&env, "check_purchase"), args);
        }

        if config.kyc_required {
            let kyc: Option<KycStatus> = env
                .storage()
                .persistent()
                .get(&ComplianceDataKey::WalletKyc(buyer.clone()));
            match kyc {
                None => return ComplianceDecision::PendingKyc,
                Some(status) => {
                    if !status.verified || status.tier < config.min_kyc_tier {
                        return ComplianceDecision::Reject(BytesN::from_array(&env, &[0x02; 32]));
                    }
                }
            }
        }

        ComplianceDecision::Approve
    }

    pub fn set_compliance_contract(env: Env, pool_id: BytesN<32>, new_contract: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ComplianceDataKey::Admin)
            .unwrap();
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&ComplianceDataKey::ExternalContract(pool_id), &new_contract);
    }
}

#[cfg(test)]
mod test;
