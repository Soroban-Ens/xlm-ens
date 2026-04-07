pub mod manager;
pub mod test;

use std::collections::HashSet;

use manager::build_subdomain;

#[derive(Debug, Default)]
pub struct SubdomainContract {
    names: HashSet<String>,
}

impl SubdomainContract {
    pub fn create(&mut self, label: &str, parent: &str) -> bool {
        self.names.insert(build_subdomain(label, parent))
    }

    pub fn exists(&self, fqdn: &str) -> bool {
        self.names.contains(fqdn)
    }
}
