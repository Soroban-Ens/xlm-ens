use crate::config::NetworkConfig;
use crate::signer::SignerProfile;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::RegistrationRequest;

pub fn run_register(
    config: NetworkConfig,
    label: &str,
    owner: &str,
    signer: Option<SignerProfile>,
) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
        Some(config.bridge_contract_id),
    );

    let duration_years = 1;
    let quote = match client.quote_registration(label, duration_years) {
        Ok(quote) => quote,
        Err(e) => {
            eprintln!("ERROR: Failed to fetch registration quote: {e:?}");
            return;
        }
    };

    println!("Registration quote for {label}.xlm:");
    println!(
        "  Fee: {} {} (base {}, premium {}, network {})",
        quote.total_fee,
        quote.fee_currency,
        quote.fee_breakdown.base_fee,
        quote.fee_breakdown.premium_fee,
        quote.fee_breakdown.network_fee,
    );
    println!("  Duration: {duration_years} year(s)");
    println!("  Expiry: {}", quote.expires_at);
    if let Some(ref cid) = quote.contract_id {
        println!("  Registry: {cid}");
    }

    let signer_handle = signer.as_ref().map(|s| s.name.clone());
    if let Some(ref s) = signer {
        println!("  Signer: {}", s.describe());
    }

    match client.register(RegistrationRequest {
        label: label.into(),
        owner: owner.into(),
        duration_years,
        signer: signer_handle,
    }) {
        Ok(receipt) => {
            println!("\nSUCCESS: registered {} to {}", receipt.name, receipt.owner);
            println!("  Fee paid: {} {}", receipt.fee_paid, quote.fee_currency);
            println!("  Expires at: {}", receipt.expires_at);
            println!("  Status: {}", receipt.submission.status);
            println!("  Transaction Hash: {}", receipt.submission.tx_hash);
        }
        Err(e) => {
            eprintln!("\nERROR: Failed to submit registration: {e:?}");
        }
    }
}
