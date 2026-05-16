#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

#[test]
fn issues_and_verifies_credential() {
    let env = Env::default();
    let contract_id = env.register(CredChainContract, ());
    let client = CredChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    let institution = String::from_str(&env, "Accra Tech Institute");
    let course = String::from_str(&env, "Backend Engineering");
    let hash = BytesN::from_array(&env, &[7u8; 32]);

    env.mock_all_auths();
    let token_id = client.issue(&student, &institution, &course, &1735689600u64, &hash);

    assert_eq!(token_id, 1u64);
    assert_eq!(client.owner_of(&token_id), student);
    assert!(client.verify_hash(&token_id, &hash));
}

