pub mod mint;
pub mod test;
pub mod transfer;

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct NftContract {
    owners: HashMap<String, String>,
}

impl NftContract {
    pub fn owner_of(&self, token_id: &str) -> Option<&str> {
        self.owners.get(token_id).map(String::as_str)
    }
}
