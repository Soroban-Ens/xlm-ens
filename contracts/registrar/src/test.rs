#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::expiry::{expiry_from_now, within_grace_period};
    use crate::pricing::price_for_label_length;
    use crate::{
        can_renew, RegistrarContract, RegistrarContractClient, RegistrarError, GRACE_PERIOD_SECONDS,
    };
    use xlm_ns_registry::RegistryContract;

    #[test]
    fn applies_tiered_pricing() {
        assert_eq!(price_for_label_length(3), 1_000_000_000);
        assert_eq!(price_for_label_length(5), 250_000_000);
        assert_eq!(price_for_label_length(12), 100_000_000);
    }

    #[test]
    fn computes_expiry_and_grace_period() {
        let expiry = expiry_from_now(100, 1);
        assert!(within_grace_period(expiry, expiry + 10));
        assert!(can_renew(expiry, expiry + 10).unwrap());
    }

    #[test]
    fn stores_registrations_in_contract_storage() {
        let env = Env::default();
        let contract_id = env.register(RegistrarContract, ());
        let client = RegistrarContractClient::new(&env, &contract_id);

        let registry_id = env.register(RegistryContract, ());
        client.initialize(&registry_id);

        let owner = Address::generate(&env);
        let label = String::from_str(&env, "timmy");
        let name = String::from_str(&env, "timmy.xlm");

        let quote = client.quote_registration(&label, &1, &100);
        client.register(&label, &owner, &1, &quote.fee_stroops, &100);
        assert!(!client.is_available(&label, &101));

        client.renew(&name, &owner, &1, &quote.fee_stroops, &200);

        let record = client.registration(&name).unwrap();
        assert_eq!(record.owner, owner);
        assert!(client.treasury_balance() >= quote.fee_stroops * 2);
    }

    // ==================== Renewal Lifecycle Tests ====================

    #[test]
    fn can_renew_active_registration_before_expiry() {
        let now = 1000;
        let expiry = 2000;
        let result = can_renew(expiry, now);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn can_renew_at_exact_expiry() {
        let now = 2000;
        let expiry = 2000;
        let result = can_renew(expiry, now);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn can_renew_during_grace_period() {
        let expiry = 1000;
        let _grace_end = expiry + GRACE_PERIOD_SECONDS;
        let now = expiry + 100;
        let result = can_renew(expiry, now);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn can_renew_at_grace_period_boundary_minus_one() {
        let expiry = 1000;
        let grace_end = expiry + GRACE_PERIOD_SECONDS;
        let now = grace_end - 1;
        let result = can_renew(expiry, now);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn can_renew_at_exact_grace_period_end() {
        let expiry = 1000;
        let grace_end = expiry + GRACE_PERIOD_SECONDS;
        let now = grace_end;
        let result = can_renew(expiry, now);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn cannot_renew_claimable_registration_after_grace_period() {
        let expiry = 1000;
        let grace_end = expiry + GRACE_PERIOD_SECONDS;
        let now = grace_end + 1;
        let result = can_renew(expiry, now);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), RegistrarError::RegistrationClaimable);
    }

    #[test]
    fn cannot_renew_claimable_registration_far_future() {
        let expiry = 1000;
        let grace_end = expiry + GRACE_PERIOD_SECONDS;
        let now = grace_end + 1000000;
        let result = can_renew(expiry, now);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), RegistrarError::RegistrationClaimable);
    }

    #[test]
    fn renew_fails_for_claimable_registration() {
        let env = Env::default();
        let contract_id = env.register(RegistrarContract, ());
        let client = RegistrarContractClient::new(&env, &contract_id);

        let registry_id = env.register(RegistryContract, ());
        client.initialize(&registry_id);

        let owner = Address::generate(&env);
        let label = String::from_str(&env, "test");
        let name = String::from_str(&env, "test.xlm");

        let quote = client.quote_registration(&label, &1, &100);
        client.register(&label, &owner, &1, &quote.fee_stroops, &100);

        let grace_end = quote.grace_period_ends_at;
        let after_grace = grace_end + 1;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.renew(&name, &owner, &1, &quote.fee_stroops, &after_grace);
        }));
        assert!(
            result.is_err(),
            "Renewal should fail for claimable registration"
        );
    }

    #[test]
    fn renew_succeeds_at_grace_period_boundary() {
        let env = Env::default();
        let contract_id = env.register(RegistrarContract, ());
        let client = RegistrarContractClient::new(&env, &contract_id);

        let registry_id = env.register(RegistryContract, ());
        client.initialize(&registry_id);

        let owner = Address::generate(&env);
        let label = String::from_str(&env, "boundary");
        let name = String::from_str(&env, "boundary.xlm");

        let quote = client.quote_registration(&label, &1, &100);
        client.register(&label, &owner, &1, &quote.fee_stroops, &100);

        let grace_end = quote.grace_period_ends_at;
        client.renew(&name, &owner, &1, &quote.fee_stroops, &grace_end);

        let record = client.registration(&name).unwrap();
        assert!(record.expires_at > quote.expiry_unix);
    }

    #[test]
    fn declares_that_admin_recovery_is_not_supported() {
        let env = Env::default();
        let contract_id = env.register(RegistrarContract, ());
        let client = RegistrarContractClient::new(&env, &contract_id);

        assert!(!client.supports_admin_recovery());
    }

    #[test]
    fn quote_includes_pricing_breakdown() {
        let env = Env::default();
        let contract_id = env.register(RegistrarContract, ());
        let client = RegistrarContractClient::new(&env, &contract_id);

        // 5-char label → 250_000_000 stroops/year
        let label = String::from_str(&env, "alice");
        let quote = client.quote_registration(&label, &2, &100);
        assert_eq!(quote.pricing.annual_fee_stroops, 250_000_000);
        assert_eq!(quote.pricing.duration_years, 2);
        assert_eq!(quote.pricing.premium_stroops, 0);
        assert_eq!(quote.fee_stroops, 500_000_000);

        // 3-char label → 1_000_000_000 stroops/year
        let short_label = String::from_str(&env, "foo");
        let short_quote = client.quote_registration(&short_label, &1, &100);
        assert_eq!(short_quote.pricing.annual_fee_stroops, 1_000_000_000);
        assert_eq!(short_quote.pricing.duration_years, 1);
        assert_eq!(short_quote.fee_stroops, 1_000_000_000);

        // 10-char label → 100_000_000 stroops/year
        let long_label = String::from_str(&env, "longerlabel");
        let long_quote = client.quote_registration(&long_label, &3, &100);
        assert_eq!(long_quote.pricing.annual_fee_stroops, 100_000_000);
        assert_eq!(long_quote.pricing.duration_years, 3);
        assert_eq!(long_quote.fee_stroops, 300_000_000);
    }

    #[test]
    fn fee_metrics_track_operations() {
        let env = Env::default();
        let contract_id = env.register(RegistrarContract, ());
        let client = RegistrarContractClient::new(&env, &contract_id);

        let registry_id = env.register(RegistryContract, ());
        client.initialize(&registry_id);

        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);
        let label1 = String::from_str(&env, "alpha");
        let label2 = String::from_str(&env, "delta");
        let name1 = String::from_str(&env, "alpha.xlm");

        let quote1 = client.quote_registration(&label1, &1, &100);
        let quote2 = client.quote_registration(&label2, &1, &100);

        client.register(&label1, &owner1, &1, &quote1.fee_stroops, &100);
        client.register(&label2, &owner2, &1, &quote2.fee_stroops, &100);
        client.renew(&name1, &owner1, &1, &quote1.fee_stroops, &200);

        let metrics = client.fee_metrics();
        assert_eq!(metrics.total_registrations, 2);
        assert_eq!(metrics.total_renewals, 1);
        assert_eq!(
            metrics.treasury_balance,
            quote1.fee_stroops + quote2.fee_stroops + quote1.fee_stroops
        );
        assert_eq!(metrics.treasury_balance, client.treasury_balance());
    }
}
