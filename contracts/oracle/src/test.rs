use soroban_sdk::{BytesN, Env};
use soroban_sdk::testutils::Ledger as _;

use crate::TwapOracle;

#[test]
fn test_initialize_oracle() {
    let env = Env::default();
    let oracle_id = env.register_contract(None, TwapOracle);
    let client = crate::TwapOracleClient::new(&env, &oracle_id);
    let pool_id = BytesN::from_array(&env, &[10; 32]);

    client.initialize(&pool_id);

    let latest = client.get_latest_price(&pool_id);
    assert_eq!(latest, 0);
}

#[test]
fn test_record_and_retrieve() {
    let env = Env::default();
    let oracle_id = env.register_contract(None, TwapOracle);
    let client = crate::TwapOracleClient::new(&env, &oracle_id);
    let pool_id = BytesN::from_array(&env, &[11; 32]);

    client.initialize(&pool_id);

    env.ledger().with_mut(|info| info.timestamp = 0);
    client.record_observation(&pool_id, &100_0000000u128);

    env.ledger().with_mut(|info| info.timestamp = 3600);
    client.record_observation(&pool_id, &110_0000000u128);

    let latest = client.get_latest_price(&pool_id);
    assert_eq!(latest, 110_0000000);

    let twap = client.get_twap(&pool_id, &3600u64);
    assert!(twap > 0);
}

#[test]
fn test_twap_smooths_spikes() {
    let env = Env::default();
    let oracle_id = env.register_contract(None, TwapOracle);
    let client = crate::TwapOracleClient::new(&env, &oracle_id);
    let pool_id = BytesN::from_array(&env, &[12; 32]);

    client.initialize(&pool_id);

    env.ledger().with_mut(|info| info.timestamp = 0);
    client.record_observation(&pool_id, &100_0000000u128);

    env.ledger().with_mut(|info| info.timestamp = 3600);
    client.record_observation(&pool_id, &100_0000000u128);

    env.ledger().with_mut(|info| info.timestamp = 7200);
    client.record_observation(&pool_id, &1000_0000000u128);

    env.ledger().with_mut(|info| info.timestamp = 10800);
    client.record_observation(&pool_id, &100_0000000u128);

    let twap = client.get_twap(&pool_id, &10800u64);
    assert!(twap > 100_0000000);
    assert!(twap < 1000_0000000);
}

#[test]
fn test_get_price_history() {
    let env = Env::default();
    let oracle_id = env.register_contract(None, TwapOracle);
    let client = crate::TwapOracleClient::new(&env, &oracle_id);
    let pool_id = BytesN::from_array(&env, &[13; 32]);

    client.initialize(&pool_id);

    env.ledger().with_mut(|info| info.timestamp = 0);
    client.record_observation(&pool_id, &100_0000000u128);

    env.ledger().with_mut(|info| info.timestamp = 3600);
    client.record_observation(&pool_id, &110_0000000u128);

    env.ledger().with_mut(|info| info.timestamp = 7200);
    client.record_observation(&pool_id, &120_0000000u128);

    let history = client.get_price_history(&pool_id, &3600u64, &10u32);
    assert_eq!(history.len(), 2);

    let all = client.get_price_history(&pool_id, &0u64, &10u32);
    assert_eq!(all.len(), 3);
}

#[test]
fn test_twap_single_observation() {
    let env = Env::default();
    let oracle_id = env.register_contract(None, TwapOracle);
    let client = crate::TwapOracleClient::new(&env, &oracle_id);
    let pool_id = BytesN::from_array(&env, &[14; 32]);

    client.initialize(&pool_id);

    env.ledger().with_mut(|info| info.timestamp = 0);
    client.record_observation(&pool_id, &100_0000000u128);

    let twap = client.get_twap(&pool_id, &3600u64);
    assert_eq!(twap, 100_0000000);
}

#[test]
fn test_twap_no_observations() {
    let env = Env::default();
    let oracle_id = env.register_contract(None, TwapOracle);
    let client = crate::TwapOracleClient::new(&env, &oracle_id);
    let pool_id = BytesN::from_array(&env, &[15; 32]);

    client.initialize(&pool_id);

    let twap = client.get_twap(&pool_id, &3600u64);
    assert_eq!(twap, 0);
}
