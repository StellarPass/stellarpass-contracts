#![no_std]
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env,
    Symbol,
};

#[contracttype]
#[derive(Clone)]
pub enum InstanceKey {
    UpgradeAdmin,
    NextCommunityId,
}

#[contracttype]
#[derive(Clone)]
pub struct Community {
    pub id: u64,
    pub admin: Address,
    pub name: Symbol,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct RoleRecord {
    pub role: Symbol,
    pub expires_at: Option<u64>,
    pub revoked: bool,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum CommunityKey {
    ById(u64),
}

#[contracttype]
#[derive(Clone)]
pub struct MembershipKey {
    pub community_id: u64,
    pub member: Address,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    CommunityNotFound = 3,
    NotCommunityAdmin = 4,
    MemberNotFound = 5,
    UpgradeNotAuthorized = 6,
}

const TTL_THRESHOLD: u32 = 120 * 17280;
const TTL_EXTEND_TO: u32 = 180 * 17280;

#[contractevent]
pub struct CommunityCreated {
    #[topic]
    pub community_id: u64,
    #[topic]
    pub admin: Address,
    pub name: Symbol,
}

#[contractevent]
pub struct RoleGranted {
    #[topic]
    pub community_id: u64,
    #[topic]
    pub member: Address,
    pub role: Symbol,
    pub expires_at: Option<u64>,
}

#[contractevent]
pub struct RoleUpdated {
    #[topic]
    pub community_id: u64,
    #[topic]
    pub member: Address,
    pub old_role: Symbol,
    pub role: Symbol,
    pub expires_at: Option<u64>,
}

#[contractevent]
pub struct RoleRevoked {
    #[topic]
    pub community_id: u64,
    #[topic]
    pub member: Address,
}

#[contractevent]
pub struct ContractUpgraded {
    pub wasm_hash: BytesN<32>,
}

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

#[contract]
pub struct StellarPassContract;

#[contractimpl]
impl StellarPassContract {
    pub fn __constructor(env: Env, upgrade_admin: Address) {
        if env
            .storage()
            .instance()
            .get::<InstanceKey, Address>(&InstanceKey::UpgradeAdmin)
            .is_some()
        {
            panic!("Already initialized");
        }

        env.storage()
            .instance()
            .set(&InstanceKey::UpgradeAdmin, &upgrade_admin);
        env.storage()
            .instance()
            .set(&InstanceKey::NextCommunityId, &0u64);
        extend_instance_ttl(&env);
    }

    pub fn create_community(env: Env, admin: Address, name: Symbol) -> Result<u64, Error> {
        admin.require_auth();

        let next_id: u64 = env
            .storage()
            .instance()
            .get(&InstanceKey::NextCommunityId)
            .unwrap_or(0);

        let new_id = next_id + 1;
        env.storage()
            .instance()
            .set(&InstanceKey::NextCommunityId, &new_id);

        let community = Community {
            id: new_id,
            admin: admin.clone(),
            name: name.clone(),
            created_at: env.ledger().timestamp(),
        };

        let ckey = CommunityKey::ById(new_id);
        env.storage().persistent().set(&ckey, &community);
        env.storage()
            .persistent()
            .extend_ttl(&ckey, TTL_THRESHOLD, TTL_EXTEND_TO);

        CommunityCreated {
            community_id: new_id,
            admin,
            name,
        }
        .publish(&env);

        extend_instance_ttl(&env);
        Ok(new_id)
    }

    pub fn grant_role(
        env: Env,
        admin: Address,
        community_id: u64,
        member: Address,
        role: Symbol,
        expires_at: Option<u64>,
    ) -> Result<(), Error> {
        admin.require_auth();

        let ckey = CommunityKey::ById(community_id);
        let community = env
            .storage()
            .persistent()
            .get::<CommunityKey, Community>(&ckey)
            .ok_or(Error::CommunityNotFound)?;

        if community.admin != admin {
            return Err(Error::NotCommunityAdmin);
        }

        let mkey = MembershipKey {
            community_id,
            member: member.clone(),
        };

        if let Some(existing) = env
            .storage()
            .persistent()
            .get::<MembershipKey, RoleRecord>(&mkey)
        {
            if !existing.revoked && existing.role == role && existing.expires_at == expires_at {
                return Ok(());
            }

            let old_role = existing.role;
            let record = RoleRecord {
                role: role.clone(),
                expires_at,
                revoked: false,
                updated_at: env.ledger().timestamp(),
            };
            env.storage().persistent().set(&mkey, &record);
            env.storage()
                .persistent()
                .extend_ttl(&mkey, TTL_THRESHOLD, TTL_EXTEND_TO);

            RoleUpdated {
                community_id,
                member,
                old_role,
                role,
                expires_at,
            }
            .publish(&env);
        } else {
            let record = RoleRecord {
                role: role.clone(),
                expires_at,
                revoked: false,
                updated_at: env.ledger().timestamp(),
            };
            env.storage().persistent().set(&mkey, &record);
            env.storage()
                .persistent()
                .extend_ttl(&mkey, TTL_THRESHOLD, TTL_EXTEND_TO);

            RoleGranted {
                community_id,
                member,
                role,
                expires_at,
            }
            .publish(&env);
        }

        env.storage()
            .persistent()
            .extend_ttl(&ckey, TTL_THRESHOLD, TTL_EXTEND_TO);

        Ok(())
    }

    pub fn revoke_role(
        env: Env,
        admin: Address,
        community_id: u64,
        member: Address,
    ) -> Result<(), Error> {
        admin.require_auth();

        let ckey = CommunityKey::ById(community_id);
        let community = env
            .storage()
            .persistent()
            .get::<CommunityKey, Community>(&ckey)
            .ok_or(Error::CommunityNotFound)?;

        if community.admin != admin {
            return Err(Error::NotCommunityAdmin);
        }

        let mkey = MembershipKey {
            community_id,
            member: member.clone(),
        };

        let mut record = env
            .storage()
            .persistent()
            .get::<MembershipKey, RoleRecord>(&mkey)
            .ok_or(Error::MemberNotFound)?;

        if record.revoked {
            return Ok(());
        }

        record.revoked = true;
        record.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&mkey, &record);
        env.storage()
            .persistent()
            .extend_ttl(&mkey, TTL_THRESHOLD, TTL_EXTEND_TO);

        RoleRevoked {
            community_id,
            member,
        }
        .publish(&env);

        env.storage()
            .persistent()
            .extend_ttl(&ckey, TTL_THRESHOLD, TTL_EXTEND_TO);

        Ok(())
    }

    pub fn has_access(env: Env, member: Address, community_id: u64) -> Result<bool, Error> {
        let mkey = MembershipKey {
            community_id,
            member,
        };

        let record = match env
            .storage()
            .persistent()
            .get::<MembershipKey, RoleRecord>(&mkey)
        {
            Some(r) => r,
            None => return Ok(false),
        };

        if record.revoked {
            return Ok(false);
        }

        if let Some(exp) = record.expires_at {
            if exp <= env.ledger().timestamp() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn get_role(
        env: Env,
        member: Address,
        community_id: u64,
    ) -> Result<Option<RoleRecord>, Error> {
        let mkey = MembershipKey {
            community_id,
            member,
        };

        Ok(env
            .storage()
            .persistent()
            .get::<MembershipKey, RoleRecord>(&mkey))
    }

    pub fn get_community(env: Env, community_id: u64) -> Result<Option<Community>, Error> {
        let key = CommunityKey::ById(community_id);
        Ok(env.storage().persistent().get::<CommunityKey, Community>(&key))
    }

    pub fn upgrade(
        env: Env,
        upgrade_admin: Address,
        wasm_hash: BytesN<32>,
    ) -> Result<(), Error> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&InstanceKey::UpgradeAdmin)
            .ok_or(Error::NotInitialized)?;

        upgrade_admin.require_auth();

        if stored_admin != upgrade_admin {
            return Err(Error::UpgradeNotAuthorized);
        }

        ContractUpgraded {
            wasm_hash: wasm_hash.clone(),
        }
        .publish(&env);

        env.deployer().update_current_contract_wasm(wasm_hash);

        Ok(())
    }

    pub fn version() -> u32 {
        1
    }
}

mod test;
