#![cfg(test)]
use super::*;
use soroban_sdk::{Env, Address, String};

#[test]
fn test_happy_path_end_to_end() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, LocalLedgerContract);
    let client = LocalLedgerContractClient::new(&env, &contract_id);

    let resident = Address::generate(&env);
    let vendor = Address::generate(&env);

    // Step 1: Deposit funds
    let total_pool = client.deposit(&resident, &10000);
    assert_eq!(total_pool, 10000);

    // Step 2: Propose project
    let desc = String::from_str(&env, "Community Fence Repair");
    client.propose(&resident, &1, &desc, &4500, &vendor);

    // Step 3: Vote and establish consensus majority
    client.vote(&resident, &1, &true);

    // Step 4: Execute payout
    client.execute(&resident, &1);
}

#[test]
#[should_panic(expected = "Insufficient vault balance for proposal budget")]
fn test_edge_case_insufficient_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, LocalLedgerContract);
    let client = LocalLedgerContractClient::new(&env, &contract_id);

    let resident = Address::generate(&env);
    let vendor = Address::generate(&env);

    client.deposit(&resident, &500);
    let desc = String::from_str(&env, "Pool Overhaul Funding");
    
    // Panic triggered: 1200 exceeds total vault balance of 500
    client.propose(&resident, &1, &desc, &1200, &vendor);
}

#[test]
fn test_state_verification_post_transaction() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, LocalLedgerContract);
    let client = LocalLedgerContractClient::new(&env, &contract_id);

    let resident = Address::generate(&env);
    let vendor = Address::generate(&env);

    client.deposit(&resident, &5000);
    let desc = String::from_str(&env, "Security Cameras Setup");
    client.propose(&resident, &42, &desc, &2000, &vendor);
    
    let updated_prop = client.vote(&resident, &42, &true);
    assert_eq!(updated_prop.yes_votes, 1);
    assert_eq!(updated_prop.no_votes, 0);
    assert_eq!(updated_prop.executed, false);
}