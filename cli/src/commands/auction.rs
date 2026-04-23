use crate::config::NetworkConfig;

pub fn run_auction(config: NetworkConfig, name: &str, reserve: u64) {
    println!("Preparing auction for {name}...");
    println!("  Network: {}", config.rpc_url);
    println!("  Registry: {:?}", config.registry_contract_id);
    println!("  Reserve Price: {} XLM", reserve);

    println!("\nAuction surface initialized (placeholder).");
}
