use crate::config::NetworkConfig;
use crate::output::{emit, emit_error, OutputFormat};
use crate::signer::SignerProfile;
use serde_json::json;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::RegistrationRequest;

pub fn run_register(
    config: NetworkConfig,
    output: OutputFormat,
    label: &str,
    owner: &str,
    signer: Option<SignerProfile>,
) {
    let registrar_contract_id = config
        .registrar_contract_id
        .clone()
        .expect("register command validated registrar contract id");
    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    )
    .with_registrar(registrar_contract_id.clone());

    let duration_years = 1;
    let quote = match client.quote_registration(label, duration_years) {
        Ok(quote) => quote,
        Err(err) => {
            let message = format!("ERROR: Failed to fetch registration quote: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "label": label,
                    "owner": owner,
                }),
            );
            return;
        }
    };

    let signer_name = signer.as_ref().map(|profile| profile.name.clone());
    let signer_description = signer.as_ref().map(|profile| profile.describe());

    match client.register(RegistrationRequest {
        label: label.into(),
        owner: owner.into(),
        duration_years,
        signer: signer_name.clone(),
    }) {
        Ok(receipt) => {
            let mut lines = vec![
                format!("Registration quote for {label}.xlm:"),
                format!("  Registrar: {registrar_contract_id}"),
                format!(
                    "  Fee: {} {} (base {}, premium {}, network {})",
                    quote.total_fee,
                    quote.fee_currency,
                    quote.fee_breakdown.base_fee,
                    quote.fee_breakdown.premium_fee,
                    quote.fee_breakdown.network_fee,
                ),
                format!("  Duration: {duration_years} year(s)"),
                // Lifecycle timestamps — surfaced per issue #177
                format!("  Quoted at: {}", quote.quoted_at),
                format!("  Expiry: {}", quote.expires_at),
            ];
            if let Some(contract_id) = &quote.contract_id {
                lines.push(format!("  Quote contract: {contract_id}"));
            }
            if let Some(description) = &signer_description {
                lines.push(format!("  Signer: {description}"));
            }
            lines.push(String::new());
            lines.push(format!(
                "SUCCESS: registered {} to {}",
                receipt.name, receipt.owner
            ));
            lines.push(format!(
                "  Fee paid: {} {}",
                receipt.fee_paid, quote.fee_currency
            ));
            lines.push(format!("  Expires at: {}", receipt.expires_at));
            lines.push(format!("  Status: {}", receipt.submission.status));
            lines.push(format!(
                "  Transaction Hash: {}",
                receipt.submission.tx_hash
            ));

            emit(
                output,
                &lines.join("\n"),
                json!({
                    "label": label,
                    "name": receipt.name,
                    "owner": receipt.owner,
                    "duration_years": duration_years,
                    "quote": {
                        "currency": quote.fee_currency,
                        "total": quote.total_fee,
                        "breakdown": {
                            "base": quote.fee_breakdown.base_fee,
                            "premium": quote.fee_breakdown.premium_fee,
                            "network": quote.fee_breakdown.network_fee,
                        },
                        "quoted_at": quote.quoted_at,
                        "expires_at": quote.expires_at,
                        "contract_id": quote.contract_id,
                    },
                    "expires_at": receipt.expires_at,
                    "registrar_contract_id": registrar_contract_id,
                    "rpc_url": config.rpc_url,
                    "network": config.network.as_str(),
                    "signer": signer_name,
                    "submission": {
                        "status": receipt.submission.status.to_string(),
                        "tx_hash": receipt.submission.tx_hash,
                    }
                }),
            );
        }
        Err(err) => {
            let message = format!("ERROR: Failed to submit registration: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "label": label,
                    "owner": owner,
                    "registrar_contract_id": registrar_contract_id,
                }),
            );
        }
    }
}
