#![no_std]

// TODO: Implement subscription pool contract
// This contract will handle:
// - Create recurring investment pools
// - Manage subscriber deposits (weekly/monthly/quarterly)
// - Portfolio rebalancing
// - Withdrawal with payout calculation
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, 
    Address, Env, String, Symbol,
};

// --- Data Models ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionPeriod {
    Weekly,
    Monthly,
    Quarterly,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Pool(u64),
    Subscription(u64, Address),
    PoolCount,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Pool {
    pub pool_id: u64,
    pub name: String,
    pub token: Address,
    pub total_balance: i128,
    pub subscriber_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Subscription {
    pub subscriber: Address,
    pub pool_id: u64,
    pub amount: i128,
    pub period: SubscriptionPeriod,
    pub last_payment: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    BelowMinimum = 1,
    InsufficientBalance = 2,
    PeriodNotElapsed = 3,
    PoolNotFound = 4,
    SubscriptionNotFound = 5,
}

// --- Constants ---
const MIN_SUBSCRIPTION: i128 = 100_000_000; // Example: 10.0 units (7 decimals)

#[contract]
pub struct SubscriptionPoolContract;

#[contractimpl]
impl SubscriptionPoolContract {

    pub fn create_pool(env: Env, name: String, token: Address) -> u64 {
        let mut count: u64 = env.storage().instance().get(&DataKey::PoolCount).unwrap_or(0);
        count += 1;

        let pool = Pool {
            pool_id: count,
            name: name.clone(),
            token,
            total_balance: 0,
            subscriber_count: 0,
        };

        env.storage().instance().set(&DataKey::Pool(count), &pool);
        env.storage().instance().set(&DataKey::PoolCount, &count);

        // Event: Pool Creation
        env.events().publish((symbol_short!("created"), count), name);
        count
    }

    pub fn subscribe(env: Env, pool_id: u64, subscriber: Address, amount: i128, period: SubscriptionPeriod) -> Result<(), Error> {
        subscriber.require_auth();

        if amount < MIN_SUBSCRIPTION {
            return Err(Error::BelowMinimum);
        }

        let mut pool = self::SubscriptionPoolContract::get_pool(env.clone(), pool_id)?;
        let sub_key = DataKey::Subscription(pool_id, subscriber.clone());

        let subscription = Subscription {
            subscriber: subscriber.clone(),
            pool_id,
            amount,
            period,
            last_payment: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&sub_key, &subscription);
        
        pool.subscriber_count += 1;
        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);

        // Event: Subscription enrollment
        env.events().publish((symbol_short!("subbed"), pool_id), subscriber);
        Ok(())
    }

    pub fn process_deposits(env: Env, pool_id: u64, subscriber: Address) -> Result<(), Error> {
        let sub_key = DataKey::Subscription(pool_id, subscriber.clone());
        let mut sub: Subscription = env.storage().persistent().get(&sub_key).ok_or(Error::SubscriptionNotFound)?;
        let mut pool = self::SubscriptionPoolContract::get_pool(env.clone(), pool_id)?;

        let now = env.ledger().timestamp();
        let seconds_in_period: u64 = match sub.period {
            SubscriptionPeriod::Weekly => 604800,
            SubscriptionPeriod::Monthly => 2592000,
            SubscriptionPeriod::Quarterly => 7776000,
        };

        if now < (sub.last_payment + seconds_in_period) {
            return Err(Error::PeriodNotElapsed);
        }

        // Logic Note: In a real scenario, invoke a Token Client transfer here
        // pool_token_client.transfer(&sub.subscriber, &env.current_contract_address(), &sub.amount);

        pool.total_balance += sub.amount;
        sub.last_payment = now;

        env.storage().persistent().set(&sub_key, &sub);
        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);

        // Event: Processed deposit
        env.events().publish((symbol_short!("deposit"), pool_id), sub.amount);
        Ok(())
    }

    pub fn withdraw(env: Env, pool_id: u64, subscriber: Address, amount: i128) -> Result<(), Error> {
        subscriber.require_auth();

        let mut pool = self::SubscriptionPoolContract::get_pool(env.clone(), pool_id)?;
        
        // Validation: Prevent withdrawing more than available in pool
        if amount > pool.total_balance {
            return Err(Error::InsufficientBalance);
        }

        pool.total_balance -= amount;
        env.storage().instance().set(&DataKey::Pool(pool_id), &pool);

        // Event: Withdrawal
        env.events().publish((symbol_short!("withdraw"), pool_id), amount);
        Ok(())
    }

    // --- Helpers / Getters ---

    pub fn get_pool(env: Env, pool_id: u64) -> Result<Pool, Error> {
        env.storage().instance().get(&DataKey::Pool(pool_id)).ok_or(Error::PoolNotFound)
    }

    pub fn get_subscription(env: Env, pool_id: u64, subscriber: Address) -> Result<Subscription, Error> {
        let sub_key = DataKey::Subscription(pool_id, subscriber);
        env.storage().persistent().get(&sub_key).ok_or(Error::SubscriptionNotFound)
    }
}

mod test;