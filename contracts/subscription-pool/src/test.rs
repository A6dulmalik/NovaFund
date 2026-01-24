#![cfg(test)]
use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Env, String};

/// Helper to setup the test environment
fn setup_test(env: &Env) -> (SubscriptionPoolContractClient, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SubscriptionPoolContract);
    let client = SubscriptionPoolContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    (client, user, token)
}

// --- 1. POOL & SUBSCRIPTION TESTS ---

#[test]
fn test_pool_creation() {
    let env = Env::default();
    let (client, _, token) = setup_test(&env);

    let name = String::from_str(&env, "Strategy A");
    let pool_id = client.create_pool(&name, &token);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.pool_id, 1);
    assert_eq!(pool.name, name);
    assert_eq!(pool.total_balance, 0);
}

#[test]
fn test_subscriber_enrollment() {
    let env = Env::default();
    let (client, user, token) = setup_test(&env);
    let pool_id = client.create_pool(&String::from_str(&env, "Strategy A"), &token);

    let amount = 500_000_000;
    client.subscribe(&pool_id, &user, &amount, &SubscriptionPeriod::Weekly);

    let sub = client.get_subscription(&pool_id, &user);
    assert_eq!(sub.amount, amount);
    assert_eq!(sub.period, SubscriptionPeriod::Weekly);
    assert_eq!(sub.subscriber, user);
}

// --- 2. RECURRING CONTRIBUTIONS TESTS ---

#[test]
fn test_process_deposits_increases_balance_after_period() {
    let env = Env::default();
    let (client, user, token) = setup_test(&env);
    let pool_id = client.create_pool(&String::from_str(&env, "Strategy A"), &token);
    let amount = 200_000_000;

    client.subscribe(&pool_id, &user, &amount, &SubscriptionPeriod::Weekly);

    // Warp time: 1 week = 604,800 seconds
    env.ledger().with_mut(|li| li.timestamp = 604_801);

    client.process_deposits(&pool_id, &user);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.total_balance, amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")] // #3 is Error::PeriodNotElapsed
fn test_cannot_process_deposit_too_soon() {
    let env = Env::default();
    let (client, user, token) = setup_test(&env);
    let pool_id = client.create_pool(&String::from_str(&env, "Strategy A"), &token);

    client.subscribe(&pool_id, &user, &200_000_000, &SubscriptionPeriod::Monthly);

    // Only 1 day passes instead of 30
    env.ledger().with_mut(|li| li.timestamp = 86_400);

    client.process_deposits(&pool_id, &user);
}

// --- 3. WITHDRAWAL TESTS ---

#[test]
fn test_withdrawal_reduces_balance() {
    let env = Env::default();
    let (client, user, token) = setup_test(&env);
    let pool_id = client.create_pool(&String::from_str(&env, "Strategy A"), &token);
    
    // Simulate a deposit first so there is money to withdraw
    client.subscribe(&pool_id, &user, &500_000_000, &SubscriptionPeriod::Weekly);
    env.ledger().with_mut(|li| li.timestamp = 604_801);
    client.process_deposits(&pool_id, &user);

    client.withdraw(&pool_id, &user, &200_000_000);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.total_balance, 300_000_000);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")] // #2 is Error::InsufficientBalance
fn test_guardrail_prevents_excessive_withdrawal() {
    let env = Env::default();
    let (client, user, token) = setup_test(&env);
    let pool_id = client.create_pool(&String::from_str(&env, "Strategy A"), &token);

    // Pool is empty (0 balance)
    client.withdraw(&pool_id, &user, &1); 
}