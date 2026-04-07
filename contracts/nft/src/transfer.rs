use crate::NftContract;

impl NftContract {
    pub fn transfer(&mut self, token_id: &str, new_owner: impl Into<String>) -> bool {
        match self.owners.get_mut(token_id) {
            Some(owner) => {
                *owner = new_owner.into();
                true
            }
            None => false,
        }
    }
}
