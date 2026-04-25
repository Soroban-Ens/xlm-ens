use soroban_sdk::{testutils::Address as _, Address, Env, String};
use xlm_ns_registrar::{RegistrarContract, RegistrarContractClient};
use xlm_ns_registry::{RegistryContract, RegistryContractClient};

struct TimeHelper {
    pub now: u64,
}

impl TimeHelper {
    pub fn new(start: u64) -> Self {
        Self { now: start }
    }
    pub fn advance(&mut self, seconds: u64) {
        self.now += seconds;
    }
    pub fn future(&self, seconds: u64) -> u64 {
        self.now + seconds
    }
}

#[test]
fn full_registration_flow_covers_quote_and_ownership_state() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(RegistryContract, ());
    let registrar_id = env.register(RegistrarContract, ());

    let registrar = RegistrarContractClient::new(&env, &registrar_id);
    let registry = RegistryContractClient::new(&env, &registry_id);

    registrar.initialize(&registry_id);

    let owner = Address::generate(&env);
    let label = String::from_str(&env, "integration");
    let name = String::from_str(&env, "integration.xlm");
    let time = TimeHelper::new(1_000_000);

    // 1. Quote
    let duration_years = 3;
    let quote = registrar.quote_registration(&label, &duration_years, &time.now);

    assert!(quote.fee_stroops > 0, "quote fee should be greater than zero");
    assert!(quote.expiry_unix > time.now, "expiry should be in the future");

    // 2. Register
    registrar.register(&label, &owner, &duration_years, &quote.fee_stroops, &time.now);

    // 3. Verify Registrar State
    let reg_record = registrar.registration(&name).expect("registrar record missing in registrar contract");
    assert_eq!(reg_record.owner, owner, "registrar ownership mismatch");
    assert_eq!(reg_record.expires_at, quote.expiry_unix, "registrar expiry mismatch");
    assert_eq!(reg_record.grace_period_ends_at, quote.grace_period_ends_at, "registrar grace period mismatch");

    // 4. Verify Registry State
    let registry_entry = registry.resolve(&name, &time.now);
    assert_eq!(registry_entry.owner, owner, "registry ownership mismatch");
    assert_eq!(registry_entry.expires_at, quote.expiry_unix, "registry expiry mismatch");
    assert_eq!(registry_entry.grace_period_ends_at, quote.grace_period_ends_at, "registry grace period mismatch");
}

#[test]
fn renewal_syncs_expiry_and_grace_with_registry() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(RegistryContract, ());
    let registrar_id = env.register(RegistrarContract, ());

    let registrar = RegistrarContractClient::new(&env, &registrar_id);
    let registry = RegistryContractClient::new(&env, &registry_id);

    registrar.initialize(&registry_id);

    let owner = Address::generate(&env);
    let label = String::from_str(&env, "alice");
    let name = String::from_str(&env, "alice.xlm");
    let mut time = TimeHelper::new(1_000_000);

    // Initial registration
    let quote = registrar.quote_registration(&label, &1, &time.now);
    registrar.register(&label, &owner, &1, &quote.fee_stroops, &time.now);

    let initial_reg_entry = registry.resolve(&name, &time.now);
    assert_eq!(initial_reg_entry.expires_at, quote.expiry_unix);

    // Renew
    time.advance(100_000);
    let renew_quote = registrar.quote_registration(&label, &1, &time.now);
    registrar.renew(&name, &owner, &1, &renew_quote.fee_stroops, &time.now);

    let reg_record = registrar.registration(&name).unwrap();
    let updated_reg_entry = registry.resolve(&name, &time.now);

    // Expiry cannot diverge between contracts after renewal.
    assert_eq!(reg_record.expires_at, updated_reg_entry.expires_at);
    assert_eq!(reg_record.grace_period_ends_at, updated_reg_entry.grace_period_ends_at);
}

#[test]
fn renewal_during_grace_period() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(RegistryContract, ());
    let registrar_id = env.register(RegistrarContract, ());

    let registrar = RegistrarContractClient::new(&env, &registrar_id);
    let registry = RegistryContractClient::new(&env, &registry_id);

    registrar.initialize(&registry_id);

    let owner = Address::generate(&env);
    let label = String::from_str(&env, "grace");
    let name = String::from_str(&env, "grace.xlm");
    let mut time = TimeHelper::new(1_000_000);

    let quote = registrar.quote_registration(&label, &1, &time.now);
    registrar.register(&label, &owner, &1, &quote.fee_stroops, &time.now);

    // Advance time into grace period (past expiry, before grace ends)
    time.advance(quote.expiry_unix - time.now + 100);

    let renew_quote = registrar.quote_registration(&label, &1, &time.now);
    registrar.renew(&name, &owner, &1, &renew_quote.fee_stroops, &time.now);

    let reg_record = registrar.registration(&name).unwrap();
    let updated_reg_entry = registry.resolve(&name, &time.now);

    assert_eq!(reg_record.expires_at, updated_reg_entry.expires_at);
    assert_eq!(reg_record.grace_period_ends_at, updated_reg_entry.grace_period_ends_at);
}

#[test]
fn unauthorized_renewal_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(RegistryContract, ());
    let registrar_id = env.register(RegistrarContract, ());

    let registrar = RegistrarContractClient::new(&env, &registrar_id);
    let registry = RegistryContractClient::new(&env, &registry_id);

    registrar.initialize(&registry_id);

    let owner = Address::generate(&env);
    let intruder = Address::generate(&env);
    let label = String::from_str(&env, "alice");
    let name = String::from_str(&env, "alice.xlm");
    let time = TimeHelper::new(1_000_000);

    let quote = registrar.quote_registration(&label, &1, &time.now);
    registrar.register(&label, &owner, &1, &quote.fee_stroops, &time.now);

    let renew_quote = registrar.quote_registration(&label, &1, &time.now);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        registrar.renew(&name, &intruder, &1, &renew_quote.fee_stroops, &time.now);
    }));

    assert!(result.is_err(), "intruder should not be able to renew");
}
