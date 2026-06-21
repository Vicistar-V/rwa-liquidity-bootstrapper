use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use amm_math::{FairLaunchConfig, WalletPurchaseRecord};

#[contract]
pub struct FairLaunchController;

#[contractimpl]
impl FairLaunchController {
    pub fn initialize(env: Env, pool_id: BytesN<32>, config: FairLaunchConfig) {
        unimplemented!()
    }

    pub fn check_purchase_allowed(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        amount: u128,
    ) -> bool {
        unimplemented!()
    }

    pub fn record_purchase(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
        tokens_purchased: u128,
        usdc_spent: u128,
    ) {
        unimplemented!()
    }

    pub fn get_wallet_purchases(
        env: Env,
        buyer: Address,
        pool_id: BytesN<32>,
    ) -> WalletPurchaseRecord {
        unimplemented!()
    }

    pub fn get_pool_purchase_state(env: Env, pool_id: BytesN<32>) -> (u128, u128) {
        unimplemented!()
    }
}
