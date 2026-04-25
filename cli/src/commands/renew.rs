use crate::config::NetworkConfig;
use crate::output::{emit, emit_error, OutputFormat};
use crate::signer::SignerProfile;
use serde_json::json;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::RenewalRequest;

pub fn run_renew(
    config: NetworkConfig,
    output: OutputFormat,
    name: &str,
    years: u64,
    signer: Option<SignerProfile>,
) {
    let registrar_contract_id = config
        .registrar_contract_id
        .clone()
        .expect("renew command validated registrar contract id");
    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    )
    .with_registrar(registrar_contract_id.clone());
    let signer_name = signer.as_ref().map(|profile| profile.name.clone());
    let signer_description = signer.as_ref().map(|profile| profile.describe());

    match client.renew(RenewalRequest {
        name: name.into(),
        additional_years: years as u32,
        signer: signer_name.clone(),
    }) {
        Ok(receipt) => {
            let mut lines = vec![
                format!("SUCCESS: Renewed {name} for {years} year(s)"),
                format!("  Registrar: {registrar_contract_id}"),
            ];
            if let Some(description) = &signer_description {
                lines.push(format!("  Signer: {description}"));
            }
            lines.push(format!("  Fee Paid: {} XLM", receipt.fee_paid));
            lines.push(format!("  New Expiry: {}", receipt.new_expiry));
            lines.push(format!("  Status: {}", receipt.submission.status));
            lines.push(format!("  Transaction Hash: {}", receipt.submission.tx_hash));

            emit(
                output,
                &lines.join("\n"),
                json!({
                    "name": name,
                    "additional_years": years,
                    "registrar_contract_id": registrar_contract_id,
                    "signer": signer_name,
                    "fee_paid": receipt.fee_paid,
                    "new_expiry": receipt.new_expiry,
                    "submission": {
                        "status": receipt.submission.status.to_string(),
                        "tx_hash": receipt.submission.tx_hash,
                    }
                }),
            );
        }
        Err(err) => {
            let message = format!("ERROR: Failed to renew {name}: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "name": name,
                    "additional_years": years,
                    "registrar_contract_id": registrar_contract_id,
                    "signer": signer_name,
                }),
            );
        }
    }
}
