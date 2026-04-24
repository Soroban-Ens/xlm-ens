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
    fn removes_controller_and_revokes_authority() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let controller = Address::generate(&env);
        let sub_owner = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        client.register_parent(&parent, &owner);
        
        // Add controller
        client.add_controller(&parent, &owner, &controller);
        
        // Remove controller
        client.remove_controller(&parent, &owner, &controller);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.create(
                &String::from_str(&env, "pay"),
                &parent,
                &controller,
                &sub_owner,
                &100,
            );
        }));
        assert!(result.is_err(), "post-removal create should fail");
    }

    #[test]
    fn prevents_parent_takeover() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let intruder = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        client.register_parent(&parent, &owner);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.register_parent(&parent, &intruder);
        }));

        assert!(result.is_err(), "intruder parent registration should fail");
        
        let parent_record = client.parent(&parent).unwrap();
        assert_eq!(parent_record.owner, owner, "original owner should be preserved");
    }

    #[test]
    fn duplicate_parent_registration_fails_and_preserves_controllers() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let controller = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        // First write
        client.register_parent(&parent, &owner);
        client.add_controller(&parent, &owner, &controller);

        // Duplicate write attempt
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.register_parent(&parent, &owner);
        }));

        assert!(result.is_err(), "duplicate registration should fail");

        // Verify existing controllers are not lost due to accidental overwrite
        let parent_record = client.parent(&parent).unwrap();
        assert_eq!(parent_record.owner, owner);
        assert!(parent_record.controllers.contains(&controller), "existing controllers should not be lost");
    }

    #[test]
    fn subdomain_owner_can_revoke() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let parent_owner = Address::generate(&env);
        let sub_owner = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        client.register_parent(&parent, &parent_owner);
        let fqdn = client.create(
            &String::from_str(&env, "pay"),
            &parent,
            &parent_owner,
            &sub_owner,
            &100,
        );

        assert!(client.exists(&fqdn));
        client.revoke(&fqdn, &sub_owner);
        assert!(!client.exists(&fqdn));
    }

    #[test]
    fn parent_owner_can_revoke() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let parent_owner = Address::generate(&env);
        let sub_owner = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        client.register_parent(&parent, &parent_owner);
        let fqdn = client.create(
            &String::from_str(&env, "pay"),
            &parent,
            &parent_owner,
            &sub_owner,
            &100,
        );

        assert!(client.exists(&fqdn));
        client.revoke(&fqdn, &parent_owner);
        assert!(!client.exists(&fqdn));
    }

    #[test]
    fn parent_controller_can_revoke() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let parent_owner = Address::generate(&env);
        let controller = Address::generate(&env);
        let sub_owner = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        client.register_parent(&parent, &parent_owner);
        client.add_controller(&parent, &parent_owner, &controller);

        let fqdn = client.create(
            &String::from_str(&env, "pay"),
            &parent,
            &controller,
            &sub_owner,
            &100,
        );

        assert!(client.exists(&fqdn));
        client.revoke(&fqdn, &controller);
        assert!(!client.exists(&fqdn));
    }

    #[test]
    fn unauthorized_caller_cannot_revoke() {
        let env = Env::default();
        let contract_id = env.register(SubdomainContract, ());
        let client = SubdomainContractClient::new(&env, &contract_id);

        let parent_owner = Address::generate(&env);
        let sub_owner = Address::generate(&env);
        let intruder = Address::generate(&env);
        let parent = String::from_str(&env, "timmy.xlm");

        client.register_parent(&parent, &parent_owner);
        let fqdn = client.create(
            &String::from_str(&env, "pay"),
            &parent,
            &parent_owner,
            &sub_owner,
            &100,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.revoke(&fqdn, &intruder);
        }));
        assert!(result.is_err(), "unauthorized revocation should fail");
        assert!(client.exists(&fqdn));
    }
}
