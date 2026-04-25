use crate::config::NetworkConfig;
use crate::output::{emit, OutputFormat};
use serde_json::json;

pub fn run_auction(config: NetworkConfig, output: OutputFormat, name: &str, reserve: u64) {
    let auction_contract_id = config
        .auction_contract_id
        .expect("auction command validated auction contract id");
    let human = format!(
        "Preparing auction for {name}...\n  Network: {}\n  Auction: {}\n  Reserve Price: {} XLM\n\nAuction surface initialized (placeholder).",
        config.rpc_url, auction_contract_id, reserve
    );

    emit(
        output,
        &human,
        json!({
            "name": name,
            "reserve": reserve,
            "rpc_url": config.rpc_url,
            "network": config.network.as_str(),
            "auction_contract_id": auction_contract_id,
            "status": "initialized",
        }),
    );
}
