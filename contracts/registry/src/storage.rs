use std::collections::{BTreeSet, HashMap};

use crate::errors::RegistryError;
use crate::types::RegistryEntry;

#[derive(Debug, Default)]
pub struct RegistryStorage {
    entries: HashMap<String, RegistryEntry>,
    owner_index: HashMap<String, BTreeSet<String>>,
}

impl RegistryStorage {
    pub fn insert(&mut self, entry: RegistryEntry) -> Result<(), RegistryError> {
        let key = entry.record.fqdn();
        if self.entries.contains_key(&key) {
            return Err(RegistryError::AlreadyRegistered);
        }

        self.owner_index
            .entry(entry.record.owner.clone())
            .or_default()
            .insert(key.clone());
        self.entries.insert(key, entry);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&RegistryEntry> {
        self.entries.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut RegistryEntry, RegistryError> {
        self.entries.get_mut(name).ok_or(RegistryError::NotFound)
    }

    pub fn replace(&mut self, name: &str, entry: RegistryEntry) {
        self.entries.insert(name.to_string(), entry);
    }

    pub fn remove(&mut self, name: &str) -> Option<RegistryEntry> {
        let removed = self.entries.remove(name)?;
        self.remove_owner_index(&removed.record.owner, name);
        Some(removed)
    }

    pub fn names_for_owner(&self, owner: &str) -> Vec<&RegistryEntry> {
        self.owner_index
            .get(owner)
            .into_iter()
            .flat_map(|names| names.iter())
            .filter_map(|name| self.entries.get(name))
            .collect()
    }

    pub fn reindex_owner(&mut self, name: &str, old_owner: &str, new_owner: &str) {
        self.remove_owner_index(old_owner, name);
        self.owner_index
            .entry(new_owner.to_string())
            .or_default()
            .insert(name.to_string());
    }

    fn remove_owner_index(&mut self, owner: &str, name: &str) {
        let should_remove = match self.owner_index.get_mut(owner) {
            Some(names) => {
                names.remove(name);
                names.is_empty()
            }
            None => false,
        };

        if should_remove {
            self.owner_index.remove(owner);
        }
    }
}
