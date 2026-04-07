pub mod forward;
pub mod reverse;
pub mod test;

use std::collections::{BTreeMap, HashMap};

use xlm_ns_common::validation::{parse_fqdn, validate_owner};
use xlm_ns_common::{CommonError, MAX_TEXT_RECORDS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionRecord {
    pub owner: String,
    pub address: String,
    pub text_records: BTreeMap<String, String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolverError {
    Validation(CommonError),
    RecordNotFound,
    Unauthorized,
    TooManyTextRecords,
}

impl core::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Validation(error) => write!(f, "{error}"),
            Self::RecordNotFound => f.write_str("resolver record was not found"),
            Self::Unauthorized => f.write_str("caller is not authorized for this resolver record"),
            Self::TooManyTextRecords => f.write_str("resolver record exceeded the text record limit"),
        }
    }
}

impl std::error::Error for ResolverError {}

#[derive(Debug, Default)]
pub struct ResolverContract {
    forward_records: HashMap<String, ResolutionRecord>,
    reverse_records: HashMap<String, String>,
    primary_names: HashMap<String, String>,
}

impl ResolverContract {
    pub fn set_record(
        &mut self,
        name: impl Into<String>,
        owner: impl Into<String>,
        address: impl Into<String>,
        now_unix: u64,
    ) -> Result<(), ResolverError> {
        let name = name.into();
        parse_fqdn(&name).map_err(ResolverError::Validation)?;
        let owner = owner.into();
        validate_owner(&owner).map_err(ResolverError::Validation)?;

        let address = address.into();
        self.reverse_records.insert(address.clone(), name.clone());
        self.forward_records.insert(
            name,
            ResolutionRecord {
                owner,
                address,
                text_records: BTreeMap::new(),
                updated_at: now_unix,
            },
        );
        Ok(())
    }

    pub fn set_text_record(
        &mut self,
        name: &str,
        caller: &str,
        key: impl Into<String>,
        value: impl Into<String>,
        now_unix: u64,
    ) -> Result<(), ResolverError> {
        let key = key.into();
        let value = value.into();
        let record = self
            .forward_records
            .get_mut(name)
            .ok_or(ResolverError::RecordNotFound)?;
        if record.owner != caller {
            return Err(ResolverError::Unauthorized);
        }
        if !record.text_records.contains_key(&key) && record.text_records.len() >= MAX_TEXT_RECORDS {
            return Err(ResolverError::TooManyTextRecords);
        }

        record.text_records.insert(key, value);
        record.updated_at = now_unix;
        Ok(())
    }

    pub fn set_primary_name(
        &mut self,
        address: &str,
        caller: &str,
        name: &str,
    ) -> Result<(), ResolverError> {
        let record = self
            .forward_records
            .get(name)
            .ok_or(ResolverError::RecordNotFound)?;
        if record.owner != caller || record.address != address {
            return Err(ResolverError::Unauthorized);
        }

        self.primary_names
            .insert(address.to_string(), name.to_string());
        Ok(())
    }

    pub fn remove_record(&mut self, name: &str, caller: &str) -> Result<(), ResolverError> {
        let record = self
            .forward_records
            .get(name)
            .ok_or(ResolverError::RecordNotFound)?;
        if record.owner != caller {
            return Err(ResolverError::Unauthorized);
        }

        let address = record.address.clone();
        self.forward_records.remove(name);
        self.reverse_records.remove(&address);
        self.primary_names.retain(|_, value| value != name);
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<&ResolutionRecord> {
        self.forward_records.get(name)
    }

    pub fn reverse(&self, address: &str) -> Option<&str> {
        self.primary_names
            .get(address)
            .or_else(|| self.reverse_records.get(address))
            .map(String::as_str)
    }
}
