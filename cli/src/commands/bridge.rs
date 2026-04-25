use crate::config::NetworkConfig;
use crate::output::{emit, emit_error, OutputFormat};
use serde_json::json;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::{BuildMessageRequest, RegisterChainRequest};

fn build_client(config: &NetworkConfig) -> XlmNsClient {
    XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    )
}

pub fn run_register_chain(config: NetworkConfig, output: OutputFormat, chain: &str) {
    let bridge_contract_id = config
        .bridge_contract_id
        .clone()
        .expect("bridge command validated bridge contract id");
    let client = build_client(&config);

    match client.register_chain(RegisterChainRequest { chain: chain.into() }) {
        Ok(()) => emit(
            output,
            &format!(
                "SUCCESS: registered bridge route for chain {chain}\n  Bridge: {bridge_contract_id}"
            ),
            json!({
                "chain": chain,
                "bridge_contract_id": bridge_contract_id,
                "status": "registered",
            }),
        ),
        Err(err) => {
            let message = format!("ERROR: Failed to register chain: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "chain": chain,
                    "bridge_contract_id": bridge_contract_id,
                }),
            );
        }
    }
}

pub fn run_inspect_route(config: NetworkConfig, output: OutputFormat, chain: &str) {
    let bridge_contract_id = config
        .bridge_contract_id
        .clone()
        .expect("bridge command validated bridge contract id");
    let client = build_client(&config);

    match client.get_route(chain) {
        Ok(Some(route)) => emit(
            output,
            &format!(
                "Bridge route for chain '{chain}':\n  Bridge: {bridge_contract_id}\n  Chain: {}\n  Gateway: {}\n  Resolver: {}",
                route.destination_chain, route.gateway, route.destination_resolver
            ),
            json!({
                "chain": route.destination_chain,
                "gateway": route.gateway,
                "resolver": route.destination_resolver,
                "bridge_contract_id": bridge_contract_id,
            }),
        ),
        Ok(None) => {
            let message = format!("ERROR: No route found for chain '{chain}'");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "chain": chain,
                    "bridge_contract_id": bridge_contract_id,
                }),
            );
        }
        Err(err) => {
            let message = format!("ERROR: Failed to inspect route: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "chain": chain,
                    "bridge_contract_id": bridge_contract_id,
                }),
            );
        }
    }
}

pub fn run_generate_payload(
    config: NetworkConfig,
    output: OutputFormat,
    name: &str,
    chain: &str,
) {
    let bridge_contract_id = config
        .bridge_contract_id
        .clone()
        .expect("bridge command validated bridge contract id");
    let client = build_client(&config);

    match client.build_message(BuildMessageRequest {
        name: name.into(),
        chain: chain.into(),
    }) {
        Ok(payload) => emit(
            output,
            &format!(
                "Generated payload for '{name}' on chain '{chain}':\n  Bridge: {bridge_contract_id}\n{payload}"
            ),
            json!({
                "name": name,
                "chain": chain,
                "bridge_contract_id": bridge_contract_id,
                "payload": payload,
            }),
        ),
        Err(err) => {
            let message = format!("ERROR: Failed to generate payload: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "name": name,
                    "chain": chain,
                    "bridge_contract_id": bridge_contract_id,
                }),
            );
        }
    }
}
