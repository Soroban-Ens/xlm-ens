use crate::config::NetworkConfig;
use crate::output::{emit, emit_error, OutputFormat};
use serde_json::json;
use xlm_ns_sdk::client::XlmNsClient;

pub fn run_resolve(config: NetworkConfig, output: OutputFormat, name: &str) {
    let resolver_contract_id = config
        .resolver_contract_id
        .clone()
        .expect("resolve command validated resolver contract id");
    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    )
    .with_resolver(resolver_contract_id.clone());

    match client.resolve(name) {
        Ok(result) => {
            let human = match &result.address {
                Some(address) => format!(
                    "{} -> {}\n  Resolver: {}",
                    result.name, address, resolver_contract_id
                ),
                None => format!("{} -> [NOT FOUND]\n  Resolver: {}", result.name, resolver_contract_id),
            };
            emit(
                output,
                &human,
                json!({
                    "name": result.name,
                    "address": result.address,
                    "resolver_contract_id": resolver_contract_id,
                    "expires_at": result.expires_at,
                }),
            );
        }
        Err(err) => {
            let message = format!("ERROR: Failed to resolve {name}: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "name": name,
                    "resolver_contract_id": resolver_contract_id,
                }),
            );
        }
    }
}
