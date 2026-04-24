use soroban_sdk::{testutils::Address as _, Address, Env, String};
use xlm_ns_registrar::{RegistrarContract, RegistrarContractClient};
use xlm_ns_registry::{RegistryContract, RegistryContractClient};

#[test]
fn renewal_syncs_expiry_and_grace_with_registry() {
    let env = Env::default();

    let registry_id = env.register(RegistryContract, ());
    let registrar_id = env.register(RegistrarContract, ());

    let registrar = RegistrarContractClient::new(&env, &registrar_id);
    let registry = RegistryContractClient::new(&env, &registry_id);

    registrar.initialize(&registry_id);

    let owner = Address::generate(&env);
    let label = String::from_str(&env, "alice");
    let name = String::from_str(&env, "alice.xlm");
    let now: u64 = 1_000_000;

    // Initial registration
    let quote = registrar.quote_registration(&label, &1, &now);
    registrar.register(&label, &owner, &1, &quote.fee_stroops, &now);

    let initial_reg_entry = registry.resolve(&name, &now);
    assert_eq!(initial_reg_entry.expires_at, quote.expiry_unix);

    // Renew
    let renew_now = now + 100_000;
    let renew_quote = registrar.quote_registration(&label, &1, &renew_now);
    registrar.renew(&name, &owner, &1, &renew_quote.fee_stroops, &renew_now);

    let reg_record = registrar.registration(&name).unwrap();
    let updated_reg_entry = registry.resolve(&name, &renew_now);

    // Expiry cannot diverge between contracts after renewal.
    assert_eq!(reg_record.expires_at, updated_reg_entry.expires_at);
    assert_eq!(reg_record.grace_period_ends_at, updated_reg_entry.grace_period_ends_at);
}
