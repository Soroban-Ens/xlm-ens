use crate::config::NetworkConfig;
use xlm_ns_sdk::client::XlmNsClient;

pub fn run_reverse(config: NetworkConfig, address: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id.clone()),
    )
    .with_resolver(config.resolver_contract_id);

    match client.reverse_resolve(address) {
        Ok(result) => match result.primary_name {
            Some(name) => {
                println!("{} -> {}", result.address, name);
                if let Some(resolver) = result.resolver {
                    println!("  Resolver: {resolver}");
                }
            }
            None => {
                println!("{} -> [NO PRIMARY NAME]", result.address);
            }
        },
        Err(e) => {
            eprintln!("ERROR: Failed reverse lookup for {address}: {e:?}");
        }
    }
}
