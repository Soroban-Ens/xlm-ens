mod test;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Map, String, Symbol};
use xlm_ns_common::soroban::validate_fqdn_soroban;
use xlm_ns_common::MAX_TEXT_RECORDS;
use xlm_ns_common::RegistryEntry;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ResolutionRecord {
    pub owner: Address,
    pub address: String,
    pub text_records: Map<String, String>,
    pub updated_at: u64,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Forward(String),
    Reverse(String),
    Primary(String),
    Registry,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ResolverError {
    Validation = 1,
    RecordNotFound = 2,
    Unauthorized = 3,
    TooManyTextRecords = 4,
    NotInitialized = 5,
}

#[contractclient(name = "ResolverContractClient")]
#[contract]
pub struct ResolverContract;

#[contractimpl]
impl ResolverContract {
    pub fn initialize(env: Env, registry: Address) -> Result<(), ResolverError> {
        if env.storage().instance().has(&DataKey::Registry) {
            return Err(ResolverError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Registry, &registry);
        Ok(())
    }

    pub fn set_record(
        env: Env,
        name: String,
        owner: Address,
        address: String,
        now_unix: u64,
    ) -> Result<(), ResolverError> {
        validate_fqdn_soroban(&name).map_err(|_| ResolverError::Validation)?;
        let text_records = match get_record(&env, &name) {
            Ok(existing) => existing.text_records,
            Err(_) => Map::new(&env),
        };
        let record = ResolutionRecord {
            owner,
            address: address.clone(),
            text_records,
            updated_at: now_unix,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Forward(name.clone()), &record);
        env.storage()
            .persistent()
            .set(&DataKey::Reverse(address), &name);
        Ok(())
    }

    pub fn set_text_record(
        env: Env,
        name: String,
        caller: Address,
        key: String,
        value: String,
        now_unix: u64,
    ) -> Result<(), ResolverError> {
        let registry = get_registry(&env)?;
        let registry_entry = env.invoke_contract::<RegistryEntry>(
            &registry,
            &Symbol::new(&env, "resolve"),
            (name.clone(), now_unix).into_val(&env),
        ).map_err(|_| ResolverError::Unauthorized)?;
        if registry_entry.owner != caller {
            return Err(ResolverError::Unauthorized);
        }
        let mut record = get_record(&env, &name)?;
        if !record.text_records.contains_key(key.clone())
            && record.text_records.len() >= MAX_TEXT_RECORDS as u32
        {
            return Err(ResolverError::TooManyTextRecords);
        }
        record.text_records.set(key, value);
        record.updated_at = now_unix;
        put_record(&env, &name, &record);
        Ok(())
    }

    pub fn set_primary_name(
        env: Env,
        address: String,
        caller: Address,
        name: String,
    ) -> Result<(), ResolverError> {
        let registry = get_registry(&env)?;
        let registry_entry = env.invoke_contract::<RegistryEntry>(
            &registry,
            &Symbol::new(&env, "resolve"),
            (name.clone(), 0).into_val(&env), // now_unix not needed for owner check
        ).map_err(|_| ResolverError::Unauthorized)?;
        let record = get_record(&env, &name)?;
        if registry_entry.owner != caller || record.address != address {
            return Err(ResolverError::Unauthorized);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Primary(record.address.clone()), &name);
        Ok(())
    }

    pub fn remove_record(env: Env, name: String, caller: Address) -> Result<(), ResolverError> {
        let registry = get_registry(&env)?;
        let registry_entry = env.invoke_contract::<RegistryEntry>(
            &registry,
            &Symbol::new(&env, "resolve"),
            (name.clone(), 0).into_val(&env),
        ).map_err(|_| ResolverError::Unauthorized)?;
        if registry_entry.owner != caller {
            return Err(ResolverError::Unauthorized);
        }
        let record = get_record(&env, &name)?;
        env.storage()
            .persistent()
            .remove(&DataKey::Forward(name.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::Reverse(record.address.clone()));
        env.storage()
            .persistent()
            .remove(&DataKey::Primary(record.address));
        Ok(())
    }

    pub fn update_owner(env: Env, name: String, new_owner: Address) -> Result<(), ResolverError> {
        let mut record = get_record(&env, &name)?;
        record.owner = new_owner;
        put_record(&env, &name, &record);
        Ok(())
    }

    pub fn resolve(env: Env, name: String) -> Option<ResolutionRecord> {
        env.storage().persistent().get(&DataKey::Forward(name))
    }

    pub fn has_record(env: Env, name: String) -> bool {
        env.storage().persistent().has(&DataKey::Forward(name))
    }

    pub fn reverse(env: Env, address: String) -> Option<String> {
        env.storage()
            .persistent()
            .get(&DataKey::Primary(address.clone()))
            .or_else(|| env.storage().persistent().get(&DataKey::Reverse(address)))
    }

    pub fn transfer_record_owner(
        env: Env,
        name: String,
        caller: Address,
        new_owner: Address,
    ) -> Result<(), ResolverError> {
        let mut record = get_record(&env, &name)?;
        if record.owner != caller {
            return Err(ResolverError::Unauthorized);
        }
        record.owner = new_owner;
        put_record(&env, &name, &record);
        Ok(())
    }
}

fn get_registry(env: &Env) -> Result<Address, ResolverError> {
    env.storage().instance().get(&DataKey::Registry).ok_or(ResolverError::NotInitialized)
}

fn put_record(env: &Env, name: &String, record: &ResolutionRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Forward(name.clone()), record);
}
