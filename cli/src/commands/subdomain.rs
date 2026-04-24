use crate::config::NetworkConfig;
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::{AddControllerRequest, CreateSubdomainRequest, RegisterParentRequest, TransferSubdomainRequest};

pub fn run_register_parent(config: NetworkConfig, parent: &str, owner: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
    );

    match client.register_parent(RegisterParentRequest {
        parent: parent.into(),
        owner: owner.into(),
    }) {
        Ok(()) => {
            println!("SUCCESS: registered parent domain {parent} with owner {owner}");
        }
        Err(e) => {
            eprintln!("ERROR: Failed to register parent domain: {e:?}");
        }
    }
}

pub fn run_add_controller(config: NetworkConfig, parent: &str, controller: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
    );

    match client.add_controller(AddControllerRequest {
        parent: parent.into(),
        controller: controller.into(),
    }) {
        Ok(()) => {
            println!("SUCCESS: added controller {controller} to parent domain {parent}");
        }
        Err(e) => {
            eprintln!("ERROR: Failed to add controller: {e:?}");
        }
    }
}

pub fn run_create_subdomain(config: NetworkConfig, label: &str, parent: &str, owner: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
    );

    match client.create_subdomain(CreateSubdomainRequest {
        label: label.into(),
        parent: parent.into(),
        owner: owner.into(),
    }) {
        Ok(fqdn) => {
            println!("SUCCESS: created subdomain {fqdn} with owner {owner}");
        }
        Err(e) => {
            eprintln!("ERROR: Failed to create subdomain: {e:?}");
        }
    }
}

pub fn run_transfer_subdomain(config: NetworkConfig, fqdn: &str, new_owner: &str) {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
    );

    match client.transfer_subdomain(TransferSubdomainRequest {
        fqdn: fqdn.into(),
        new_owner: new_owner.into(),
    }) {
        Ok(()) => {
            println!("SUCCESS: transferred subdomain {fqdn} to new owner {new_owner}");
        }
        Err(e) => {
            eprintln!("ERROR: Failed to transfer subdomain: {e:?}");
        }
    }
}