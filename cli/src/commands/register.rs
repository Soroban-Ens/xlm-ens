use crate::config::NetworkConfig;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::RegistrationRequest;

pub fn run_register(config: NetworkConfig, label: &str, owner: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
    );

    // 1. Fetch Quote
    let duration_years = 1; // Default to 1 year
    match client.quote_registration(label, duration_years) {
        Ok(quote) => {
            println!("Registration quote for {label}.xlm:");
            println!("  Fee: {} XLM", quote.fee);
            println!("  Duration: {duration_years} year(s)");
            println!("  Expiry: {}", quote.expires_at);

            // 2. Submit Request
            match client.register(RegistrationRequest {
                label: label.into(),
                owner: owner.into(),
                duration_years,
            }) {
                Ok(tx_hash) => {
                    println!("\nSUCCESS: registered {label}.xlm to {owner}");
                    println!("Transaction Hash: {tx_hash}");
                }
                Err(e) => {
                    eprintln!("\nERROR: Failed to submit registration: {e:?}");
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR: Failed to fetch registration quote: {e:?}");
        }
    }
}
