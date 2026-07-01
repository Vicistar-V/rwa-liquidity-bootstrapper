use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

use amm_math::{FairLaunchConfig, WalletPurchaseRecord};

#[contracttype]
pub enum FairLaunchDataKey {
    PurchaseRecord(Address, BytesN<32>),
    PoolState(BytesN<32>),
    FairLaunchConfig(BytesN<32>),
    PoolStartTime(BytesN<32>),
    RwaToken(BytesN<32>),
    ReserveToken(BytesN<32>),
}

#[contract]
pub struct FairLaunchController;

#[contractimpl]
impl FairLaunchController {
    pub fn initialize(
        env: Env,
        pool_id: BytesN<32>,
        config: FairLaunchConfig,
        start_time: u64,
        rwa_token: Address,
        reserve_token: Address,
    ) {
        if env
            .storage()
            .persistent()
            .has(&FairLaunchDataKey::FairLaunchConfig(pool_id.clone()))
        {
            panic!("already initialized");
        }
        env.storage()
            .persistent()
            .set(&FairLaunchDataKey::FairLaunchConfig(pool_id.clone()), &config);
        env.storage()
            .persistent()
            .set(&FairLaunchDataKey::PoolStartTime(pool_id.clone()), &start_time);
        env.storage()
            .persistent()
            .set(&FairLaunchDataKey::RwaToken(pool_id.clone()), &rwa_token);
        env.storage()
            .persistent()
            .set(&FairLaunchDataKey::ReserveToken(pool_id.clone()), &reserve_token);

        let initial_state: (u128, u128) = (0, 0);
        env.storage()
            .persistent()
            .set(&FairLaunchDataKey::PoolState(pool_id), &initial_state);
    }

    pub fn check_purchase_allowed(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        amount: u128,
    ) -> bool {
        let config: FairLaunchConfig = env
            .storage()
            .persistent()
            .get(&FairLaunchDataKey::FairLaunchConfig(pool_id.clone()))
            .unwrap();
        let start_time: u64 = env
            .storage()
            .persistent()
            .get(&FairLaunchDataKey::PoolStartTime(pool_id.clone()))
            .unwrap();
        let now = env.ledger().timestamp();

        // 1. Blackout period
        if let Some(blackout) = config.initial_blackout_period {
            if now < start_time.saturating_add(blackout) {
                panic!("BlackoutPeriodActive");
            }
        }

        // 2. Wallet age (approximate using a stored creation ledger)
        if let Some(min_age) = config.min_wallet_age_ledgers {
            let ledger_seq = env.ledger().sequence();
            let wallet_key = (buyer.clone(), pool_id.clone());
            let record: Option<WalletPurchaseRecord> = env
                .storage()
                .persistent()
                .get(&FairLaunchDataKey::PurchaseRecord(wallet_key.0.clone(), wallet_key.1.clone()));
            if let Some(rec) = record {
                if rec.purchase_count == 0 {
                    if ledger_seq < min_age {
                        panic!("WalletTooNew");
                    }
                }
            } else {
                if ledger_seq < min_age {
                    panic!("WalletTooNew");
                }
            }
        }

        // 3. Cooldown
        if let Some(cooldown) = config.cooldown_between_purchases {
            let record: Option<WalletPurchaseRecord> = env
                .storage()
                .persistent()
                .get(&FairLaunchDataKey::PurchaseRecord(buyer.clone(), pool_id.clone()));
            if let Some(rec) = record {
                if now < rec.last_purchase_time.saturating_add(cooldown) {
                    panic!("CooldownNotElapsed");
                }
            }
        }

        // 4. Wallet cap
        if let Some(max_tokens) = config.max_tokens_per_wallet {
            let record: Option<WalletPurchaseRecord> = env
                .storage()
                .persistent()
                .get(&FairLaunchDataKey::PurchaseRecord(buyer.clone(), pool_id.clone()));
            let total_purchased = record
                .as_ref()
                .map(|r| r.total_tokens_purchased)
                .unwrap_or(0);
            if total_purchased.saturating_add(amount) > max_tokens {
                panic!("WalletCapExceeded");
            }
        }

        if let Some(max_usdc) = config.max_usdc_per_wallet {
            let record: Option<WalletPurchaseRecord> = env
                .storage()
                .persistent()
                .get(&FairLaunchDataKey::PurchaseRecord(buyer.clone(), pool_id.clone()));
            let total_spent = record
                .as_ref()
                .map(|r| r.total_usdc_spent)
                .unwrap_or(0);
            if total_spent.saturating_add(amount) > max_usdc {
                panic!("WalletCapExceeded");
            }
        }

        // 5. Pool ownership cap
        if let Some(max_ownership_pct) = config.max_pool_ownership_pct {
            let state: (u128, u128) = env
                .storage()
                .persistent()
                .get(&FairLaunchDataKey::PoolState(pool_id.clone()))
                .unwrap_or((0, 0));
            let total_supply = state.0;
            if total_supply > 0 {
                let record: Option<WalletPurchaseRecord> = env
                    .storage()
                    .persistent()
                    .get(&FairLaunchDataKey::PurchaseRecord(
                        buyer.clone(),
                        pool_id.clone(),
                    ));
                let total_purchased = record
                    .as_ref()
                    .map(|r| r.total_tokens_purchased)
                    .unwrap_or(0);
                let new_total = total_purchased.saturating_add(amount);
                if new_total.saturating_mul(amm_math::SCALE) / total_supply > max_ownership_pct {
                    panic!("PoolOwnershipCapExceeded");
                }
            }
        }

        true
    }

    pub fn record_purchase(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        tokens_purchased: u128,
        usdc_spent: u128,
    ) {
        let now = env.ledger().timestamp();

        let mut record: WalletPurchaseRecord = env
            .storage()
            .persistent()
            .get(&FairLaunchDataKey::PurchaseRecord(buyer.clone(), pool_id.clone()))
            .unwrap_or(WalletPurchaseRecord {
                wallet: buyer.clone(),
                pool_id: pool_id.clone(),
                total_tokens_purchased: 0,
                total_usdc_spent: 0,
                last_purchase_time: 0,
                purchase_count: 0,
            });

        record.total_tokens_purchased = record.total_tokens_purchased.saturating_add(tokens_purchased);
        record.total_usdc_spent = record.total_usdc_spent.saturating_add(usdc_spent);
        record.last_purchase_time = now;
        record.purchase_count = record.purchase_count.saturating_add(1);

        env.storage()
            .persistent()
            .set(&FairLaunchDataKey::PurchaseRecord(buyer, pool_id.clone()), &record);

        let mut state: (u128, u128) = env
            .storage()
            .persistent()
            .get(&FairLaunchDataKey::PoolState(pool_id.clone()))
            .unwrap_or((0, 0));
        state.0 = state.0.saturating_add(tokens_purchased);
        state.1 = state.1.saturating_add(usdc_spent);
        env.storage()
            .persistent()
            .set(&FairLaunchDataKey::PoolState(pool_id), &state);
    }

    pub fn get_wallet_purchases(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
    ) -> WalletPurchaseRecord {
        let b = buyer.clone();
        let p = pool_id.clone();
        env.storage()
            .persistent()
            .get(&FairLaunchDataKey::PurchaseRecord(b, p))
            .unwrap_or(WalletPurchaseRecord {
                wallet: buyer,
                pool_id: pool_id,
                total_tokens_purchased: 0,
                total_usdc_spent: 0,
                last_purchase_time: 0,
                purchase_count: 0,
            })
    }

    pub fn get_pool_purchase_state(env: Env, pool_id: BytesN<32>) -> (u128, u128) {
        env.storage()
            .persistent()
            .get(&FairLaunchDataKey::PoolState(pool_id))
            .unwrap_or((0, 0))
    }
}
