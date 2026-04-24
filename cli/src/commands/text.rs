use crate::config::NetworkConfig;
use crate::signer::SignerProfile;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::TextRecordUpdate;

fn build_client(config: &NetworkConfig) -> XlmNsClient {
    XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        Some(config.registry_contract_id.clone()),
    )
    .with_resolver(config.resolver_contract_id.clone())
}

pub fn run_get(config: NetworkConfig, name: &str, key: &str) {
    let client = build_client(&config);
    match client.get_text_record(name, key) {
        Ok(record) => match record.value {
            Some(value) => println!("{} {} = {}", record.name, record.key, value),
            None => println!("{} {} = [UNSET]", record.name, record.key),
        },
        Err(e) => {
            eprintln!("ERROR: Failed to read text record: {e:?}");
        }
    }
}

pub fn run_set(
    config: NetworkConfig,
    name: &str,
    key: &str,
    value: Option<String>,
    signer: Option<SignerProfile>,
) {
    let client = build_client(&config);
    if let Some(ref s) = signer {
        println!("  Signer: {}", s.describe());
    }

    let update = TextRecordUpdate {
        name: name.into(),
        key: key.into(),
        value,
        signer: signer.as_ref().map(|s| s.name.clone()),
    };

    match client.set_text_record(update) {
        Ok(submission) => {
            println!("SUCCESS: Updated text record '{key}' on {name}");
            println!("  Status: {}", submission.status);
            println!("  Transaction Hash: {}", submission.tx_hash);
        }
        Err(e) => {
            eprintln!("ERROR: Failed to set text record: {e:?}");
        }
    }
}
