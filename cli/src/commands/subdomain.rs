use crate::config::NetworkConfig;
use crate::output::{emit, emit_error, OutputFormat};
use serde_json::json;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::{
    AddControllerRequest, CreateSubdomainRequest, RegisterParentRequest, TransferSubdomainRequest,
};

pub fn run_register_parent(config: NetworkConfig, output: OutputFormat, parent: &str, owner: &str) {
    let subdomain_contract_id = config
        .subdomain_contract_id
        .clone()
        .expect("subdomain command validated subdomain contract id");

    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    );

    match client.register_parent(RegisterParentRequest {
        parent: parent.into(),
        owner: owner.into(),
    }) {
        Ok(()) => emit(
            output,
            &format!("Registered parent {parent}\n  Owner: {owner}\n  Subdomain: {subdomain_contract_id}"),
            json!({
                "parent": parent,
                "owner": owner,
                "subdomain_contract_id": subdomain_contract_id,
                "rpc_url": config.rpc_url,
                "network": config.network.as_str(),
            }),
        ),
        Err(err) => {
            let message = format!("ERROR: Failed to register parent {parent}: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "parent": parent,
                    "owner": owner,
                    "subdomain_contract_id": subdomain_contract_id,
                }),
            );
        }
    }
}

pub fn run_add_controller(
    config: NetworkConfig,
    output: OutputFormat,
    parent: &str,
    controller: &str,
) {
    let subdomain_contract_id = config
        .subdomain_contract_id
        .clone()
        .expect("subdomain command validated subdomain contract id");

    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    );

    match client.add_controller(AddControllerRequest {
        parent: parent.into(),
        controller: controller.into(),
    }) {
        Ok(()) => emit(
            output,
            &format!(
                "Added controller\n  Parent: {parent}\n  Controller: {controller}\n  Subdomain: {subdomain_contract_id}"
            ),
            json!({
                "parent": parent,
                "controller": controller,
                "subdomain_contract_id": subdomain_contract_id,
                "rpc_url": config.rpc_url,
                "network": config.network.as_str(),
            }),
        ),
        Err(err) => {
            let message = format!("ERROR: Failed to add controller: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "parent": parent,
                    "controller": controller,
                    "subdomain_contract_id": subdomain_contract_id,
                }),
            );
        }
    }
}

pub fn run_create_subdomain(
    config: NetworkConfig,
    output: OutputFormat,
    label: &str,
    parent: &str,
    owner: &str,
) {
    let subdomain_contract_id = config
        .subdomain_contract_id
        .clone()
        .expect("subdomain command validated subdomain contract id");

    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    );

    match client.create_subdomain(CreateSubdomainRequest {
        label: label.into(),
        parent: parent.into(),
        owner: owner.into(),
    }) {
        Ok(fqdn) => emit(
            output,
            &format!(
                "Created subdomain {fqdn}\n  Owner: {owner}\n  Subdomain: {subdomain_contract_id}"
            ),
            json!({
                "label": label,
                "parent": parent,
                "fqdn": fqdn,
                "owner": owner,
                "subdomain_contract_id": subdomain_contract_id,
                "rpc_url": config.rpc_url,
                "network": config.network.as_str(),
            }),
        ),
        Err(err) => {
            let message = format!("ERROR: Failed to create subdomain: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "label": label,
                    "parent": parent,
                    "owner": owner,
                    "subdomain_contract_id": subdomain_contract_id,
                }),
            );
        }
    }
}

pub fn run_transfer_subdomain(
    config: NetworkConfig,
    output: OutputFormat,
    fqdn: &str,
    new_owner: &str,
) {
    let subdomain_contract_id = config
        .subdomain_contract_id
        .clone()
        .expect("subdomain command validated subdomain contract id");

    let client = XlmNsClient::new(
        config.rpc_url.clone(),
        Some(config.network_passphrase.clone()),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
    );

    match client.transfer_subdomain(TransferSubdomainRequest {
        fqdn: fqdn.into(),
        new_owner: new_owner.into(),
    }) {
        Ok(()) => emit(
            output,
            &format!(
                "Transferred subdomain {fqdn}\n  New owner: {new_owner}\n  Subdomain: {subdomain_contract_id}"
            ),
            json!({
                "fqdn": fqdn,
                "new_owner": new_owner,
                "subdomain_contract_id": subdomain_contract_id,
                "rpc_url": config.rpc_url,
                "network": config.network.as_str(),
            }),
        ),
        Err(err) => {
            let message = format!("ERROR: Failed to transfer subdomain: {err}");
            emit_error(
                output,
                &message,
                json!({
                    "error": message,
                    "fqdn": fqdn,
                    "new_owner": new_owner,
                    "subdomain_contract_id": subdomain_contract_id,
                }),
            );
        }
    }
}
