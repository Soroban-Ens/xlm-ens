#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};
    use xlm_ns_common::MAX_TEXT_RECORDS;

    use crate::{ResolverContract, ResolverContractClient};

    #[test]
    fn persists_forward_reverse_and_primary_resolution_records() {
        let env = Env::default();
        let contract_id = env.register(ResolverContract, ());
        let client = ResolverContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let address = String::from_str(&env, "GABC");

        client.set_record(&name, &owner, &address, &100);
        client.set_text_record(
            &name,
            &owner,
            &String::from_str(&env, "com.twitter"),
            &String::from_str(&env, "@timmy"),
            &101,
        );
        client.set_primary_name(&address, &owner, &name);

        let record = client.resolve(&name).unwrap();
        assert_eq!(record.owner, owner);
        assert_eq!(record.address, address);
        assert_eq!(
            record
                .text_records
                .get(String::from_str(&env, "com.twitter")),
            Some(String::from_str(&env, "@timmy"))
        );
        assert_eq!(record.updated_at, 101);
        assert_eq!(client.reverse(&String::from_str(&env, "GABC")), Some(name));
    }

    #[test]
    fn removes_forward_reverse_and_primary_records() {
        let env = Env::default();
        let contract_id = env.register(ResolverContract, ());
        let client = ResolverContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let address = String::from_str(&env, "GABC");

        client.set_record(&name, &owner, &address, &100);
        client.set_primary_name(&address, &owner, &name);
        client.remove_record(&name, &owner);

        assert_eq!(client.resolve(&name), None);
        assert_eq!(client.reverse(&address), None);
    }

    #[test]
    fn rejects_text_record_updates_from_non_owner() {
        let env = Env::default();
        let contract_id = env.register(ResolverContract, ());
        let client = ResolverContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let intruder = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let address = String::from_str(&env, "GABC");

        client.set_record(&name, &owner, &address, &100);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.set_text_record(
                &name,
                &intruder,
                &String::from_str(&env, "com.twitter"),
                &String::from_str(&env, "@timmy"),
                &101,
            );
        }));

        assert!(result.is_err(), "non-owner text update should fail");
        let stored = client.resolve(&name).unwrap();
        assert_eq!(stored.text_records.len(), 0);
    }

    #[test]
    fn rejects_record_removal_from_non_owner() {
        let env = Env::default();
        let contract_id = env.register(ResolverContract, ());
        let client = ResolverContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let intruder = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let address = String::from_str(&env, "GABC");

        client.set_record(&name, &owner, &address, &100);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.remove_record(&name, &intruder);
        }));

        assert!(result.is_err(), "non-owner record removal should fail");
        assert!(client.resolve(&name).is_some());
        assert_eq!(client.reverse(&address), Some(name));
    }

    #[test]
    fn enforces_text_record_limit_but_allows_updating_existing_key_at_limit() {
        let env = Env::default();
        let contract_id = env.register(ResolverContract, ());
        let client = ResolverContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let address = String::from_str(&env, "GABC");

        client.set_record(&name, &owner, &address, &100);

        for idx in 0..MAX_TEXT_RECORDS {
            client.set_text_record(
                &name,
                &owner,
                &String::from_str(&env, &format!("key-{idx}")),
                &String::from_str(&env, &format!("value-{idx}")),
                &(101 + idx as u64),
            );
        }

        client.set_text_record(
            &name,
            &owner,
            &String::from_str(&env, "key-0"),
            &String::from_str(&env, "updated"),
            &500,
        );

        let updated_record = client.resolve(&name).unwrap();
        assert_eq!(updated_record.text_records.len(), MAX_TEXT_RECORDS as u32);
        assert_eq!(
            updated_record
                .text_records
                .get(String::from_str(&env, "key-0")),
            Some(String::from_str(&env, "updated"))
        );
        assert_eq!(updated_record.updated_at, 500);

        let overflow = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.set_text_record(
                &name,
                &owner,
                &String::from_str(&env, "overflow"),
                &String::from_str(&env, "value"),
                &501,
            );
        }));

        assert!(
            overflow.is_err(),
            "adding a new key past the limit should fail"
        );
    }

    #[test]
    fn reverse_lookup_prefers_primary_name_when_present() {
        let env = Env::default();
        let contract_id = env.register(ResolverContract, ());
        let client = ResolverContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let first_name = String::from_str(&env, "timmy.xlm");
        let second_name = String::from_str(&env, "pay.timmy.xlm");
        let address = String::from_str(&env, "GABC");

        client.set_record(&first_name, &owner, &address, &100);
        client.set_record(&second_name, &owner, &address, &101);

        assert_eq!(client.reverse(&address), Some(second_name.clone()));

        client.set_primary_name(&address, &owner, &first_name);
        assert_eq!(client.reverse(&address), Some(first_name));
    }

    #[test]
    fn replacing_address_updates_forward_record_but_keeps_previous_reverse_lookup() {
        let env = Env::default();
        let contract_id = env.register(ResolverContract, ());
        let client = ResolverContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let old_address = String::from_str(&env, "GABC");
        let new_address = String::from_str(&env, "GDEF");

        client.set_record(&name, &owner, &old_address, &100);
        client.set_record(&name, &owner, &new_address, &101);

        let record = client.resolve(&name).unwrap();
        assert_eq!(record.address, new_address);
        assert_eq!(record.updated_at, 101);
        assert_eq!(
            client.reverse(&String::from_str(&env, "GDEF")),
            Some(name.clone())
        );
        assert_eq!(client.reverse(&String::from_str(&env, "GABC")), Some(name));
    }
}
