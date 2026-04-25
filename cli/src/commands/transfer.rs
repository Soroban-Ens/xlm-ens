use crate::config::NetworkConfig;
use crate::output::{emit, emit_error, OutputFormat};
use crate::signer::SignerProfile;
use serde_json::json;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::TransferRequest;

pub fn run_transfer(
    config: NetworkConfig,
    output: OutputFormat,
    name: &str,
    new_owner: &str,
    signer: Option<SignerProfile>,
) {
    let registry_contract_id = config
        .registry_contract_id
        .clone()
        .expect("transfer command validated registry contract id");
    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        Some(registry_contract_id.clone()),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    );
    let signer_name = signer.as_ref().map(|profile| profile.name.clone());
    let signer_description = signer.as_ref().map(|profile| profile.describe());

    match client.transfer(TransferRequest {
        name: name.into(),
        new_owner: new_owner.into(),
        signer: signer_name.clone(),
    }) {
        Ok(submission) => {
            let verified_owner = match client.get_registration(name) {
                Ok(Some(registration)) => registration.address,
                _ => None,
            };
            let mut lines = vec![
                format!("SUCCESS: {name} ownership transferred to {new_owner}"),
                format!("  Registry: {registry_contract_id}"),
            ];
            if let Some(description) = &signer_description {
                lines.push(format!("  Signer: {description}"));
            }
            lines.push(format!("  Status: {}", submission.status));
            lines.push(format!("  Transaction Hash: {}", submission.tx_hash));
            if let Some(owner) = &verified_owner {
                lines.push(format!("  Verified owner: {owner}"));
            }

            emit(
                output,
                &lines.join("\n"),
                json!({
                    "name": name,
                    "new_owner": new_owner,
                    "registry_contract_id": registry_contract_id,
                    "signer": signer_name,
                    "submission": {
                        "status": submission.status.to_string(),
                        "tx_hash": submission.tx_hash,
                    },
                    "verified_owner": verified_owner,
                }),
            );
        }
        Err(err) => {
            let message = format!("ERROR: Failed to transfer {name}: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "name": name,
                    "new_owner": new_owner,
                    "registry_contract_id": registry_contract_id,
                    "signer": signer_name,
                }),
            );
        }
    }
}
