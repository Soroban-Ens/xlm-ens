use crate::NftContract;

impl NftContract {
    pub fn mint(&mut self, token_id: impl Into<String>, owner: impl Into<String>) -> bool {
        self.owners.insert(token_id.into(), owner.into()).is_none()
    }
}
