use anyhow::{Context, anyhow};
use crate::config::NetworkConfig;
use xlm_ns_sdk::client::XlmNsClient;

pub async fn run_resolve(config: NetworkConfig, name: &str) -> anyhow::Result<()> {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        Some(config.registry_contract_id),
        Some(config.subdomain_contract_id),
        Some(config.bridge_contract_id),
        Some(config.auction_contract_id),
    );

    let result = client.resolve(name).await.context("Failed to resolve name")?;

    if let Some(addr) = result.address {
        println!("Name: {}", result.name);
        println!("Address: {}", addr);
        if let Some(resolver) = result.resolver {
            println!("Resolver: {}", resolver);
        }
        if let Some(expiry) = result.expires_at {
            println!("Expires at: {}", expiry);
        }
        Ok(())
    } else {
        Err(anyhow!("Name '{}' not found or has no resolution", name))
    }
}
