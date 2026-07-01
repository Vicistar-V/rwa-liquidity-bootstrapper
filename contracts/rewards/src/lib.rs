use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

#[contracttype]
pub enum RewardsDataKey {
    Initialized(BytesN<32>),
    FeesRwa(BytesN<32>),
    FeesUsdc(BytesN<32>),
}

#[contract]
pub struct LpRewards;

#[contractimpl]
impl LpRewards {
    pub fn initialize(env: Env, pool_id: BytesN<32>) {
        if env.storage().persistent().has(&RewardsDataKey::Initialized(pool_id.clone())) {
            panic!("already initialized");
        }
        env.storage().persistent().set(&RewardsDataKey::Initialized(pool_id.clone()), &true);
        env.storage().persistent().set(&RewardsDataKey::FeesRwa(pool_id.clone()), &0u128);
        env.storage().persistent().set(&RewardsDataKey::FeesUsdc(pool_id), &0u128);
    }

    pub fn deposit_fees(env: Env, pool_id: BytesN<32>, amount_rwa: u128, amount_usdc: u128) {
        let rwa: u128 = env.storage().persistent().get(&RewardsDataKey::FeesRwa(pool_id.clone())).unwrap_or(0);
        let usdc: u128 = env.storage().persistent().get(&RewardsDataKey::FeesUsdc(pool_id.clone())).unwrap_or(0);
        env.storage().persistent().set(&RewardsDataKey::FeesRwa(pool_id.clone()), &rwa.saturating_add(amount_rwa));
        env.storage().persistent().set(&RewardsDataKey::FeesUsdc(pool_id), &usdc.saturating_add(amount_usdc));
    }

    pub fn claim(_env: Env, _lp: Address, _pool_id: BytesN<32>) -> (u128, u128) {
        (0, 0)
    }

    pub fn get_rewards(_env: Env, _lp: Address, _pool_id: BytesN<32>) -> (u128, u128) {
        (0, 0)
    }
}
