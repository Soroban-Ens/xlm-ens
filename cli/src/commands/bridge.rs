use crate::config::NetworkConfig;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::{BuildMessageRequest, RegisterChainRequest};

pub fn run_register_chain(config: NetworkConfig, chain: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
        Some(config.bridge_contract_id),
    );

    match client.register_chain(RegisterChainRequest {
        chain: chain.into(),
    }) {
        Ok(()) => {
            println!("SUCCESS: registered bridge route for chain {}", chain);
        }
        Err(e) => {
            eprintln!("ERROR: Failed to register chain: {e:?}");
        }
    }
}

pub fn run_inspect_route(config: NetworkConfig, chain: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
        Some(config.bridge_contract_id),
    );

    match client.get_route(chain) {
        Ok(Some(route)) => {
            println!("Bridge route for chain '{}':", chain);
            println!("  Chain: {}", route.destination_chain);
            println!("  Gateway: {}", route.gateway);
            println!("  Resolver: {}", route.destination_resolver);
        }
        Ok(None) => {
            eprintln!("ERROR: No route found for chain '{}'", chain);
        }
        Err(e) => {
            eprintln!("ERROR: Failed to inspect route: {e:?}");
        }
    }
}

pub fn run_generate_payload(config: NetworkConfig, name: &str, chain: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
        Some(config.bridge_contract_id),
    );

    match client.build_message(BuildMessageRequest {
        name: name.into(),
        chain: chain.into(),
    }) {
        Ok(payload) => {
            println!("Generated payload for '{}' on chain '{}':", name, chain);
            println!("{}", payload);
        }
        Err(e) => {
            eprintln!("ERROR: Failed to generate payload: {e:?}");
        }
    }
}