use crate::config::NetworkConfig;
use crate::output::{emit, emit_error, OutputFormat};
use serde_json::json;
use xlm_ns_sdk::client::XlmNsClient;

pub fn run_reverse(config: NetworkConfig, output: OutputFormat, address: &str) {
    let resolver_contract_id = config
        .resolver_contract_id
        .clone()
        .expect("reverse command validated resolver contract id");
    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    )
    .with_resolver(resolver_contract_id.clone());

    match client.reverse_resolve(address) {
        Ok(result) => {
            let human = match &result.primary_name {
                Some(name) => format!(
                    "{} -> {}\n  Resolver: {}",
                    result.address, name, resolver_contract_id
                ),
                None => format!(
                    "{} -> [NO PRIMARY NAME]\n  Resolver: {}",
                    result.address, resolver_contract_id
                ),
            };
            emit(
                output,
                &human,
                json!({
                    "address": result.address,
                    "primary_name": result.primary_name,
                    "resolver_contract_id": resolver_contract_id,
                }),
            );
        }
        Err(err) => {
            let message = format!("ERROR: Failed reverse lookup for {address}: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "address": address,
                    "resolver_contract_id": resolver_contract_id,
                }),
            );
        }
    }
}
