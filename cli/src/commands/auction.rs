use crate::config::NetworkConfig;
use crate::signer::SignerProfile;
use anyhow::{anyhow, Context};
use xlm_ns_sdk::client::XlmNsClient;
use xlm_ns_sdk::types::{AuctionCreateRequest, BidRequest};
use crate::output::OutputFormat;
use serde_json::json;
use std::io::{self, Write};

pub async fn run_create(
    config: NetworkConfig,
    name: &str,
    reserve: u64,
    duration: u64,
    signer: Option<SignerProfile>,
) -> anyhow::Result<()> {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
        config.auction_contract_id.clone(),
    );

    println!("Creating auction for {name}...");
    if let Some(ref s) = signer {
        println!("  Signer: {}", s.describe());
    }
    let treasury = signer
        .as_ref()
        .map(|s| s.public_address.clone())
        .unwrap_or_else(|| format!("G{}", "A".repeat(55)));

    let submission = client
        .create_auction(AuctionCreateRequest {
            name: name.into(),
            asset: "XLM".to_string(),
            treasury,
            reserve_price: reserve,
            duration_seconds: duration,
            signer: signer.as_ref().map(|s| s.name.clone()),
        })
        .await
        .context("Failed to create auction")?;

    println!("SUCCESS: auction created for {name}");
    println!("  Reserve: {reserve} XLM");
    println!("  Duration: {duration}s");
    println!("  Transaction Hash: {}", submission.tx_hash);

    Ok(())
}

pub async fn run_bid(
    config: NetworkConfig,
    name: &str,
    amount: u64,
    signer: Option<SignerProfile>,
) -> anyhow::Result<()> {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
        config.auction_contract_id.clone(),
    );

    println!("Preparing to place bid of {amount} XLM on {name}...");

    if let Some(ref s) = signer {
        println!("  Signer: {}", s.describe());
    }

    // Fetch auction to validate state and make basic balance/bid checks
    let auction = client
        .get_auction(name)
        .await
        .context("Failed to fetch auction state for validation")?
        .ok_or_else(|| anyhow!("No active auction found for '{}'", name))?;

    if auction.status.to_string() != "active" {
        return Err(anyhow!("auction for '{}' is not active", name));
    }

    if amount == 0 {
        return Err(anyhow!("bid amount must be greater than zero"));
    }

    let current_highest = auction.highest_bid;
    if amount <= current_highest {
        return Err(anyhow!(
            "bid amount must be greater than current highest bid ({} XLM)",
            current_highest
        ));
    }

    println!("  Current highest bid: {} XLM", current_highest);

    // Confirmation prompt
    print!("Confirm sending bid of {amount} XLM on {name}? [y/N]: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let confirmed = matches!(input.trim().to_lowercase().as_str(), "y" | "yes");
    if !confirmed {
        println!("Aborted by user");
        return Ok(());
    }

    let submission = client
        .bid_auction(BidRequest {
            name: name.into(),
            amount,
            signer: signer.as_ref().map(|s| s.name.clone()),
        })
        .await
        .context("Failed to place bid")?;

    println!("SUCCESS: bid placed on {name}");
    println!("  Transaction Hash: {}", submission.tx_hash);

    Ok(())
}

pub async fn run_list(
    config: NetworkConfig,
    format: OutputFormat,
    name_length: Option<usize>,
    min_bid: Option<u64>,
    max_bid: Option<u64>,
    min_time_remaining: Option<u64>,
    page: Option<usize>,
    limit: usize,
) -> anyhow::Result<()> {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
        config.auction_contract_id.clone(),
    );

    let (items, _next) = client.list_auctions_page(page, limit).await.context("Failed to list auctions")?;

    let filtered: Vec<_> = items
        .into_iter()
        .filter(|a| {
            if let Some(len) = name_length {
                let label = a.name.split('.').next().unwrap_or("");
                if label.len() != len {
                    return false;
                }
            }
            if let Some(min) = min_bid {
                if a.highest_bid < min {
                    return false;
                }
            }
            if let Some(max) = max_bid {
                if a.highest_bid > max {
                    return false;
                }
            }
            if let Some(min_time) = min_time_remaining {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if a.ends_at.saturating_sub(now) < min_time {
                    return false;
                }
            }
            true
        })
        .collect();

    match format {
        OutputFormat::Human => {
            println!("{:<24} {:<8} {:<8} {:<12} {}", "Name", "Status", "Reserve", "HighestBid", "EndsIn(s)");
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            for a in filtered {
                let ends_in = a.ends_at.saturating_sub(now);
                println!("{:<24} {:<8} {:<8} {:<12} {}", a.name, a.status, a.reserve_price, a.highest_bid, ends_in);
            }
        }
        OutputFormat::Json => {
            let arr: Vec<_> = filtered
                .into_iter()
                .map(|a| json!({"name": a.name, "status": a.status.to_string(), "reserve": a.reserve_price, "highest_bid": a.highest_bid, "highest_bidder": a.highest_bidder, "ends_at": a.ends_at}))
                .collect();
            crate::output::emit(OutputFormat::Json, "", json!(arr));
        }
        OutputFormat::Csv => {
            let arr: Vec<_> = filtered
                .into_iter()
                .map(|a| json!({"name": a.name, "status": a.status.to_string(), "reserve": a.reserve_price, "highest_bid": a.highest_bid, "highest_bidder": a.highest_bidder, "ends_at": a.ends_at}))
                .collect();
            crate::output::emit(OutputFormat::Csv, "", json!(arr));
        }
    }

    Ok(())
}

pub async fn run_show(
    config: NetworkConfig,
    format: OutputFormat,
    name: &str,
) -> anyhow::Result<()> {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
        config.auction_contract_id.clone(),
    );

    let auction = client
        .get_auction(name)
        .await
        .context("Failed to fetch auction state")?
        .ok_or_else(|| anyhow!("No auction found for '{}'", name))?;

    match format {
        OutputFormat::Human => {
            println!("Auction for {}:", auction.name);
            println!("  Status: {}", auction.status);
            println!("  Owner: {}", auction.owner);
            println!("  Reserve Price: {} XLM", auction.reserve_price);
            println!("  Highest Bid: {} XLM", auction.highest_bid);
            if let Some(bidder) = auction.highest_bidder {
                println!("  Highest Bidder: {}", bidder);
            }
            println!("  Ends at: {}", auction.ends_at);
            println!("  Bid history: not available via read API");
        }
        OutputFormat::Json => {
            crate::output::emit(OutputFormat::Json, "", json!(auction));
        }
        OutputFormat::Csv => {
            crate::output::emit(OutputFormat::Csv, "", json!(auction));
        }
    }

    Ok(())
}

pub async fn run_my_bids(
    config: NetworkConfig,
    format: OutputFormat,
    signer: Option<SignerProfile>,
) -> anyhow::Result<()> {
    let signer = signer.ok_or_else(|| anyhow!("signer is required for my-bids"))?;
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
        config.auction_contract_id.clone(),
    );

    let all = client.list_auctions().await.context("Failed to list auctions")?;
    let public = signer.public_address.clone();
    let my: Vec<_> = all
        .into_iter()
        .filter(|a| a.highest_bidder.as_deref() == Some(public.as_str()))
        .collect();

    match format {
        OutputFormat::Human => {
            if my.is_empty() {
                println!("No active/past top bids found for {}", signer.public_address);
            } else {
                println!("Bids for {}:", signer.public_address);
                for a in my {
                    println!("  {} - highest: {} XLM (ends: {})", a.name, a.highest_bid, a.ends_at);
                }
            }
        }
        OutputFormat::Json => crate::output::emit(OutputFormat::Json, "", json!(my)),
        OutputFormat::Csv => crate::output::emit(OutputFormat::Csv, "", json!(my)),
    }

    Ok(())
}

pub async fn run_inspect(config: NetworkConfig, name: &str) -> anyhow::Result<()> {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
        config.auction_contract_id.clone(),
    );

    let auction = client
        .get_auction(name)
        .await
        .context("Failed to fetch auction state")?
        .ok_or_else(|| anyhow!("No active auction found for '{}'", name))?;

    println!("Auction for {}:", auction.name);
    println!("  Status: {}", auction.status);
    println!("  Owner: {}", auction.owner);
    println!("  Reserve Price: {} XLM", auction.reserve_price);
    println!("  Highest Bid: {} XLM", auction.highest_bid);
    if let Some(bidder) = auction.highest_bidder {
        println!("  Highest Bidder: {}", bidder);
    }
    println!("  Ends at: {}", auction.ends_at);

    Ok(())
}

pub async fn run_settle(
    config: NetworkConfig,
    name: &str,
    signer: Option<SignerProfile>,
) -> anyhow::Result<()> {
    let client = XlmNsClient::new(
        config.rpc_url,
        Some(config.network_passphrase),
        config.registry_contract_id.clone(),
        config.subdomain_contract_id.clone(),
        config.bridge_contract_id.clone(),
        config.auction_contract_id.clone(),
    );

    println!("Settling auction for {name}...");
    if let Some(ref s) = signer {
        println!("  Signer: {}", s.describe());
    }

    let submission = client
        .settle_auction(name, signer.as_ref().map(|s| s.name.clone()))
        .await
        .context("Failed to settle auction")?;

    println!("SUCCESS: auction settled for {name}");
    println!("  Transaction Hash: {}", submission.tx_hash);

    Ok(())
}
//
