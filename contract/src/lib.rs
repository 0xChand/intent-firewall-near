use near_sdk::{near, env, AccountId, NearToken, Promise, Gas, PromiseResult};
use near_sdk::json_types::U128;
use near_sdk::collections::UnorderedSet;

macro_rules! require { ($condition:expr, $message:expr) => { if !$condition { env::panic_str($message) } }; }
const DAY_NS: u64 = 86_400_000_000_000;
#[near(contract_state)]
pub struct Vault {
    owner: AccountId, agent: AccountId, allowed: UnorderedSet<AccountId>,
    per_payment: u128, daily_limit: u128, spent: u128, day: u64,
    paused: bool, reserved: u128, next_id: u64,
}
impl Default for Vault { fn default() -> Self { env::panic_str("Initialize first") } }
#[near]
impl Vault {
    #[init]
    pub fn new(owner: AccountId, agent: AccountId, per_payment: U128, daily_limit: U128) -> Self {
        require!(!env::state_exists(), "Already initialized");
        require!(env::predecessor_account_id() == env::current_account_id(), "Only account can initialize");
        require!(per_payment.0 > 0 && daily_limit.0 >= per_payment.0, "Invalid limits");
        Self { owner, agent, allowed: UnorderedSet::new(b"a"), per_payment: per_payment.0, daily_limit: daily_limit.0, spent: 0, day: env::block_timestamp()/DAY_NS, paused: false, reserved: 0, next_id: 0 }
    }
    fn owner_only(&self) { require!(env::predecessor_account_id() == self.owner, "Owner only"); }
    #[payable]
    pub fn deposit(&mut self) { require!(env::attached_deposit().as_yoctonear() > 0, "Attach funds"); }
    pub fn set_policy(&mut self, agent: AccountId, per_payment: U128, daily_limit: U128, paused: bool) {
        self.owner_only(); require!(per_payment.0 > 0 && daily_limit.0 >= per_payment.0, "Invalid limits");
        self.agent=agent; self.per_payment=per_payment.0; self.daily_limit=daily_limit.0; self.paused=paused;
    }
    pub fn set_recipient(&mut self, account: AccountId, allowed: bool) {
        self.owner_only(); if allowed { self.allowed.insert(&account); } else { self.allowed.remove(&account); }
    }
    pub fn withdraw(&mut self, amount: U128) -> Promise {
        self.owner_only();
        let storage=env::storage_byte_cost().as_yoctonear().saturating_mul(env::storage_usage() as u128);
        require!(amount.0 > 0 && env::account_balance().as_yoctonear() >= storage.saturating_add(self.reserved).saturating_add(amount.0), "Insufficient available balance");
        Promise::new(self.owner.clone()).transfer(NearToken::from_yoctonear(amount.0))
    }
    pub fn pay(&mut self, receiver: AccountId, amount: U128) -> Promise {
        require!(env::predecessor_account_id() == self.agent, "Agent only");
        require!(!self.paused, "Vault paused"); require!(self.allowed.contains(&receiver), "Recipient not allowed");
        let a=amount.0; require!(a>0 && a<=self.per_payment, "Payment limit exceeded");
        let day=env::block_timestamp()/DAY_NS; if day!=self.day {self.day=day;self.spent=0;}
        require!(self.spent.checked_add(a).is_some_and(|v|v<=self.daily_limit), "Daily limit exceeded");
        // Preserve funds needed to cover contract storage; reserve all pending transfers.
        let storage=env::storage_byte_cost().as_yoctonear().saturating_mul(env::storage_usage() as u128);
        let balance=env::account_balance().as_yoctonear();
        require!(balance >= storage.saturating_add(self.reserved).saturating_add(a), "Insufficient available balance");
        self.spent+=a; self.reserved+=a; self.next_id+=1;
        env::log_str(&format!("PAYMENT_REQUEST id={} receiver={} amount={}",self.next_id,receiver,a));
        Promise::new(receiver).transfer(NearToken::from_yoctonear(a)).then(Self::ext(env::current_account_id()).with_static_gas(Gas::from_tgas(10)).settle(U128(a),self.next_id))
    }
    #[private]
    pub fn settle(&mut self, amount: U128, id: u64) -> bool {
        self.reserved=self.reserved.saturating_sub(amount.0);
        let success=matches!(env::promise_result(0),PromiseResult::Successful(_));
        // Spend is intentionally not refunded on failure: prevents retry abuse and cross-day accounting races.
        env::log_str(&format!("PAYMENT_RESULT id={} success={}",id,success)); success
    }
    pub fn status(&self) -> near_sdk::serde_json::Value {
        near_sdk::serde_json::json!({"owner":self.owner,"agent":self.agent,"per_payment":self.per_payment.to_string(),"daily_limit":self.daily_limit.to_string(),"spent":if env::block_timestamp()/DAY_NS==self.day {self.spent.to_string()} else {"0".into()},"paused":self.paused,"reserved":self.reserved.to_string(),"recipients":self.allowed.to_vec(),"balance":env::account_balance().as_yoctonear().to_string()})
    }
}


#[cfg(test)]
mod tests {
 use super::*; use near_sdk::{test_utils::VMContextBuilder,testing_env};
 fn context(who:&str){let mut c=VMContextBuilder::new();c.current_account_id("vault.testnet".parse().unwrap()).predecessor_account_id(who.parse().unwrap()).account_balance(NearToken::from_near(100));testing_env!(c.build());}
 fn vault()->Vault{context("vault.testnet");Vault::new("owner.testnet".parse().unwrap(),"agent.testnet".parse().unwrap(),U128(10),U128(20))}
 #[test] #[should_panic(expected="Owner only")] fn agent_cannot_change_policy(){let mut v=vault();context("agent.testnet");v.set_policy("agent.testnet".parse().unwrap(),U128(100),U128(100),false)}
 #[test] #[should_panic(expected="Recipient not allowed")] fn agent_cannot_pay_unknown(){let mut v=vault();context("agent.testnet");v.pay("bad.testnet".parse().unwrap(),U128(1));}
 #[test] #[should_panic(expected="Payment limit exceeded")] fn cannot_exceed_payment_limit(){let mut v=vault();context("owner.testnet");v.set_recipient("good.testnet".parse().unwrap(),true);context("agent.testnet");v.pay("good.testnet".parse().unwrap(),U128(11));}
 #[test] #[should_panic(expected="Vault paused")] fn pause_blocks_payment(){let mut v=vault();context("owner.testnet");v.set_policy("agent.testnet".parse().unwrap(),U128(10),U128(20),true);context("agent.testnet");v.pay("good.testnet".parse().unwrap(),U128(1));}
 #[test] #[should_panic(expected="Agent only")] fn stranger_cannot_pay(){let mut v=vault();context("stranger.testnet");v.pay("bad.testnet".parse().unwrap(),U128(1));}
}

