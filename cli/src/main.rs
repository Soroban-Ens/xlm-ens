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
    /// Subdomain management commands
    /// 
    /// Subdomain flow:
    /// 1. Register a parent domain: xlm-ns subdomain register-parent example.xlm <owner>
    /// 2. Add controllers (optional): xlm-ns subdomain add-controller example.xlm <controller>
    /// 3. Create subdomains: xlm-ns subdomain create sub example.xlm <owner>
    /// 4. Transfer subdomains: xlm-ns subdomain transfer sub.example.xlm <new_owner>
    Subdomain {
        #[command(subcommand)]
        command: SubdomainCommands,
    },
}

#[derive(Subcommand)]
enum SubdomainCommands {
    /// Register a parent domain for subdomain management
    /// This enables the parent domain owner to create and manage subdomains
    RegisterParent {
        /// Parent domain name (e.g., example.xlm)
        parent: String,
        /// Owner address for the parent domain
        owner: String,
    },
    /// Add a controller to a parent domain
    /// Controllers can create subdomains under the parent domain
    AddController {
        /// Parent domain name
        parent: String,
        /// Controller address to add (must be called by parent owner)
        controller: String,
    },
    /// Create a subdomain under a registered parent
    /// Can be called by parent owner or authorized controllers
    Create {
        /// Subdomain label (e.g., 'sub' for sub.example.xlm)
        label: String,
        /// Parent domain name
        parent: String,
        /// Owner address for the new subdomain
        owner: String,
    },
    /// Transfer ownership of a subdomain
    /// Can only be called by the current subdomain owner
    Transfer {
        /// Full subdomain name (e.g., sub.example.xlm)
        fqdn: String,
        /// New owner address
        new_owner: String,
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
        Commands::Subdomain { command } => match command {
            SubdomainCommands::RegisterParent { parent, owner } => {
                commands::subdomain::run_register_parent(config, &parent, &owner);
            }
            SubdomainCommands::AddController { parent, controller } => {
                commands::subdomain::run_add_controller(config, &parent, &controller);
            }
            SubdomainCommands::Create { label, parent, owner } => {
                commands::subdomain::run_create_subdomain(config, &label, &parent, &owner);
            }
            SubdomainCommands::Transfer { fqdn, new_owner } => {
                commands::subdomain::run_transfer_subdomain(config, &fqdn, &new_owner);
            }
        }
    }
}
