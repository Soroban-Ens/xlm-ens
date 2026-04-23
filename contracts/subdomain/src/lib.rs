mod test;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};
use xlm_ns_common::soroban::{build_subdomain_name, validate_fqdn_soroban};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ParentDomain {
    pub owner: Address,
    pub controllers: Vec<Address>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SubdomainRecord {
    pub parent: String,
    pub owner: Address,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Parent(String),
    Subdomain(String),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SubdomainError {
    Validation = 1,
    ParentNotFound = 2,
    AlreadyExists = 3,
    NotFound = 4,
    Unauthorized = 5,
}

#[contract]
pub struct SubdomainContract;

#[contractimpl]
impl SubdomainContract {
    pub fn register_parent(env: Env, parent: String, owner: Address) -> Result<(), SubdomainError> {
        validate_fqdn_soroban(&parent).map_err(|_| SubdomainError::Validation)?;
        let record = ParentDomain {
            owner,
            controllers: Vec::new(&env),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Parent(parent), &record);
        Ok(())
    }

    pub fn add_controller(
        env: Env,
        parent: String,
        caller: Address,
        controller: Address,
    ) -> Result<(), SubdomainError> {
        let mut parent_record = get_parent(&env, &parent)?;
        if parent_record.owner != caller {
            return Err(SubdomainError::Unauthorized);
        }
        if !parent_record.controllers.contains(&controller) {
            parent_record.controllers.push_back(controller);
            env.storage()
                .persistent()
                .set(&DataKey::Parent(parent), &parent_record);
        }
        Ok(())
    }

    pub fn create(
        env: Env,
        label: String,
        parent: String,
        caller: Address,
        owner: Address,
        now_unix: u64,
    ) -> Result<String, SubdomainError> {
        let parent_record = get_parent(&env, &parent)?;
        if parent_record.owner != caller && !parent_record.controllers.contains(&caller) {
            return Err(SubdomainError::Unauthorized);
        }

        let fqdn =
            build_subdomain_name(&env, &label, &parent).map_err(|_| SubdomainError::Validation)?;
        let key = DataKey::Subdomain(fqdn.clone());
        if env.storage().persistent().has(&key) {
            return Err(SubdomainError::AlreadyExists);
        }

        let record = SubdomainRecord {
            parent,
            owner,
            created_at: now_unix,
        };
        env.storage().persistent().set(&key, &record);
        Ok(fqdn)
    }

    pub fn transfer(
        env: Env,
        fqdn: String,
        caller: Address,
        new_owner: Address,
    ) -> Result<(), SubdomainError> {
        let mut record = get_subdomain(&env, &fqdn)?;
        if record.owner != caller {
            return Err(SubdomainError::Unauthorized);
        }
        record.owner = new_owner;
        env.storage()
            .persistent()
            .set(&DataKey::Subdomain(fqdn), &record);
        Ok(())
    }

    pub fn exists(env: Env, fqdn: String) -> bool {
        env.storage().persistent().has(&DataKey::Subdomain(fqdn))
    }

    pub fn parent(env: Env, parent: String) -> Option<ParentDomain> {
        env.storage().persistent().get(&DataKey::Parent(parent))
    }

    pub fn record(env: Env, fqdn: String) -> Option<SubdomainRecord> {
        env.storage().persistent().get(&DataKey::Subdomain(fqdn))
    }
}

fn get_parent(env: &Env, parent: &String) -> Result<ParentDomain, SubdomainError> {
    env.storage()
        .persistent()
        .get(&DataKey::Parent(parent.clone()))
        .ok_or(SubdomainError::ParentNotFound)
}

fn get_subdomain(env: &Env, fqdn: &String) -> Result<SubdomainRecord, SubdomainError> {
    env.storage()
        .persistent()
        .get(&DataKey::Subdomain(fqdn.clone()))
        .ok_or(SubdomainError::NotFound)
}
