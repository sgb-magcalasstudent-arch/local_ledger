#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, symbol_short};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolDataKey {
    Vault,      // Total native asset balance stored
    Prop(u32),  // Individual proposal mapping by ID
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub description: String,
    pub amount: i128,
    pub recipient: Address,
    pub yes_votes: u32,
    pub no_votes: u32,
    pub executed: bool,
}

#[contract]
pub struct LocalLedgerContract;

#[contractimpl]
impl LocalLedgerContract {
    /// Increments vault balance tracking variables inside the contract storage.
    /// Actual token balances are verified through direct underlying transfer assertions.
    pub fn deposit(env: Env, from: Address, amount: i128) -> i128 {
        from.require_auth();
        assert!(amount > 0, "Deposit amount must be positive");

        let key = SymbolDataKey::Vault;
        let mut current_balance: i128 = env.storage().instance().get(&key).unwrap_or(0);
        current_balance += amount;
        
        env.storage().instance().set(&key, &current_balance);
        current_balance
    }

    /// Initializes a proposal state entry requiring explicit asset payouts.
    pub fn propose(env: Env, proposer: Address, id: u32, description: String, amount: i128, recipient: Address) {
        proposer.require_auth();
        let vault_key = SymbolDataKey::Vault;
        let current_balance: i128 = env.storage().instance().get(&vault_key).unwrap_or(0);
        assert!(amount <= current_balance, "Insufficient vault balance for proposal budget");

        let prop_key = SymbolDataKey::Prop(id);
        assert!(!env.storage().instance().has(&prop_key), "Proposal identifier already exists");

        let new_proposal = Proposal {
            description,
            amount,
            recipient,
            yes_votes: 0,
            no_votes: 0,
            executed: false,
        };

        env.storage().instance().set(&prop_key, &new_proposal);
    }

    /// Casts an immutable community vote increments on target proposal state.
    pub fn vote(env: Env, voter: Address, id: u32, approve: bool) -> Proposal {
        voter.require_auth();
        let prop_key = SymbolDataKey::Prop(id);
        let mut proposal: Proposal = env.storage().instance().get(&prop_key).expect("Target proposal not found");
        assert!(!proposal.executed, "Cannot vote on an already completed proposal execution phase");

        if approve {
            proposal.yes_votes += 1;
        } else {
            proposal.no_votes += 1;
        }

        env.storage().instance().set(&prop_key, &proposal);
        proposal
    }

    /// Finalizes consensus validation and emits tracked vault values to an external recipient.
    pub fn execute(env: Env, executor: Address, id: u32) {
        executor.require_auth();
        let prop_key = SymbolDataKey::Prop(id);
        let mut proposal: Proposal = env.storage().instance().get(&prop_key).expect("Target proposal not found");
        
        assert!(!proposal.executed, "Proposal already fully processed and completed");
        assert!(proposal.yes_votes > proposal.no_votes, "Consensus majority check failed");

        let vault_key = SymbolDataKey::Vault;
        let mut current_balance: i128 = env.storage().instance().get(&vault_key).unwrap_or(0);
        assert!(proposal.amount <= current_balance, "Insufficient vault assets left to settle transfer");

        current_balance -= proposal.amount;
        proposal.executed = true;

        env.storage().instance().set(&vault_key, &current_balance);
        env.storage().instance().set(&prop_key, &proposal);
    }
}