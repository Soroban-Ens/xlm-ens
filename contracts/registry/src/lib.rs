mod test;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};
use xlm_ns_common::soroban::validate_fqdn_soroban;
use xlm_ns_common::{DEFAULT_TTL_SECONDS, MAX_METADATA_URI_LENGTH};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RegistryEntry {
    pub name: String,
    pub owner: Address,
    pub resolver: Option<String>,
    pub target_address: Option<String>,
    pub metadata_uri: Option<String>,
    pub ttl_seconds: u64,
    pub registered_at: u64,
    pub expires_at: u64,
    pub grace_period_ends_at: u64,
    pub transfer_count: u32,
}

impl RegistryEntry {
    fn is_active_at(&self, now_unix: u64) -> bool {
        now_unix <= self.expires_at
    }

    fn is_claimable_at(&self, now_unix: u64) -> bool {
        now_unix > self.grace_period_ends_at
    }
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Entry(String),
    OwnerNames(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    AlreadyRegistered = 1,
    NotFound = 2,
    NotYetClaimable = 3,
    NotActive = 4,
    Unauthorized = 5,
    MetadataTooLong = 6,
    Validation = 7,
    InvalidExpiry = 8,
    InvalidGracePeriod = 9,
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    pub fn register(
        env: Env,
        name: String,
        owner: Address,
        target_address: Option<String>,
        metadata_uri: Option<String>,
        now_unix: u64,
        expires_at: u64,
        grace_period_ends_at: u64,
    ) -> Result<(), RegistryError> {
        validate_fqdn_soroban(&name).map_err(|_| RegistryError::Validation)?;
        validate_metadata(&metadata_uri)?;
        validate_lifecycle_timestamps(now_unix, expires_at, grace_period_ends_at)?;

        let key = DataKey::Entry(name.clone());
        if let Some(existing) = env.storage().persistent().get::<_, RegistryEntry>(&key) {
            if existing.is_active_at(now_unix) {
                return Err(RegistryError::AlreadyRegistered);
            }
            if !existing.is_claimable_at(now_unix) {
                return Err(RegistryError::NotYetClaimable);
            }
            remove_owner_name(&env, &existing.owner, &name);
            env.storage().persistent().remove(&key);
        }

        let entry = RegistryEntry {
            name: name.clone(),
            owner: owner.clone(),
            resolver: None,
            target_address,
            metadata_uri,
            ttl_seconds: DEFAULT_TTL_SECONDS,
            registered_at: now_unix,
            expires_at,
            grace_period_ends_at,
            transfer_count: 0,
        };
        env.storage().persistent().set(&key, &entry);
        add_owner_name(&env, &owner, &name);
        Ok(())
    }

    pub fn resolve(env: Env, name: String, now_unix: u64) -> Result<RegistryEntry, RegistryError> {
        validate_fqdn_soroban(&name).map_err(|_| RegistryError::Validation)?;
        let entry = get_entry(&env, &name)?;
        if !entry.is_active_at(now_unix) {
            return Err(RegistryError::NotActive);
        }
        Ok(entry)
    }

    pub fn transfer(
        env: Env,
        name: String,
        caller: Address,
        new_owner: Address,
        now_unix: u64,
    ) -> Result<(), RegistryError> {
        let mut entry = get_entry(&env, &name)?;
        ensure_owner(&entry, &caller, now_unix)?;
        let old_owner = entry.owner.clone();
        entry.owner = new_owner.clone();
        entry.transfer_count = entry.transfer_count.saturating_add(1);
        put_entry(&env, &name, &entry);
        remove_owner_name(&env, &old_owner, &name);
        add_owner_name(&env, &new_owner, &name);
        Ok(())
    }

    pub fn set_resolver(
        env: Env,
        name: String,
        caller: Address,
        resolver: Option<String>,
        now_unix: u64,
    ) -> Result<(), RegistryError> {
        let mut entry = get_entry(&env, &name)?;
        ensure_owner(&entry, &caller, now_unix)?;
        entry.resolver = resolver;
        put_entry(&env, &name, &entry);
        Ok(())
    }

    pub fn set_target_address(
        env: Env,
        name: String,
        caller: Address,
        target_address: Option<String>,
        now_unix: u64,
    ) -> Result<(), RegistryError> {
        let mut entry = get_entry(&env, &name)?;
        ensure_owner(&entry, &caller, now_unix)?;
        entry.target_address = target_address;
        put_entry(&env, &name, &entry);
        Ok(())
    }

    pub fn set_metadata(
        env: Env,
        name: String,
        caller: Address,
        metadata_uri: Option<String>,
        now_unix: u64,
    ) -> Result<(), RegistryError> {
        validate_metadata(&metadata_uri)?;
        let mut entry = get_entry(&env, &name)?;
        ensure_owner(&entry, &caller, now_unix)?;
        entry.metadata_uri = metadata_uri;
        put_entry(&env, &name, &entry);
        Ok(())
    }

    pub fn renew(
        env: Env,
        name: String,
        caller: Address,
        expires_at: u64,
        grace_period_ends_at: u64,
        now_unix: u64,
    ) -> Result<(), RegistryError> {
        let mut entry = get_entry(&env, &name)?;
        ensure_owner(&entry, &caller, now_unix)?;
        validate_lifecycle_timestamps(now_unix, expires_at, grace_period_ends_at)?;
        entry.expires_at = expires_at;
        entry.grace_period_ends_at = grace_period_ends_at;
        put_entry(&env, &name, &entry);
        Ok(())
    }

    pub fn names_for_owner(env: Env, owner: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::OwnerNames(owner))
            .unwrap_or(Vec::new(&env))
    }
}

fn get_entry(env: &Env, name: &String) -> Result<RegistryEntry, RegistryError> {
    env.storage()
        .persistent()
        .get(&DataKey::Entry(name.clone()))
        .ok_or(RegistryError::NotFound)
}

fn put_entry(env: &Env, name: &String, entry: &RegistryEntry) {
    env.storage()
        .persistent()
        .set(&DataKey::Entry(name.clone()), entry);
}

fn validate_metadata(metadata_uri: &Option<String>) -> Result<(), RegistryError> {
    if metadata_uri
        .as_ref()
        .map(|value| value.len() as usize > MAX_METADATA_URI_LENGTH)
        .unwrap_or(false)
    {
        return Err(RegistryError::MetadataTooLong);
    }

    Ok(())
}

fn validate_lifecycle_timestamps(
    now_unix: u64,
    expires_at: u64,
    grace_period_ends_at: u64,
) -> Result<(), RegistryError> {
    if expires_at < now_unix {
        return Err(RegistryError::InvalidExpiry);
    }

    if grace_period_ends_at < expires_at {
        return Err(RegistryError::InvalidGracePeriod);
    }

    Ok(())
}

fn ensure_owner(
    entry: &RegistryEntry,
    caller: &Address,
    now_unix: u64,
) -> Result<(), RegistryError> {
    if !entry.is_active_at(now_unix) {
        return Err(RegistryError::NotActive);
    }
    if entry.owner != *caller {
        return Err(RegistryError::Unauthorized);
    }

    Ok(())
}

fn add_owner_name(env: &Env, owner: &Address, name: &String) {
    let key = DataKey::OwnerNames(owner.clone());
    let mut names: Vec<String> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));
    if !names.contains(name) {
        names.push_back(name.clone());
        env.storage().persistent().set(&key, &names);
    }
}

fn remove_owner_name(env: &Env, owner: &Address, name: &String) {
    let key = DataKey::OwnerNames(owner.clone());
    let names: Vec<String> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env));
    let mut filtered = Vec::new(env);
    for existing in names.iter() {
        if existing != *name {
            filtered.push_back(existing);
        }
    }
    env.storage().persistent().set(&key, &filtered);
}
