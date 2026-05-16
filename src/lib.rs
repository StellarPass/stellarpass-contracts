#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    NextId,
    Owner(u64),
    Meta(u64),
}

#[contracttype]
#[derive(Clone)]
pub struct CredentialMetadata {
    pub institution: String,
    pub course: String,
    pub issued_at: u64,
    pub credential_hash: BytesN<32>,
}

#[contract]
pub struct CredChainContract;

#[contractimpl]
impl CredChainContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextId, &1u64);
    }

    pub fn issue(
        env: Env,
        recipient: Address,
        institution: String,
        course: String,
        issued_at: u64,
        credential_hash: BytesN<32>,
    ) -> u64 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut next_id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(1u64);
        let token_id = next_id;
        next_id += 1;
        env.storage().instance().set(&DataKey::NextId, &next_id);

        let meta = CredentialMetadata {
            institution,
            course,
            issued_at,
            credential_hash,
        };

        env.storage().persistent().set(&DataKey::Owner(token_id), &recipient);
        env.storage().persistent().set(&DataKey::Meta(token_id), &meta);
        token_id
    }

    pub fn owner_of(env: Env, token_id: u64) -> Address {
        env.storage().persistent().get(&DataKey::Owner(token_id)).unwrap()
    }

    pub fn metadata(env: Env, token_id: u64) -> CredentialMetadata {
        env.storage().persistent().get(&DataKey::Meta(token_id)).unwrap()
    }

    pub fn verify_hash(env: Env, token_id: u64, credential_hash: BytesN<32>) -> bool {
        let meta: CredentialMetadata = env.storage().persistent().get(&DataKey::Meta(token_id)).unwrap();
        meta.credential_hash == credential_hash
    }
}

#[cfg(test)]
mod test;

