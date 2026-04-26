//! Blocking-client example for callers in synchronous codebases.
//!
//! ```sh
//! cargo run -p xlm-ns-sdk --example blocking_client -- \
//!     https://soroban-rpc.example CDAD...REGISTRY alice.xlm
//! ```
//!
//! Compared to `async_client.rs` this example does not use `#[tokio::main]`;
//! the `XlmNsBlockingClient` owns its own current-thread runtime and drives
//! the underlying async methods to completion on the caller's thread.

use std::env;
use std::time::Duration;

use xlm_ns_sdk::{types::RenewalRequest, ClientConfig, XlmNsBlockingClient, XlmNsClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let rpc_url = args
        .first()
        .cloned()
        .unwrap_or_else(|| "http://localhost:8000/soroban/rpc".into());
    let registry_id = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "CDAD...REGISTRY".into());
    let name = args.get(2).cloned().unwrap_or_else(|| "alice.xlm".into());

    let async_client = XlmNsClient::builder(rpc_url)
        .registry(registry_id)
        .registrar("CDAD...REGISTRAR")
        .config(
            ClientConfig::default()
                .with_timeout(Duration::from_secs(10))
                .with_user_agent("blocking-client-example/0.1"),
        )
        .build();
    let client = XlmNsBlockingClient::from_async(async_client)?;

    let resolution = client.resolve(&name)?;
    println!("{name} -> {:?}", resolution.address);

    let receipt = client.renew(RenewalRequest {
        name: name.clone(),
        additional_years: 1,
        signer: Some("treasury".into()),
    })?;
    println!(
        "renewed {name}: tx_hash={}, fee_paid={}",
        receipt.submission.tx_hash, receipt.fee_paid,
    );
    Ok(())
}
