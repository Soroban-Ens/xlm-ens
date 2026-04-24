use crate::config::NetworkConfig;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::TransferRequest;

pub fn run_transfer(config: NetworkConfig, name: &str, new_owner: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
    );

    println!("Initiating transfer of {name} to {new_owner}...");

    match client.transfer(TransferRequest {
        name: name.into(),
        new_owner: new_owner.into(),
    }) {
        Ok(_) => {
            println!("SUCCESS: {name} ownership transferred to {new_owner}");

            // 3. Verify ownership change (mocked)
            match client.get_registration(name) {
                Ok(Some(reg)) => {
                    if let Some(addr) = reg.address {
                        println!("Verified: Current owner is now {addr}");
                    }
                }
                _ => {
                    println!("Warning: Could not verify ownership change immediately.");
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to transfer {name}: {e:?}");
        }
    }
}
