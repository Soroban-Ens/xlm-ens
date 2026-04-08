#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::expiry::{expiry_from_now, within_grace_period};
    use crate::pricing::price_for_label_length;
    use crate::{can_renew, RegistrarContract, RegistrarContractClient};

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
        assert!(can_renew(expiry, expiry + 10));
    }

    #[test]
    fn stores_registrations_in_contract_storage() {
        let env = Env::default();
        let contract_id = env.register(RegistrarContract, ());
        let client = RegistrarContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let label = String::from_str(&env, "timmy");
        let name = String::from_str(&env, "timmy.xlm");

        let quote = client.quote_registration(&label, &1, &100).unwrap();
        client.register(&label, &owner, &1, &quote.fee_stroops, &100).unwrap();
        assert!(!client.is_available(&label, &101));

        client
            .renew(&name, &owner, &1, &quote.fee_stroops, &200)
            .unwrap();

        let record = client.registration(&name).unwrap();
        assert_eq!(record.owner, owner);
        assert!(client.treasury_balance() >= quote.fee_stroops * 2);
    }
}
