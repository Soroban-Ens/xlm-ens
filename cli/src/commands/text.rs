use crate::config::NetworkConfig;
use crate::output::{emit, emit_error, OutputFormat};
use crate::signer::SignerProfile;
use serde_json::json;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::TextRecordUpdate;

fn build_client(config: &NetworkConfig) -> XlmNsClient {
    XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    )
    .with_resolver(
        config
            .resolver_contract_id
            .clone()
            .expect("text command validated resolver contract id"),
    )
}

pub fn run_get(config: NetworkConfig, output: OutputFormat, name: &str, key: &str) {
    let resolver_contract_id = config
        .resolver_contract_id
        .clone()
        .expect("text command validated resolver contract id");
    let client = build_client(&config);

    match client.get_text_record(name, key) {
        Ok(record) => {
            let human = match &record.value {
                Some(value) => format!(
                    "{} {} = {}\n  Resolver: {}",
                    record.name, record.key, value, resolver_contract_id
                ),
                None => format!(
                    "{} {} = [UNSET]\n  Resolver: {}",
                    record.name, record.key, resolver_contract_id
                ),
            };
            emit(
                output,
                &human,
                json!({
                    "name": record.name,
                    "key": record.key,
                    "value": record.value,
                    "resolver_contract_id": resolver_contract_id,
                }),
            );
        }
        Err(err) => {
            let message = format!("ERROR: Failed to read text record: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "name": name,
                    "key": key,
                    "resolver_contract_id": resolver_contract_id,
                }),
            );
        }
    }
}

pub fn run_set(
    config: NetworkConfig,
    output: OutputFormat,
    name: &str,
    key: &str,
    value: Option<String>,
    signer: Option<SignerProfile>,
) {
    let resolver_contract_id = config
        .resolver_contract_id
        .clone()
        .expect("text command validated resolver contract id");
    let client = build_client(&config);
    let signer_name = signer.as_ref().map(|profile| profile.name.clone());
    let signer_description = signer.as_ref().map(|profile| profile.describe());
    let update = TextRecordUpdate {
        name: name.into(),
        key: key.into(),
        value: value.clone(),
        signer: signer_name.clone(),
    };

    match client.set_text_record(update) {
        Ok(submission) => {
            let mut lines = vec![format!("SUCCESS: Updated text record '{key}' on {name}")];
            lines.push(format!("  Resolver: {resolver_contract_id}"));
            if let Some(description) = &signer_description {
                lines.push(format!("  Signer: {description}"));
            }
            lines.push(format!("  Status: {}", submission.status));
            lines.push(format!("  Transaction Hash: {}", submission.tx_hash));

            emit(
                output,
                &lines.join("\n"),
                json!({
                    "name": name,
                    "key": key,
                    "value": value,
                    "resolver_contract_id": resolver_contract_id,
                    "signer": signer_name,
                    "submission": {
                        "status": submission.status.to_string(),
                        "tx_hash": submission.tx_hash,
                    }
                }),
            );
        }
        Err(err) => {
            let message = format!("ERROR: Failed to set text record: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "name": name,
                    "key": key,
                    "resolver_contract_id": resolver_contract_id,
                    "signer": signer_name,
                }),
            );
        }
    }
}
