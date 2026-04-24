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

        let fqdn = client.create(
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

    #[test]
    fn tracks_subdomains_for_parent_and_owner() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let sub_owner1 = Address::generate(&env);
        let sub_owner2 = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        client.register_parent(&parent, &owner);

        let fqdn1 = client.create(
            &String::from_str(&env, "pay"),
            &parent,
            &owner,
            &sub_owner1,
            &100,
        );

        let fqdn2 = client.create(
            &String::from_str(&env, "app"),
            &parent,
            &owner,
            &sub_owner1,
            &100,
        );

        let parent_subs = client.subdomains_for_parent(&parent);
        assert_eq!(parent_subs.len(), 2);
        assert!(parent_subs.contains(&fqdn1));
        assert!(parent_subs.contains(&fqdn2));

        let owner1_subs = client.subdomains_for_owner(&sub_owner1);
        assert_eq!(owner1_subs.len(), 2);

        client.transfer(&fqdn2, &sub_owner1, &sub_owner2);

        let owner1_subs_after = client.subdomains_for_owner(&sub_owner1);
        assert_eq!(owner1_subs_after.len(), 1);
        assert_eq!(owner1_subs_after.get(0).unwrap(), fqdn1);

        let owner2_subs = client.subdomains_for_owner(&sub_owner2);
        assert_eq!(owner2_subs.len(), 1);
        assert_eq!(owner2_subs.get(0).unwrap(), fqdn2);

        client.delete(&fqdn1, &sub_owner1);

        let parent_subs_final = client.subdomains_for_parent(&parent);
        assert_eq!(parent_subs_final.len(), 1);
        assert_eq!(parent_subs_final.get(0).unwrap(), fqdn2);

        let owner1_subs_final = client.subdomains_for_owner(&sub_owner1);
        assert_eq!(owner1_subs_final.len(), 0);
    }
}
