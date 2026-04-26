//! End-to-end async-client example.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p xlm-ns-sdk --example async_client -- \
//!     https://soroban-rpc.example CDAD...REGISTRY alice.xlm
//! ```
//!
//! Defaults are used when the args are omitted (RPC URL falls back to
//! `http://localhost:8000/soroban/rpc`). The example shows the canonical
//! shape of an async integration: configure the client with explicit
//! timeout / retry / user-agent for production observability, then issue
//! a read path (`resolve`) and a write path (`renew`).

use std::env;
use std::time::Duration;

use xlm_ns_sdk::{
    types::{RenewalRequest, SubmissionStatus},
    ClientConfig, XlmNsClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let client = XlmNsClient::builder(rpc_url)
        .registry(registry_id)
        .registrar("CDAD...REGISTRAR")
        .config(
            ClientConfig::default()
                .with_timeout(Duration::from_secs(15))
                .with_max_retries(2)
                .with_user_agent("async-client-example/0.1"),
        )
        .build();

    println!("resolving {name}...");
    let resolution = client.resolve(&name).await?;
    println!(
        "  -> address: {:?}, expires_at: {:?}",
        resolution.address, resolution.expires_at,
    );

    println!("renewing {name} for one year...");
    let receipt = client
        .renew(RenewalRequest {
            name: name.clone(),
            additional_years: 1,
            signer: Some("treasury".into()),
        })
        .await?;
    println!(
        "  -> tx_hash: {}, status: {:?}, fee_paid: {}",
        receipt.submission.tx_hash, receipt.submission.status, receipt.fee_paid,
    );

    if matches!(
        receipt.submission.status,
        SubmissionStatus::Submitted | SubmissionStatus::Confirmed
    ) {
        println!("done.");
    } else {
        eprintln!(
            "warning: unexpected submission status {:?}",
            receipt.submission.status
        );
    }
    Ok(())
}
