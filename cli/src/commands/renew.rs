use crate::config::NetworkConfig;
use crate::signer::SignerProfile;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::RenewalRequest;

pub fn run_renew(
    config: NetworkConfig,
    name: &str,
    years: u64,
    signer: Option<SignerProfile>,
) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
        Some(config.bridge_contract_id),
    );

    match client.get_registration(name) {
        Ok(Some(_)) => {
            if let Some(ref s) = signer {
                println!("  Signer: {}", s.describe());
            }
            match client.renew(RenewalRequest {
                name: name.into(),
                additional_years: years as u32,
                signer: signer.as_ref().map(|s| s.name.clone()),
            }) {
                Ok(receipt) => {
                    println!("SUCCESS: Renewed {name} for {years} year(s)");
                    println!("  Fee Paid: {} XLM", receipt.fee_paid);
                    println!("  New Expiry: {}", receipt.new_expiry);
                    println!("  Status: {}", receipt.submission.status);
                    println!("  Transaction Hash: {}", receipt.submission.tx_hash);
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to renew {name}: {e:?}");
                }
            }
        }
        Ok(None) => {
            eprintln!("ERROR: Name '{name}' is not registered and cannot be renewed.");
        }
        Err(e) => {
            eprintln!("ERROR: Failed to fetch registration state: {e:?}");
        }
    }
}
