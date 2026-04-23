use crate::config::NetworkConfig;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::RenewalRequest;

pub fn run_renew(config: NetworkConfig, name: &str, years: u64) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
    );

    // 1. Fetch current registration to validate (mocked in SDK)
    match client.get_registration(name) {
        Ok(Some(_)) => {
            // 2. Perform renewal
            match client.renew(RenewalRequest {
                name: name.into(),
                additional_years: years as u32,
            }) {
                Ok(result) => {
                    println!("SUCCESS: Renewed {name} for {years} year(s)");
                    println!("  Fee Paid: {} XLM", result.fee_paid);
                    println!("  New Expiry: {}", result.new_expiry);
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
