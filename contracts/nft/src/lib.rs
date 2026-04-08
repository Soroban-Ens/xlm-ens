mod test;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct TokenRecord {
    pub owner: Address,
    pub approved: Option<Address>,
    pub metadata_uri: Option<String>,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Token(String),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum NftError {
    AlreadyMinted = 1,
    NotFound = 2,
    Unauthorized = 3,
}

#[contract]
pub struct NftContract;

#[contractimpl]
impl NftContract {
    pub fn mint(
        env: Env,
        token_id: String,
        owner: Address,
        metadata_uri: Option<String>,
    ) -> Result<(), NftError> {
        let key = DataKey::Token(token_id);
        if env.storage().persistent().has(&key) {
            return Err(NftError::AlreadyMinted);
        }
        let record = TokenRecord {
            owner,
            approved: None,
            metadata_uri,
        };
        env.storage().persistent().set(&key, &record);
        Ok(())
    }

    pub fn approve(
        env: Env,
        token_id: String,
        caller: Address,
        approved: Address,
    ) -> Result<(), NftError> {
        let mut record = get_token(&env, &token_id)?;
        if record.owner != caller {
            return Err(NftError::Unauthorized);
        }
        record.approved = Some(approved);
        env.storage().persistent().set(&DataKey::Token(token_id), &record);
        Ok(())
    }

    pub fn transfer(
        env: Env,
        token_id: String,
        caller: Address,
        new_owner: Address,
    ) -> Result<(), NftError> {
        let mut record = get_token(&env, &token_id)?;
        if record.owner != caller && record.approved.as_ref() != Some(&caller) {
            return Err(NftError::Unauthorized);
        }
        record.owner = new_owner;
        record.approved = None;
        env.storage().persistent().set(&DataKey::Token(token_id), &record);
        Ok(())
    }

    pub fn owner_of(env: Env, token_id: String) -> Option<Address> {
        env.storage()
            .persistent()
            .get::<_, TokenRecord>(&DataKey::Token(token_id))
            .map(|record| record.owner)
    }

    pub fn token(env: Env, token_id: String) -> Option<TokenRecord> {
        env.storage().persistent().get(&DataKey::Token(token_id))
    }
}

fn get_token(env: &Env, token_id: &String) -> Result<TokenRecord, NftError> {
    env.storage()
        .persistent()
        .get(&DataKey::Token(token_id.clone()))
        .ok_or(NftError::NotFound)
}
