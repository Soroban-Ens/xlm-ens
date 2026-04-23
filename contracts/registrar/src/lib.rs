pub mod expiry;
pub mod pricing;
mod test;

use expiry::expiry_from_now;
use pricing::price_for_label_length;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, IntoVal, String, Symbol,
};
use xlm_ns_common::soroban::{
    build_xlm_name, extract_label_soroban, validate_label_soroban,
    validate_registration_years_soroban,
};
use xlm_ns_common::GRACE_PERIOD_SECONDS;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RegistrationQuote {
    pub fee_stroops: u64,
    pub expiry_unix: u64,
    pub grace_period_ends_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RegistrationRecord {
    pub name: String,
    pub owner: Address,
    pub registered_at: u64,
    pub expires_at: u64,
    pub grace_period_ends_at: u64,
    pub fee_paid: u64,
    pub renewed_at: u64,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Registration(String),
    Reserved(String),
    Treasury,
    Registry,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistrarError {
    InsufficientFee = 1,
    NotFound = 2,
    NotRenewable = 3,
    AlreadyRegistered = 4,
    Reserved = 5,
    Unauthorized = 6,
    Validation = 7,
    RegistrationClaimable = 8,
    NotInitialized = 9,
}

#[contract]
pub struct RegistrarContract;

#[contractimpl]
impl RegistrarContract {
    pub fn initialize(env: Env, registry: Address) -> Result<(), RegistrarError> {
        if env.storage().instance().has(&DataKey::Registry) {
            return Err(RegistrarError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Registry, &registry);
        Ok(())
    }

    pub fn reserve_label(env: Env, label: String) -> Result<(), RegistrarError> {
        validate_label_soroban(&label).map_err(|_| RegistrarError::Validation)?;
        env.storage()
            .persistent()
            .set(&DataKey::Reserved(label), &true);
        Ok(())
    }

    pub fn quote_registration(
        _env: Env,
        label: String,
        years: u64,
        now_unix: u64,
    ) -> Result<RegistrationQuote, RegistrarError> {
        validate_label_soroban(&label).map_err(|_| RegistrarError::Validation)?;
        validate_registration_years_soroban(years).map_err(|_| RegistrarError::Validation)?;
        Ok(build_quote(&label, years, now_unix))
    }

    pub fn register(
        env: Env,
        label: String,
        owner: Address,
        years: u64,
        payment_stroops: u64,
        now_unix: u64,
    ) -> Result<(), RegistrarError> {
        validate_label_soroban(&label).map_err(|_| RegistrarError::Validation)?;
        validate_registration_years_soroban(years).map_err(|_| RegistrarError::Validation)?;

        if env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::Reserved(label.clone()))
            .unwrap_or(false)
        {
            return Err(RegistrarError::Reserved);
        }

        let quote = build_quote(&label, years, now_unix);
        if payment_stroops < quote.fee_stroops {
            return Err(RegistrarError::InsufficientFee);
        }

        let name = build_xlm_name(&env, &label).map_err(|_| RegistrarError::Validation)?;
        if let Some(existing) = env
            .storage()
            .persistent()
            .get::<_, RegistrationRecord>(&DataKey::Registration(name.clone()))
        {
            if now_unix <= existing.grace_period_ends_at {
                return Err(RegistrarError::AlreadyRegistered);
            }
        }

        let record = RegistrationRecord {
            name: name.clone(),
            owner: owner.clone(),
            registered_at: now_unix,
            expires_at: quote.expiry_unix,
            grace_period_ends_at: quote.grace_period_ends_at,
            fee_paid: payment_stroops,
            renewed_at: now_unix,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Registration(name.clone()), &record);
        let treasury = env
            .storage()
            .persistent()
            .get::<_, u64>(&DataKey::Treasury)
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::Treasury,
            &treasury.saturating_add(payment_stroops),
        );

        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::Registry)
            .ok_or(RegistrarError::NotInitialized)?;

        env.invoke_contract::<()>(
            &registry,
            &Symbol::new(&env, "register"),
            (
                name,
                owner,
                Option::<String>::None,
                Option::<String>::None,
                now_unix,
                record.expires_at,
                record.grace_period_ends_at,
            )
                .into_val(&env),
        );

        Ok(())
    }

    pub fn renew(
        env: Env,
        name: String,
        caller: Address,
        years: u64,
        payment_stroops: u64,
        now_unix: u64,
    ) -> Result<(), RegistrarError> {
        let label = extract_label_soroban(&env, &name).map_err(|_| RegistrarError::Validation)?;
        validate_registration_years_soroban(years).map_err(|_| RegistrarError::Validation)?;

        let mut record = env
            .storage()
            .persistent()
            .get::<_, RegistrationRecord>(&DataKey::Registration(name.clone()))
            .ok_or(RegistrarError::NotFound)?;
        if record.owner != caller {
            return Err(RegistrarError::Unauthorized);
        }
        match can_renew(record.expires_at, now_unix) {
            Ok(true) => {}
            Ok(false) => return Err(RegistrarError::NotRenewable),
            Err(e) => return Err(e),
        }

        let fee_due = price_for_label_length(label.len() as usize).saturating_mul(years);
        if payment_stroops < fee_due {
            return Err(RegistrarError::InsufficientFee);
        }

        let base_time = if record.expires_at > now_unix {
            record.expires_at
        } else {
            now_unix
        };
        let expires_at = expiry_from_now(base_time, years);
        record.expires_at = expires_at;
        record.grace_period_ends_at = expires_at.saturating_add(GRACE_PERIOD_SECONDS);
        record.renewed_at = now_unix;
        record.fee_paid = record.fee_paid.saturating_add(payment_stroops);
        env.storage()
            .persistent()
            .set(&DataKey::Registration(name.clone()), &record);

        let treasury = env
            .storage()
            .persistent()
            .get::<_, u64>(&DataKey::Treasury)
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::Treasury,
            &treasury.saturating_add(payment_stroops),
        );

        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::Registry)
            .ok_or(RegistrarError::NotInitialized)?;

        env.invoke_contract::<()>(
            &registry,
            &Symbol::new(&env, "renew"),
            (
                name,
                caller,
                record.expires_at,
                record.grace_period_ends_at,
                now_unix,
            )
                .into_val(&env),
        );

        Ok(())
    }

    pub fn registration(env: Env, name: String) -> Option<RegistrationRecord> {
        env.storage().persistent().get(&DataKey::Registration(name))
    }

    pub fn is_available(env: Env, label: String, now_unix: u64) -> bool {
        let name = match build_xlm_name(&env, &label) {
            Ok(name) => name,
            Err(_) => return false,
        };
        env.storage()
            .persistent()
            .get::<_, RegistrationRecord>(&DataKey::Registration(name))
            .map(|record| now_unix > record.grace_period_ends_at)
            .unwrap_or(true)
    }

    pub fn treasury_balance(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Treasury)
            .unwrap_or(0)
    }
}

fn build_quote(label: &String, years: u64, now_unix: u64) -> RegistrationQuote {
    let annual_fee = price_for_label_length(label.len() as usize);
    let expiry_unix = expiry_from_now(now_unix, years);

    RegistrationQuote {
        fee_stroops: annual_fee.saturating_mul(years),
        expiry_unix,
        grace_period_ends_at: expiry_unix.saturating_add(GRACE_PERIOD_SECONDS),
    }
}

pub fn can_renew(expiry_unix: u64, now_unix: u64) -> Result<bool, RegistrarError> {
    let grace_period_end = expiry_unix.saturating_add(GRACE_PERIOD_SECONDS);

    if now_unix > grace_period_end {
        return Err(RegistrarError::RegistrationClaimable);
    }

    Ok(now_unix <= grace_period_end)
}
