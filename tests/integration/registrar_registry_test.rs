/// Integration tests: registrar registration materialises ownership state in the registry.
///
/// These tests verify the full registration path described in the README:
///   1. Obtain a quote from the registrar.
///   2. Submit payment and create a registration record (registrar).
///   3. Verify that the registry entry is automatically created (registry).
///   4. Renew through the registrar and verify registry expiry values match.
#[cfg(test)]
mod registrar_registry_integration {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};
    use xlm_ns_registrar::{RegistrarContract, RegistrarContractClient};
    use xlm_ns_registry::{RegistryContract, RegistryContractClient};

    fn setup_env() -> (
        Env,
        RegistrarContractClient<'static>,
        RegistryContractClient<'static>,
    ) {
        let env = Env::default();

        let registry_id = env.register(RegistryContract, ());
        let registrar_id = env.register(RegistrarContract, ());

        let registrar = RegistrarContractClient::new(&env, &registrar_id);
        let registry = RegistryContractClient::new(&env, &registry_id);

        // Wire the registrar to the registry.
        registrar.initialize(&registry_id);

        (env, registrar, registry)
    }

    /// A successful registration through the registrar must produce a matching
    /// ownership record in the registry.
    #[test]
    fn registration_materialises_registry_ownership() {
        let (env, registrar, registry) = setup_env();
        let owner = Address::generate(&env);
        let label = String::from_str(&env, "alice");
        let name = String::from_str(&env, "alice.xlm");
        let now: u64 = 1_000_000;

        let quote = registrar.quote_registration(&label, &1, &now);
        registrar.register(&label, &owner, &1, &quote.fee_stroops, &now);

        // Registrar should have a record.
        let reg_record = registrar.registration(&name).expect("registrar record missing");
        assert_eq!(reg_record.owner, owner);

        // Registry must also have the matching entry.
        let reg_entry = registry.resolve(&name, &now);
        assert_eq!(reg_entry.owner, owner);
    }

    /// Expiry and grace values must be identical between the registrar record
    /// and the registry entry after registration.
    #[test]
    fn expiry_and_grace_values_match_after_registration() {
        let (env, registrar, registry) = setup_env();
        let owner = Address::generate(&env);
        let label = String::from_str(&env, "bob");
        let name = String::from_str(&env, "bob.xlm");
        let now: u64 = 2_000_000;

        let quote = registrar.quote_registration(&label, &2, &now);
        registrar.register(&label, &owner, &2, &quote.fee_stroops, &now);

        let reg_record = registrar.registration(&name).unwrap();
        let reg_entry = registry.resolve(&name, &now);

        assert_eq!(
            reg_record.expires_at, reg_entry.expires_at,
            "expires_at mismatch between registrar and registry"
        );
        assert_eq!(
            reg_record.grace_period_ends_at, reg_entry.grace_period_ends_at,
            "grace_period_ends_at mismatch between registrar and registry"
        );
    }

    /// After a renewal the updated expiry and grace values must be reflected in
    /// both the registrar and the registry.
    #[test]
    fn renewal_updates_registry_expiry_and_grace() {
        let (env, registrar, registry) = setup_env();
        let owner = Address::generate(&env);
        let label = String::from_str(&env, "carol");
        let name = String::from_str(&env, "carol.xlm");
        let now: u64 = 3_000_000;

        // Initial registration.
        let quote = registrar.quote_registration(&label, &1, &now);
        registrar.register(&label, &owner, &1, &quote.fee_stroops, &now);

        // Renew shortly after.
        let renew_now: u64 = now + 1_000;
        registrar.renew(&name, &owner, &1, &quote.fee_stroops, &renew_now);

        let reg_record = registrar.registration(&name).unwrap();
        let reg_entry = registry.resolve(&name, &renew_now);

        assert!(
            reg_record.expires_at > quote.expiry_unix,
            "expires_at should be extended after renewal"
        );
        assert_eq!(
            reg_record.expires_at, reg_entry.expires_at,
            "expires_at must match between registrar and registry post-renewal"
        );
        assert_eq!(
            reg_record.grace_period_ends_at, reg_entry.grace_period_ends_at,
            "grace_period_ends_at must match between registrar and registry post-renewal"
        );
    }

    /// A name registered for multiple years should carry the correct ownership
    /// state in the registry across the full tenure.
    #[test]
    fn full_registration_path_multi_year() {
        let (env, registrar, registry) = setup_env();
        let owner = Address::generate(&env);
        let label = String::from_str(&env, "dave");
        let name = String::from_str(&env, "dave.xlm");
        let now: u64 = 5_000_000;

        let quote = registrar.quote_registration(&label, &3, &now);
        registrar.register(&label, &owner, &3, &quote.fee_stroops, &now);

        // Check just before expiry.
        let near_expiry = quote.expiry_unix - 1;
        let entry = registry.resolve(&name, &near_expiry);
        assert_eq!(entry.owner, owner);
        assert_eq!(entry.expires_at, quote.expiry_unix);
    }
}
