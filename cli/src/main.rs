mod commands;
mod config;
mod output;
mod signer;

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use config::{load_config, ContractKind, ContractOverrides, Network, ResolveOptions};
use output::OutputFormat;
use signer::{load_profile, SignerProfile};
use std::path::PathBuf;
use std::process;

const BIN_NAME: &str = "xlm-ns";

#[derive(Parser)]
#[command(name = BIN_NAME)]
#[command(about = "XLM Name Service CLI", long_about = None)]
struct Cli {
    /// Network to use (`testnet` or `mainnet`)
    #[arg(short, long, default_value = "testnet", global = true)]
    network: String,

    /// Config file path. Falls back to `XLM_NS_CONFIG`, then the documented search path.
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Output format for command results.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human, global = true)]
    output: OutputFormat,

    /// Override the Soroban RPC URL.
    #[arg(long, global = true)]
    rpc_url: Option<String>,

    /// Override the Soroban network passphrase.
    #[arg(long, global = true)]
    network_passphrase: Option<String>,

    #[arg(long, global = true)]
    registry_contract_id: Option<String>,

    #[arg(long, global = true)]
    registrar_contract_id: Option<String>,

    #[arg(long, global = true)]
    resolver_contract_id: Option<String>,

    #[arg(long, global = true)]
    auction_contract_id: Option<String>,

    #[arg(long, global = true)]
    bridge_contract_id: Option<String>,

    #[arg(long, global = true)]
    subdomain_contract_id: Option<String>,

    #[arg(long, global = true)]
    nft_contract_id: Option<String>,

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
    /// Resolve a name to an address.
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
    /// Start or inspect an auction using the auction contract.
    Auction {
        /// Name for auction
        name: String,
        /// Reserve price
        #[arg(default_value_t = 0)]
        reserve: u64,
    },
    /// Generate a shell completion script.
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Bridge management commands.
    Bridge {
        #[command(subcommand)]
        command: BridgeCommands,
    },
    /// Subdomain management commands.
    Subdomain {
        #[command(subcommand)]
        command: SubdomainCommands,
    },
    /// Inspect NFT ownership metadata.
    Nft {
        #[command(subcommand)]
        command: NftCommands,
    },
    /// Show registration details for a single name.
    Whois {
        /// Name to inspect
        name: String,
    },
    /// List names owned by an address.
    Portfolio {
        /// Owner address to inspect
        owner: String,
    },
    /// Fetch a registration price quote without submitting a transaction (read-only).
    ///
    /// Use this to inspect the full fee breakdown and lifecycle timestamps before
    /// deciding whether to register a name.
    Quote {
        /// Name label to quote (without the .xlm suffix)
        name: String,
        /// Number of years to quote for
        #[arg(default_value_t = 1)]
        years: u32,
    },
    /// Check whether a name is available for registration (read-only).
    ///
    /// Outputs the availability status: available, active, grace-period, or claimable.
    /// No transaction is submitted.
    Availability {
        /// Name to check (e.g. `alice.xlm` or just `alice`)
        name: String,
    },
}

#[derive(Subcommand)]
enum TextCommand {
    /// Read a text record value for a name.
    Get { name: String, key: String },
    /// Write a text record value on a name.
    Set {
        name: String,
        key: String,
        value: Option<String>,
        #[arg(long)]
        signer: Option<String>,
    },
}

#[derive(Subcommand)]
enum BridgeCommands {
    /// Register a supported bridge route.
    RegisterChain { chain: String },
    /// Inspect a configured bridge route.
    InspectRoute { chain: String },
    /// Generate a bridge payload for a name.
    GeneratePayload { name: String, chain: String },
}

#[derive(Subcommand)]
enum SubdomainCommands {
    /// Register a parent domain for delegated subdomain issuance.
    RegisterParent { parent: String, owner: String },
    /// Add a controller for a delegated parent domain.
    AddController { parent: String, controller: String },
    /// Create a subdomain beneath an already-registered parent.
    Create {
        label: String,
        parent: String,
        owner: String,
    },
    /// Transfer an existing subdomain.
    Transfer { fqdn: String, new_owner: String },
}

#[derive(Subcommand)]
enum NftCommands {
    /// Inspect the owner and metadata for a token id.
    Inspect { token_id: String },
}

fn resolve_signer(name: Option<String>) -> Option<SignerProfile> {
    let name = name?;
    match load_profile(&name) {
        Ok(profile) => Some(profile),
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if let Commands::Completions { shell } = &cli.command {
        commands::completions::run_completions::<Cli>(*shell, BIN_NAME);
        return;
    }

    let network = match Network::parse(&cli.network) {
        Some(network) => network,
        None => {
            eprintln!(
                "Error: Invalid network '{}'. Use 'testnet' or 'mainnet'.",
                cli.network
            );
            process::exit(1);
        }
    };

    let contract_overrides = ContractOverrides {
        registry_contract_id: cli.registry_contract_id.clone(),
        registrar_contract_id: cli.registrar_contract_id.clone(),
        resolver_contract_id: cli.resolver_contract_id.clone(),
        auction_contract_id: cli.auction_contract_id.clone(),
        bridge_contract_id: cli.bridge_contract_id.clone(),
        subdomain_contract_id: cli.subdomain_contract_id.clone(),
        nft_contract_id: cli.nft_contract_id.clone(),
    };

    let config = match load_config(
        network,
        ResolveOptions {
            config_path: cli.config.clone(),
            rpc_url: cli.rpc_url.clone(),
            network_passphrase: cli.network_passphrase.clone(),
            contract_overrides: contract_overrides.clone(),
        },
    ) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(1);
        }
    };

    if let Err(err) = validate_contract_policy(&cli.command, &contract_overrides, &config) {
        eprintln!("Error: {err}");
        process::exit(1);
    }

    match cli.command {
        Commands::Register {
            name,
            owner,
            signer,
        } => {
            commands::register::run_register(
                config,
                cli.output,
                &name,
                &owner,
                resolve_signer(signer),
            );
        }
        Commands::Resolve { name } => {
            commands::resolve::run_resolve(config, cli.output, &name);
        }
        Commands::ReverseLookup { address } => {
            commands::reverse::run_reverse(config, cli.output, &address);
        }
        Commands::Text(subcommand) => match subcommand {
            TextCommand::Get { name, key } => {
                commands::text::run_get(config, cli.output, &name, &key);
            }
            TextCommand::Set {
                name,
                key,
                value,
                signer,
            } => {
                commands::text::run_set(
                    config,
                    cli.output,
                    &name,
                    &key,
                    value,
                    resolve_signer(signer),
                );
            }
        },
        Commands::Transfer {
            name,
            new_owner,
            signer,
        } => {
            commands::transfer::run_transfer(
                config,
                cli.output,
                &name,
                &new_owner,
                resolve_signer(signer),
            );
        }
        Commands::Renew {
            name,
            years,
            signer,
        } => {
            commands::renew::run_renew(config, cli.output, &name, years, resolve_signer(signer));
        }
        Commands::Auction { name, reserve } => {
            commands::auction::run_auction(config, cli.output, &name, reserve);
        }
        Commands::Bridge { command } => match command {
            BridgeCommands::RegisterChain { chain } => {
                commands::bridge::run_register_chain(config, cli.output, &chain);
            }
            BridgeCommands::InspectRoute { chain } => {
                commands::bridge::run_inspect_route(config, cli.output, &chain);
            }
            BridgeCommands::GeneratePayload { name, chain } => {
                commands::bridge::run_generate_payload(config, cli.output, &name, &chain);
            }
        },
        Commands::Subdomain { command } => match command {
            SubdomainCommands::RegisterParent { parent, owner } => {
                commands::subdomain::run_register_parent(config, cli.output, &parent, &owner);
            }
            SubdomainCommands::AddController { parent, controller } => {
                commands::subdomain::run_add_controller(config, cli.output, &parent, &controller);
            }
            SubdomainCommands::Create {
                label,
                parent,
                owner,
            } => {
                commands::subdomain::run_create_subdomain(
                    config, cli.output, &label, &parent, &owner,
                );
            }
            SubdomainCommands::Transfer { fqdn, new_owner } => {
                commands::subdomain::run_transfer_subdomain(config, cli.output, &fqdn, &new_owner);
            }
        },
        Commands::Nft { command } => match command {
            NftCommands::Inspect { token_id } => {
                commands::nft::run_inspect(config, cli.output, &token_id);
            }
        },
        Commands::Whois { name } => {
            commands::whois::run_whois(config, cli.output, &name);
        }
        Commands::Portfolio { owner } => {
            commands::portfolio::run_portfolio(config, cli.output, &owner);
        }
        Commands::Quote { name, years } => {
            commands::quote::run_quote(config, cli.output, &name, years);
        }
        Commands::Availability { name } => {
            commands::quote::run_availability(config, cli.output, &name);
        }
        Commands::Completions { .. } => unreachable!("handled above"),
    }
}

fn validate_contract_policy(
    command: &Commands,
    overrides: &ContractOverrides,
    config: &config::NetworkConfig,
) -> Result<(), String> {
    let (command_name, allowed, required): (&str, &[ContractKind], &[ContractKind]) = match command
    {
        Commands::Register { .. } => (
            "register",
            &[ContractKind::Registrar],
            &[ContractKind::Registrar],
        ),
        Commands::Resolve { .. } => (
            "resolve",
            &[ContractKind::Resolver],
            &[ContractKind::Resolver],
        ),
        Commands::ReverseLookup { .. } => (
            "reverse-lookup",
            &[ContractKind::Resolver],
            &[ContractKind::Resolver],
        ),
        Commands::Text(_) => ("text", &[ContractKind::Resolver], &[ContractKind::Resolver]),
        Commands::Transfer { .. } => (
            "transfer",
            &[ContractKind::Registry],
            &[ContractKind::Registry],
        ),
        Commands::Renew { .. } => (
            "renew",
            &[ContractKind::Registrar],
            &[ContractKind::Registrar],
        ),
        Commands::Auction { .. } => (
            "auction",
            &[ContractKind::Auction],
            &[ContractKind::Auction],
        ),
        Commands::Completions { .. } => ("completions", &[], &[]),
        Commands::Bridge { .. } => ("bridge", &[ContractKind::Bridge], &[ContractKind::Bridge]),
        Commands::Subdomain { .. } => (
            "subdomain",
            &[ContractKind::Subdomain],
            &[ContractKind::Subdomain],
        ),
        Commands::Nft { .. } => ("nft", &[ContractKind::Nft], &[ContractKind::Nft]),
        Commands::Whois { .. } => (
            "whois",
            &[ContractKind::Registry, ContractKind::Resolver],
            &[ContractKind::Registry],
        ),
        Commands::Portfolio { .. } => (
            "portfolio",
            &[ContractKind::Registry, ContractKind::Resolver],
            &[ContractKind::Registry],
        ),
        // Quote and Availability are read-only; registrar is needed for pricing.
        Commands::Quote { .. } => (
            "quote",
            &[ContractKind::Registrar],
            &[ContractKind::Registrar],
        ),
        Commands::Availability { .. } => (
            "availability",
            &[ContractKind::Registry],
            &[],
        ),
    };

    for kind in overrides.provided_kinds() {
        if !allowed.contains(&kind) {
            return Err(format!(
                "`--{}` cannot be used with `{command_name}`",
                kind.flag_name()
            ));
        }
    }

    for kind in required {
        if config.contract_id(*kind).is_none() {
            return Err(format!(
                "`{command_name}` requires {}. Set `--{}`, `{}`, or the config file value.",
                kind.display_name(),
                kind.flag_name(),
                kind.env_var()
            ));
        }
    }

    Ok(())
}
