use crate::config::NetworkConfig;
use xlm_ns_sdk::client::XlmNsClient;

pub fn run_resolve(config: NetworkConfig, name: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
    );

    match client.resolve(name) {
        Ok(result) => {
            if let Some(addr) = result.address {
                println!("{} -> {}", result.name, addr);
            } else {
                println!("{} -> [NOT FOUND]", result.name);
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to resolve {name}: {e:?}");
        }
    }
}
