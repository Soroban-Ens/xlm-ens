#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{RegistryContract, RegistryContractClient};

    #[test]
    fn stores_registry_entries_in_persistent_storage() {
        let env = Env::default();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let next_owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let target = Some(String::from_str(&env, "GABC"));

        client
            .register(
                &name,
                &owner,
                &target,
                &None::<String>,
                &100,
                &1_000,
                &2_000,
            )
            .unwrap();
        client.transfer(&name, &owner, &next_owner, &101).unwrap();

        let resolved = client.resolve(&name, &101).unwrap();
        assert_eq!(resolved.owner, next_owner);
        assert_eq!(client.names_for_owner(&next_owner).len(), 1);
    }
}
