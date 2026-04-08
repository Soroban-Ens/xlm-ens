#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{NftContract, NftContractClient};

    #[test]
    fn stores_token_records_in_contract_storage() {
        let env = Env::default();
        let contract_id = env.register(NftContract, ());
        let client = NftContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let approved = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let token_id = String::from_str(&env, "timmy.xlm");

        client
            .mint(&token_id, &owner, &Some(String::from_str(&env, "ipfs://timmy")));
        client.approve(&token_id, &owner, &approved);
        client.transfer(&token_id, &approved, &new_owner);

        assert_eq!(client.owner_of(&token_id), Some(new_owner));
    }
}
