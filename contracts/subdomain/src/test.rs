#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{SubdomainContract, SubdomainContractClient};

    #[test]
    fn stores_subdomain_records_in_contract_storage() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let controller = Address::generate(&env);
        let sub_owner = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        client.register_parent(&parent, &owner);
        client.add_controller(&parent, &owner, &controller);

        let fqdn = client
            .create(
                &String::from_str(&env, "pay"),
                &parent,
                &controller,
                &sub_owner,
                &100,
            );

        assert_eq!(fqdn, String::from_str(&env, "pay.timmy.xlm"));
        assert!(client.exists(&fqdn));
        assert_eq!(client.record(&fqdn).unwrap().owner, sub_owner);
    }
}
