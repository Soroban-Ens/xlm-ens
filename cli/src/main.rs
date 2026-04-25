mod commands;
mod config;
mod signer;

use anyhow::Context;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use config::Network;
use signer::{load_profile, SignerProfile};
use std::process;

const BIN_NAME: &str = "xlm-ns";

#[derive(Parser)]
#[command(name = BIN_NAME)]
#[command(about = "XLM Name Service CLI", long_about = None)]
struct Cli {
    /// Network to use (testnet, mainnet)
    #[arg(short, long, default_value = "testnet", global = true)]
    network: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new name.
    Register {
        /// Name to register
        name: String,
        /// Owner address
        owner: String,
        /// Signer profile to use for submission
        #[arg(long)]
        signer: Option<String>,
    },
    /// Resolve a name to an address
    Resolve {
        /// Name to resolve
        name: String,
    },
    /// Reverse-resolve an address to its primary name.
    ReverseLookup {
        /// Address to reverse-lookup
        address: String,
    },
    /// Read or mutate resolver text records.
    #[command(subcommand)]
    Text(TextCommand),
    /// Transfer ownership of a name.
    Transfer {
        /// Name to transfer
        name: String,
        /// New owner address
        new_owner: String,
        /// Signer profile to use for submission
        #[arg(long)]
        signer: Option<String>,
    },
    /// Renew a name registration.
    Renew {
        /// Name to renew
        name: String,
        /// Additional years to renew for
        #[arg(default_value_t = 1)]
        years: u64,
        /// Signer profile to use for submission
        #[arg(long)]
        signer: Option<String>,
    },
    /// Manage auctions for names
    #[command(subcommand)]
    Auction(AuctionCommands),
    /// Generate a shell completion script.
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Bridge management commands
    Bridge {
        #[command(subcommand)]
        command: BridgeCommands,
    },
    /// Subdomain management commands
    Subdomain {
        #[command(subcommand)]
        command: SubdomainCommands,
    },
}

#[derive(Subcommand)]
enum AuctionCommands {
    /// Create a new auction for a name
    Create {
        /// Name to auction
        name: String,
        /// Reserve price in XLM
        #[arg(long, default_value_t = 0)]
        reserve: u64,
        /// Auction duration in seconds
        #[arg(long, default_value_t = 86400)]
        duration: u64,
        /// Signer profile
        #[arg(long)]
        signer: Option<String>,
    },
    /// Place a bid on an active auction
    Bid {
        /// Name under auction
        name: String,
        /// Bid amount in XLM
        amount: u64,
        /// Signer profile
        #[arg(long)]
        signer: Option<String>,
    },
    /// Inspect the state of an auction
    Inspect {
        /// Name to inspect
        name: String,
    },
    /// Settle a completed auction
    Settle {
        /// Name to settle
        name: String,
        /// Signer profile
        #[arg(long)]
        signer: Option<String>,
    },
}

#[derive(Subcommand)]
enum BridgeCommands {
    /// Register a bridge route for a remote chain
    Register {
        /// Remote chain name
        chain: String,
    },
    /// Inspect a bridge route
    Inspect {
        /// Chain name
        chain: String,
    },
    /// Generate a resolution payload for bridging
    Payload {
        /// Name to bridge
        name: String,
        /// Target chain
        chain: String,
    },
}

#[derive(Subcommand)]
enum SubdomainCommands {
    /// Register a parent domain for subdomains
    RegisterParent {
        /// Parent name (e.g. "com")
        name: String,
        /// Owner address
        owner: String,
    },
    /// Add a controller to a parent domain
    AddController {
        /// Parent name
        parent: String,
        /// Controller address
        controller: String,
    },
    /// Create a subdomain
    Create {
        /// Subdomain label
        label: String,
        /// Parent name
        parent: String,
        /// Owner address
        owner: String,
    },
    /// Transfer subdomain ownership
    Transfer {
        /// Full subdomain FQDN
        fqdn: String,
        /// New owner address
        new_owner: String,
    },
}

#[derive(Subcommand)]
enum TextCommand {
    /// Read a text record value for a name.
    Get {
        /// Name to query
        name: String,
        /// Text record key (e.g. "url", "email", "avatar")
        key: String,
    },
    /// Write a text record value on a name.
    Set {
        /// Name to update
        name: String,
        /// Text record key
        key: String,
        /// New value (omit to clear the record)
        value: Option<String>,
        /// Signer profile to use for submission
        #[arg(long)]
        signer: Option<String>,
    },
}

fn resolve_signer(name: Option<String>) -> anyhow::Result<Option<SignerProfile>> {
    let name = match name {
        Some(n) => n,
        None => return Ok(None),
    };
    load_profile(&name).map(Some).context("failed to load signer profile")
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Commands::Completions { shell } = cli.command {
        commands::completions::run_completions::<Cli>(shell, BIN_NAME);
        return Ok(());
    }

    let network = Network::parse(&cli.network)
        .with_context(|| format!("invalid network '{}'", cli.network))?;

    let config = network.config();

    match cli.command {
        Commands::Register { name, owner, signer } => {
            commands::register::run_register(config, &name, &owner, resolve_signer(signer)?).await
        }
        Commands::Resolve { name } => {
            commands::resolve::run_resolve(config, &name).await
        }
        Commands::ReverseLookup { address } => {
            commands::reverse::run_reverse(config, &address).await
        }
        Commands::Text(sub) => match sub {
            TextCommand::Get { name, key } => {
                commands::text::run_get(config, &name, &key).await
            }
            TextCommand::Set { name, key, value, signer } => {
                commands::text::run_set(config, &name, &key, value, resolve_signer(signer)?).await
            }
        },
        Commands::Transfer { name, new_owner, signer } => {
            commands::transfer::run_transfer(config, &name, &new_owner, resolve_signer(signer)?).await
        }
        Commands::Renew { name, years, signer } => {
            commands::renew::run_renew(config, &name, years, resolve_signer(signer)?).await
        }
        Commands::Auction(sub) => match sub {
            AuctionCommands::Create { name, reserve, duration, signer } => {
                commands::auction::run_create(config, &name, reserve, duration, resolve_signer(signer)?).await
            }
            AuctionCommands::Bid { name, amount, signer } => {
                commands::auction::run_bid(config, &name, amount, resolve_signer(signer)?).await
            }
            AuctionCommands::Inspect { name } => {
                commands::auction::run_inspect(config, &name).await
            }
            AuctionCommands::Settle { name, signer } => {
                commands::auction::run_settle(config, &name, resolve_signer(signer)?).await
            }
        },
        Commands::Bridge { command } => match command {
            BridgeCommands::Register { chain } => {
                commands::bridge::run_register_chain(config, &chain).await
            }
            BridgeCommands::Inspect { chain } => {
                commands::bridge::run_inspect_route(config, &chain).await
            }
            BridgeCommands::Payload { name, chain } => {
                commands::bridge::run_generate_payload(config, &name, &chain).await
            }
        },
        Commands::Subdomain { command } => match command {
            SubdomainCommands::RegisterParent { name, owner } => {
                commands::subdomain::run_register_parent(config, &name, &owner).await
            }
            SubdomainCommands::AddController { parent, controller } => {
                commands::subdomain::run_add_controller(config, &parent, &controller).await
            }
            SubdomainCommands::Create { label, parent, owner } => {
                commands::subdomain::run_create_subdomain(config, &label, &parent, &owner).await
            }
            SubdomainCommands::Transfer { fqdn, new_owner } => {
                commands::subdomain::run_transfer_subdomain(config, &fqdn, &new_owner).await
            }
        },
        Commands::Completions { .. } => unreachable!("handled above"),
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:?}", e);
        process::exit(1);
    }
}
