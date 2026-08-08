#![cfg(test)]
use crate::{StellarPassContract, StellarPassContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, Symbol,
};

fn make_client(env: &Env) -> StellarPassContractClient<'_> {
    let admin = Address::generate(env);
    StellarPassContractClient::new(env, &env.register(StellarPassContract, (&admin,)))
}

#[test]
fn test_create_community() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);

    let id = client.create_community(&admin, &Symbol::new(&env, "TestDAO"));
    assert_eq!(id, 1);

    let community = client.get_community(&1).unwrap();
    assert_eq!(community.admin, admin);
    assert_eq!(community.name, Symbol::new(&env, "TestDAO"));
}

#[test]
fn test_create_multiple_communities() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);

    let id1 = client.create_community(&admin, &Symbol::new(&env, "First"));
    let id2 = client.create_community(&admin, &Symbol::new(&env, "Second"));
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn test_grant_and_check_access() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Guild"));
    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );

    assert!(client.has_access(&member, &community_id));
    match client.get_role(&member, &community_id) {
        Some(record) => {
            assert_eq!(record.role, Symbol::new(&env, "member"));
            assert!(!record.revoked);
        }
        None => panic!("Expected role record"),
    }
}

#[test]
fn test_no_access_for_non_member() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Guild"));

    assert!(!client.has_access(&member, &community_id));
    assert!(client.get_role(&member, &community_id).is_none());
}

#[test]
fn test_revoke_access() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Guild"));
    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );

    assert!(client.has_access(&member, &community_id));

    client.revoke_role(&admin, &community_id, &member);
    assert!(!client.has_access(&member, &community_id));

    match client.get_role(&member, &community_id) {
        Some(record) => assert!(record.revoked),
        None => panic!("Record should still exist after revoke"),
    }
}

#[test]
fn test_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Timed"));
    let expires_at = 1000u64;

    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &Some(expires_at),
    );

    env.cost_estimate().budget().reset_unlimited();

    env.ledger().set_timestamp(999);
    assert!(client.has_access(&member, &community_id));

    env.ledger().set_timestamp(1000);
    assert!(!client.has_access(&member, &community_id));

    env.ledger().set_timestamp(1001);
    assert!(!client.has_access(&member, &community_id));
}

#[test]
fn test_no_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Forever"));

    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );

    env.cost_estimate().budget().reset_unlimited();
    env.ledger().set_timestamp(999999);
    assert!(client.has_access(&member, &community_id));
}

#[test]
fn test_grant_update_role_changes_event() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Evolved"));

    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );

    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "moderator"),
        &None,
    );

    match client.get_role(&member, &community_id) {
        Some(record) => assert_eq!(record.role, Symbol::new(&env, "moderator")),
        None => panic!("Expected role record"),
    }
}

#[test]
fn test_duplicate_grant_no_event() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Static"));

    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );

    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );

    assert!(client.has_access(&member, &community_id));
}

#[test]
fn test_regrant_after_revoke() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Return"));

    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );
    client.revoke_role(&admin, &community_id, &member);
    assert!(!client.has_access(&member, &community_id));

    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );
    assert!(client.has_access(&member, &community_id));

    match client.get_role(&member, &community_id) {
        Some(record) => assert!(!record.revoked),
        None => panic!("Expected role record"),
    }
}

#[test]
fn test_only_community_admin_can_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Exclusive"));

    let result = client.try_grant_role(
        &stranger,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );

    assert!(result.is_err());
}

#[test]
fn test_only_community_admin_can_revoke() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    let member = Address::generate(&env);

    let community_id = client.create_community(&admin, &Symbol::new(&env, "Exclusive"));
    client.grant_role(
        &admin,
        &community_id,
        &member,
        &Symbol::new(&env, "member"),
        &None,
    );

    let result = client.try_revoke_role(&stranger, &community_id, &member);
    assert!(result.is_err());
}

#[test]
fn test_upgrade_rejects_non_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let upgrade_admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    let contract_id = env.register(StellarPassContract, (&upgrade_admin,));
    let client = StellarPassContractClient::new(&env, &contract_id);

    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    let result = client.try_upgrade(&stranger, &wasm_hash);
    assert!(result.is_err());
}

#[test]
fn test_version() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);

    assert_eq!(client.version(), 1u32);
}
