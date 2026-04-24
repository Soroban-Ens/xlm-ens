use crate::config::NetworkConfig;
use crate::signer::SignerProfile;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::TransferRequest;

pub fn run_transfer(
    config: NetworkConfig,
    name: &str,
    new_owner: &str,
    signer: Option<SignerProfile>,
) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
    );

    println!("Initiating transfer of {name} to {new_owner}...");
    if let Some(ref s) = signer {
        println!("  Signer: {}", s.describe());
    }

    match client.transfer(TransferRequest {
        name: name.into(),
        new_owner: new_owner.into(),
        signer: signer.as_ref().map(|s| s.name.clone()),
    }) {
        Ok(submission) => {
            println!("SUCCESS: {name} ownership transferred to {new_owner}");
            println!("  Status: {}", submission.status);
            println!("  Transaction Hash: {}", submission.tx_hash);

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
