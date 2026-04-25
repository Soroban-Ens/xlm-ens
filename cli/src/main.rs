mod commands;
mod config;
mod signer;

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
    ///
    /// Signing material is never taken from the command line. Pass `--signer
    /// <profile>` to select a profile; the public address is read from
    /// `XLM_NS_SIGNER_<PROFILE>_PUBLIC` and the secret from
    /// `XLM_NS_SIGNER_<PROFILE>_SECRET`.
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
    ///
    /// Requires the resolver contract (RESOLVER_CONTRACT_ID env var, or the
    /// network default) to expose a reverse mapping entry for the address.
    ReverseLookup {
        /// Address to reverse-lookup
        address: String,
    },
    /// Read or mutate resolver text records.
    ///
    /// Read and write operations target the resolver contract
    /// (RESOLVER_CONTRACT_ID env var); write operations additionally require
    /// a signer profile (see `register --help`).
    #[command(subcommand)]
    Text(TextCommand),
    /// Transfer ownership of a name.
    ///
    /// `--signer <profile>` selects the account that will authorize the
    /// transfer. See `register --help` for how signer profiles are loaded.
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
    ///
    /// `--signer <profile>` selects the account that will authorize the
    /// renewal. See `register --help` for how signer profiles are loaded.
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
    /// Start or participate in an auction
    Auction {
        /// Name for auction
        name: String,
        /// Reserve price
        #[arg(default_value_t = 0)]
        reserve: u64,
    },
    /// Generate a shell completion script.
    ///
    /// Installation examples:
    ///   bash:  xlm-ns completions bash > /etc/bash_completion.d/xlm-ns
    ///   zsh:   xlm-ns completions zsh > "${fpath[1]}/_xlm-ns"
    ///   fish:  xlm-ns completions fish > ~/.config/fish/completions/xlm-ns.fish
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
    ///
    /// Omitting `<value>` clears the record. Requires a signer profile
    /// (see `register --help`).
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

    if let Commands::Completions { shell } = cli.command {
        commands::completions::run_completions::<Cli>(shell, BIN_NAME);
        return;
    }

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
        Commands::Register { name, owner, signer } => {
            commands::register::run_register(config, &name, &owner, resolve_signer(signer));
        }
        Commands::Resolve { name } => {
            commands::resolve::run_resolve(config, &name);
        }
        Commands::ReverseLookup { address } => {
            commands::reverse::run_reverse(config, &address);
        }
        Commands::Text(sub) => match sub {
            TextCommand::Get { name, key } => {
                commands::text::run_get(config, &name, &key);
            }
            TextCommand::Set { name, key, value, signer } => {
                commands::text::run_set(config, &name, &key, value, resolve_signer(signer));
            }
        },
        Commands::Transfer { name, new_owner, signer } => {
            commands::transfer::run_transfer(
                config,
                &name,
                &new_owner,
                resolve_signer(signer),
            );
        }
        Commands::Renew { name, years, signer } => {
            commands::renew::run_renew(config, &name, years, resolve_signer(signer));
        }
        Commands::Auction { name, reserve } => {
            commands::auction::run_auction(config, &name, reserve);
        }
        Commands::Completions { .. } => unreachable!("handled above"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;

    // Helper to spawn a command for the CLI binary
    fn cli() -> Command {
        Command::cargo_bin("xlm-ns-cli").expect("binary should be present")
    }

    #[test]
    fn test_help_output_success() {
        cli()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("XLM Name Service CLI"));
    }

    #[test]
    fn test_invalid_command_nonzero_exit() {
        cli()
            .arg("not-a-real-command")
            .assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized subcommand"));
    }

    #[test]
    fn test_register_missing_args_nonzero_exit() {
        cli()
            .arg("register")
            .assert()
            .failure();
    }

    #[test]
    fn test_resolve_missing_args_nonzero_exit() {
        cli()
            .arg("resolve")
            .assert()
            .failure();
    }

    #[test]
    fn test_reverse_lookup_missing_args_nonzero_exit() {
        cli()
            .arg("reverse-lookup")
            .assert()
            .failure();
    }

    #[test]
    fn test_text_missing_args_nonzero_exit() {
        cli()
            .arg("text")
            .arg("get")
            .assert()
            .failure();
    }

    #[test]
    fn test_transfer_missing_args_nonzero_exit() {
        cli()
            .arg("transfer")
            .assert()
            .failure();
    }

    #[test]
    fn test_renew_missing_args_nonzero_exit() {
        cli()
            .arg("renew")
            .assert()
            .failure();
    }

    #[test]
    fn test_auction_missing_args_nonzero_exit() {
        cli()
            .arg("auction")
            .assert()
            .failure();
    }

    #[test]
    fn test_bridge_missing_args_nonzero_exit() {
        cli()
            .arg("bridge")
            .assert()
            .failure();
    }

    // Success paths are tested via clap's try_parse_from to cleanly 
    // assert command structure validity without generating actual network/RPC calls.
    #[test]
    fn test_success_paths_parse_correctly() {
        assert!(Cli::try_parse_from(["xlm-ns", "register", "timmy.xlm", "G12345"]).is_ok());
        assert!(Cli::try_parse_from(["xlm-ns", "resolve", "timmy.xlm"]).is_ok());
        assert!(Cli::try_parse_from(["xlm-ns", "reverse-lookup", "G12345"]).is_ok());
        
        assert!(Cli::try_parse_from(["xlm-ns", "text", "get", "timmy.xlm", "url"]).is_ok());
        assert!(Cli::try_parse_from(["xlm-ns", "text", "set", "timmy.xlm", "url", "https://example.com"]).is_ok());
        
        assert!(Cli::try_parse_from(["xlm-ns", "transfer", "timmy.xlm", "G12345"]).is_ok());
        assert!(Cli::try_parse_from(["xlm-ns", "renew", "timmy.xlm", "2"]).is_ok());
        assert!(Cli::try_parse_from(["xlm-ns", "auction", "timmy.xlm", "100"]).is_ok());
        assert!(Cli::try_parse_from(["xlm-ns", "completions", "bash"]).is_ok());
    }
}
