#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{NftContract, NftContractClient};

    #[test]
    fn stores_metadata_and_query_methods_after_mint() {
        let env = Env::default();
        let contract_id = env.register(NftContract, ());
        let client = NftContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let token_id = String::from_str(&env, "timmy.xlm");
        let metadata_uri = String::from_str(&env, "ipfs://timmy");

        client.mint(&token_id, &owner, &Some(metadata_uri.clone()));

        assert_eq!(client.owner_of(&token_id), Some(owner.clone()));
        assert_eq!(client.total_supply(), 1);
        assert_eq!(client.balance_of(&owner), 1);
        assert_eq!(client.token_by_index(&0), Some(token_id.clone()));
        assert_eq!(client.token_of_owner_by_index(&owner, &0), Some(token_id.clone()));
        assert_eq!(client.token_uri(&token_id), Some(metadata_uri.clone()));

        let token = client.token(&token_id).unwrap();
        assert_eq!(token.owner, owner);
        assert_eq!(token.approved, None);
        assert_eq!(token.metadata_uri, Some(metadata_uri));
    }

    #[test]
    fn rejects_duplicate_mint_for_existing_token_id() {
        let env = Env::default();
        let contract_id = env.register(NftContract, ());
        let client = NftContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let other_owner = Address::generate(&env);
        let token_id = String::from_str(&env, "timmy.xlm");

        client.mint(&token_id, &owner, &None::<String>);

        let duplicate_mint = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.mint(
                &token_id,
                &other_owner,
                &Some(String::from_str(&env, "ipfs://other")),
            );
        }));

        assert!(duplicate_mint.is_err(), "duplicate mint should fail");
        let token = client.token(&token_id).unwrap();
        assert_eq!(token.owner, owner);
        assert_eq!(token.metadata_uri, None);
    }

    #[test]
    fn stores_approval_and_allows_approved_transfer() {
        let env = Env::default();
        let contract_id = env.register(NftContract, ());
        let client = NftContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let approved = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let token_id = String::from_str(&env, "timmy.xlm");

        client.mint(
            &token_id,
            &owner,
            &Some(String::from_str(&env, "ipfs://timmy")),
        );
        client.approve(&token_id, &owner, &approved);

        let approved_token = client.token(&token_id).unwrap();
        assert_eq!(approved_token.owner, owner);
        assert_eq!(approved_token.approved, Some(approved.clone()));

        client.transfer(&token_id, &approved, &new_owner);

        assert_eq!(client.owner_of(&token_id), Some(new_owner.clone()));

        let transferred_token = client.token(&token_id).unwrap();
        assert_eq!(transferred_token.owner, new_owner);
        assert_eq!(transferred_token.approved, None);
        assert_eq!(
            transferred_token.metadata_uri,
            Some(String::from_str(&env, "ipfs://timmy"))
        );
    }

    #[test]
    fn rejects_transfer_from_unauthorized_caller() {
        let env = Env::default();
        let contract_id = env.register(NftContract, ());
        let client = NftContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let intruder = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let token_id = String::from_str(&env, "timmy.xlm");

        client.mint(&token_id, &owner, &None::<String>);

        let unauthorized_transfer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.transfer(&token_id, &intruder, &new_owner);
        }));

        assert!(
            unauthorized_transfer.is_err(),
            "unauthorized transfer should fail"
        );
        assert_eq!(client.owner_of(&token_id), Some(owner.clone()));

        let token = client.token(&token_id).unwrap();
        assert_eq!(token.owner, owner);
        assert_eq!(token.approved, None);
    }

    #[test]
    fn updates_query_methods_after_owner_transfer() {
        let env = Env::default();
        let contract_id = env.register(NftContract, ());
        let client = NftContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let approved = Address::generate(&env);
        let token_id = String::from_str(&env, "timmy.xlm");

        client.mint(
            &token_id,
            &owner,
            &Some(String::from_str(&env, "ipfs://timmy")),
        );
        client.approve(&token_id, &owner, &approved);
        client.transfer(&token_id, &owner, &new_owner);

        assert_eq!(client.owner_of(&token_id), Some(new_owner.clone()));
        assert_eq!(client.total_supply(), 1);
        assert_eq!(client.balance_of(&owner), 0);
        assert_eq!(client.balance_of(&new_owner), 1);
        assert_eq!(client.token_by_index(&0), Some(token_id.clone()));
        assert_eq!(client.token_of_owner_by_index(&owner, &0), None);
        assert_eq!(
            client.token_of_owner_by_index(&new_owner, &0),
            Some(token_id.clone())
        );
        assert_eq!(
            client.token_uri(&token_id),
            Some(String::from_str(&env, "ipfs://timmy"))
        );

        let token = client.token(&token_id).unwrap();
        assert_eq!(token.owner, new_owner);
        assert_eq!(token.approved, None);
        assert_eq!(
            token.metadata_uri,
            Some(String::from_str(&env, "ipfs://timmy"))
        );
    }

    #[test]
    fn enumerates_global_and_owner_tokens_across_multiple_mints() {
        let env = Env::default();
        let contract_id = env.register(NftContract, ());
        let client = NftContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let other_owner = Address::generate(&env);
        let first_token = String::from_str(&env, "alpha.xlm");
        let second_token = String::from_str(&env, "beta.xlm");
        let third_token = String::from_str(&env, "gamma.xlm");

        client.mint(&first_token, &owner, &Some(String::from_str(&env, "ipfs://alpha")));
        client.mint(&second_token, &owner, &None::<String>);
        client.mint(&third_token, &other_owner, &Some(String::from_str(&env, "ipfs://gamma")));

        assert_eq!(client.total_supply(), 3);
        assert_eq!(client.balance_of(&owner), 2);
        assert_eq!(client.balance_of(&other_owner), 1);

        assert_eq!(client.token_by_index(&0), Some(first_token.clone()));
        assert_eq!(client.token_by_index(&1), Some(second_token.clone()));
        assert_eq!(client.token_by_index(&2), Some(third_token.clone()));
        assert_eq!(client.token_by_index(&3), None);

        assert_eq!(client.token_of_owner_by_index(&owner, &0), Some(first_token));
        assert_eq!(client.token_of_owner_by_index(&owner, &1), Some(second_token));
        assert_eq!(client.token_of_owner_by_index(&owner, &2), None);
        assert_eq!(
            client.token_of_owner_by_index(&other_owner, &0),
            Some(third_token.clone())
        );
        assert_eq!(client.token_uri(&third_token), Some(String::from_str(&env, "ipfs://gamma")));
    }

    #[test]
    fn approval_changes_do_not_change_enumeration_queries() {
        let env = Env::default();
        let contract_id = env.register(NftContract, ());
        let client = NftContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let approved = Address::generate(&env);
        let token_id = String::from_str(&env, "timmy.xlm");

        client.mint(&token_id, &owner, &Some(String::from_str(&env, "ipfs://timmy")));
        client.approve(&token_id, &owner, &approved);

        assert_eq!(client.total_supply(), 1);
        assert_eq!(client.balance_of(&owner), 1);
        assert_eq!(client.token_by_index(&0), Some(token_id.clone()));
        assert_eq!(client.token_of_owner_by_index(&owner, &0), Some(token_id.clone()));
        assert_eq!(
            client.token_uri(&token_id),
            Some(String::from_str(&env, "ipfs://timmy"))
        );

        let token = client.token(&token_id).unwrap();
        assert_eq!(token.approved, Some(approved));
    }
}
