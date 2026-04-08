#[cfg(test)]
mod tests {
    use soroban_sdk::{Env, String};

    use crate::{BridgeContract, BridgeContractClient};

    #[test]
    fn stores_bridge_routes_in_contract_storage() {
        let env = Env::default();
        let contract_id = env.register(BridgeContract, ());
        let client = BridgeContractClient::new(&env, &contract_id);

        let base = String::from_str(&env, "base");
        let name = String::from_str(&env, "timmy.xlm");

        client.register_chain(&base);
        let route = client.route(&base).unwrap();
        let payload = client.build_message(&name, &base);

        assert_eq!(route.destination_resolver, String::from_str(&env, "0xbaseResolver"));
        assert!(payload.to_string().contains("timmy.xlm"));
    }
}
