mod commands;
mod config;

use clap::{Parser, Subcommand};
use config::Network;
use std::process;

#[derive(Parser)]
#[command(name = "xlm-ns")]
#[command(about = "XLM Name Service CLI", long_about = None)]
struct Cli {
    /// Network to use (testnet, mainnet)
    #[arg(short, long, default_value = "testnet")]
    network: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new name
    Register {
        /// Name to register
        name: String,
        /// Owner address
        owner: String,
    },
    /// Resolve a name to an address
    Resolve {
        /// Name to resolve
        name: String,
    },
    /// Transfer ownership of a name
    Transfer {
        /// Name to transfer
        name: String,
        /// New owner address
        new_owner: String,
    },
    /// Renew a name registration
    Renew {
        /// Name to renew
        name: String,
        /// Additional years to renew for
        #[arg(default_value_t = 1)]
        years: u64,
    },
    /// Start or participate in an auction
    Auction {
        /// Name for auction
        name: String,
        /// Reserve price
        #[arg(default_value_t = 0)]
        reserve: u64,
    },
}

fn main() {
    let cli = Cli::parse();

    let network = match Network::parse(&cli.network) {
        Some(n) => n,
        None => {
            eprintln!(
                "Error: Invalid network '{}'. Use 'testnet' or 'mainnet'.",
                cli.network
            );
            process::exit(1);
        }
    };

    let config = network.config();

    match cli.command {
        Commands::Register { name, owner } => {
            commands::register::run_register(config, &name, &owner);
        }
        Commands::Resolve { name } => {
            commands::resolve::run_resolve(config, &name);
        }
        Commands::Transfer { name, new_owner } => {
            commands::transfer::run_transfer(config, &name, &new_owner);
        }
        Commands::Renew { name, years } => {
            commands::renew::run_renew(config, &name, years);
        }
        Commands::Auction { name, reserve } => {
            commands::auction::run_auction(config, &name, reserve);
        }
    }
}
