mod test;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Map, String};
use xlm_ns_common::soroban::validate_fqdn_soroban;
use xlm_ns_common::MAX_TEXT_RECORDS;

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
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ResolverError {
    Validation = 1,
    RecordNotFound = 2,
    Unauthorized = 3,
    TooManyTextRecords = 4,
}

#[contractclient(name = "ResolverContractClient")]
#[contract]
pub struct ResolverContract;

#[contractimpl]
impl ResolverContract {
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
        let mut record = get_record(&env, &name)?;
        if record.owner != caller {
            return Err(ResolverError::Unauthorized);
        }
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
        let record = get_record(&env, &name)?;
        if record.owner != caller || record.address != address {
            return Err(ResolverError::Unauthorized);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Primary(record.address.clone()), &name);
        Ok(())
    }

    pub fn remove_record(env: Env, name: String, caller: Address) -> Result<(), ResolverError> {
        let record = get_record(&env, &name)?;
        if record.owner != caller {
            return Err(ResolverError::Unauthorized);
        }
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

fn get_record(env: &Env, name: &String) -> Result<ResolutionRecord, ResolverError> {
    env.storage()
        .persistent()
        .get(&DataKey::Forward(name.clone()))
        .ok_or(ResolverError::RecordNotFound)
}

fn put_record(env: &Env, name: &String, record: &ResolutionRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Forward(name.clone()), record);
}
