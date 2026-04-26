#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{RegistryContract, RegistryContractClient, RegistryError};

    #[test]
    fn stores_registry_entries_in_persistent_storage() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let next_owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let target = Some(String::from_str(&env, "GABC"));

        client.register(
            &name,
            &owner,
            &target,
            &None::<String>,
            &100,
            &1_000,
            &2_000,
        );
        client.transfer(&name, &owner, &next_owner, &101);

        let resolved = client.resolve(&name, &101);
        assert_eq!(resolved.owner, next_owner);
        assert_eq!(client.names_for_owner(&next_owner).len(), 1);
    }

    #[test]
    fn rejects_registration_with_expiry_before_now() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        let result = client.try_register(
            &name,
            &owner,
            &None::<String>,
            &None::<String>,
            &100,
            &99,
            &100,
        );

        assert_eq!(result, Ok(Err(RegistryError::InvalidExpiry)));
    }

    #[test]
    fn rejects_registration_with_grace_period_before_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        let result = client.try_register(
            &name,
            &owner,
            &None::<String>,
            &None::<String>,
            &100,
            &200,
            &199,
        );

        assert_eq!(result, Ok(Err(RegistryError::InvalidGracePeriod)));
    }

    #[test]
    fn rejects_renewal_with_malformed_lifecycle_timestamps() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        client.register(
            &name,
            &owner,
            &None::<String>,
            &None::<String>,
            &100,
            &200,
            &300,
        );

        let invalid_expiry = client.try_renew(&name, &owner, &99, &300, &100);
        assert_eq!(invalid_expiry, Ok(Err(RegistryError::InvalidExpiry)));

        let invalid_grace_period = client.try_renew(&name, &owner, &250, &249, &100);
        assert_eq!(
            invalid_grace_period,
            Ok(Err(RegistryError::InvalidGracePeriod))
        );
    }

    #[test]
    fn rejects_renewal_that_reduces_expiry_or_grace_period() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        client.register(
            &name,
            &owner,
            &None::<String>,
            &None::<String>,
            &100,
            &200,
            &300,
        );

        // Try to reduce expiry but keep it valid otherwise (e.g. >= now)
        // now = 100, entry expires at 200. Let's try to renew with expires_at = 150
        let reduced_expiry = client.try_renew(&name, &owner, &150, &300, &100);
        assert_eq!(reduced_expiry, Ok(Err(RegistryError::InvalidExpiry)));

        // Try to reduce grace period but keep it valid otherwise
        let reduced_grace = client.try_renew(&name, &owner, &250, &280, &100);
        assert_eq!(reduced_grace, Ok(Err(RegistryError::InvalidGracePeriod)));

        // Valid extension
        client.renew(&name, &owner, &300, &400, &100);
        let entry = client.resolve(&name, &100);
        assert_eq!(entry.expires_at, 300);
        assert_eq!(entry.grace_period_ends_at, 400);
    }

    #[test]
    fn threat_unauthorized_actor_cannot_register_without_auth() {
        let env = Env::default();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.register(
                &name,
                &owner,
                &None::<String>,
                &None::<String>,
                &100,
                &1_000,
                &2_000,
            );
        }));

        assert!(result.is_err(), "registration without auth should fail");
        assert_eq!(
            client.try_resolve(&name, &100),
            Ok(Err(RegistryError::NotFound))
        );
    }

    #[test]
    fn threat_unauthorized_actor_cannot_transfer_without_auth() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let next_owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        client.register(
            &name,
            &owner,
            &None::<String>,
            &None::<String>,
            &100,
            &1_000,
            &2_000,
        );

        env.set_auths(&[]);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.transfer(&name, &owner, &next_owner, &101);
        }));

        assert!(result.is_err(), "transfer without auth should fail");
        let resolved = client.resolve(&name, &101);
        assert_eq!(resolved.owner, owner);
        assert_eq!(client.names_for_owner(&next_owner).len(), 0);
    }

    #[test]
    fn threat_actor_cannot_transfer_unowned_name() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let next_owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        client.register(&name, &owner, &None::<String>, &None::<String>, &100, &1_000, &2_000);

        let result = client.try_transfer(&name, &attacker, &next_owner, &101);
        assert_eq!(result, Ok(Err(RegistryError::Unauthorized)));
    }

    #[test]
    fn threat_actor_cannot_set_resolver_for_unowned_name() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        client.register(&name, &owner, &None::<String>, &None::<String>, &100, &1_000, &2_000);

        let resolver = Some(String::from_str(&env, "resolver_address"));
        let result = client.try_set_resolver(&name, &attacker, &resolver, &101);
        assert_eq!(result, Ok(Err(RegistryError::Unauthorized)));
    }

    #[test]
    fn threat_actor_cannot_set_target_address_for_unowned_name() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        client.register(&name, &owner, &None::<String>, &None::<String>, &100, &1_000, &2_000);

        let target = Some(String::from_str(&env, "target_address"));
        let result = client.try_set_target_address(&name, &attacker, &target, &101);
        assert_eq!(result, Ok(Err(RegistryError::Unauthorized)));
    }

    #[test]
    fn threat_actor_cannot_set_metadata_for_unowned_name() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        client.register(&name, &owner, &None::<String>, &None::<String>, &100, &1_000, &2_000);

        let metadata = Some(String::from_str(&env, "ipfs://hash"));
        let result = client.try_set_metadata(&name, &attacker, &metadata, &101);
        assert_eq!(result, Ok(Err(RegistryError::Unauthorized)));
    }

    #[test]
    fn threat_actor_cannot_renew_unowned_name() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");

        client.register(&name, &owner, &None::<String>, &None::<String>, &100, &1_000, &2_000);

        let result = client.try_renew(&name, &attacker, &1500, &2500, &101);
        assert_eq!(result, Ok(Err(RegistryError::Unauthorized)));
    }

    #[test]
    fn declares_that_admin_recovery_is_not_supported() {
        let env = Env::default();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        assert!(!client.supports_admin_recovery());
    }
}
