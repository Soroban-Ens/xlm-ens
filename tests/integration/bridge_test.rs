#[cfg(test)]
mod bridge_integration {
    use soroban_sdk::{Env, String};
    use xlm_ns_bridge::{BridgeContract, BridgeContractClient};

    fn setup_env() -> (Env, BridgeContractClient<'static>) {
        let env = Env::default();
        let contract_id = env.register(BridgeContract, ());
        let client = BridgeContractClient::new(&env, &contract_id);
        (env, client)
    }

    /// Test covers route registration and exact message generation shape.
    #[test]
    fn test_route_registration_and_message_generation() {
        let (env, client) = setup_env();
        
        let chain = String::from_str(&env, "ethereum");
        let name = String::from_str(&env, "alice.xlm");

        // Register route
        client.register_chain(&chain);

        // Generate payload
        let payload = client.build_message(&name, &chain);

        // Payload shape is asserted exactly, not just substring-checked.
        let expected_payload = String::from_str(
            &env,
            r#"{"type":"xlm-ns-resolution","name":"alice.xlm","destination_chain":"ethereum","resolver":"0xethResolver"}"#,
        );
        assert_eq!(payload, expected_payload);
    }

    /// Invalid chains are rejected.
    #[test]
    #[should_panic(expected = "Error(Contract, #2)")]
    fn test_invalid_chain_registration_rejected() {
        let (env, client) = setup_env();
        let chain = String::from_str(&env, "solana");
        client.register_chain(&chain);
    }

    /// Malformed route data (such as an invalid name without TLD) is rejected.
    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn test_malformed_route_data_rejected() {
        let (env, client) = setup_env();
        let chain = String::from_str(&env, "base");
        client.register_chain(&chain);

        let malformed_name = String::from_str(&env, "malformed-name");
        client.build_message(&malformed_name, &chain);
    }
}