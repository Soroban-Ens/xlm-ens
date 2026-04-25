use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Testnet,
    Mainnet,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub network_passphrase: String,
    pub registry_contract_id: String,
    pub subdomain_contract_id: String,
    pub bridge_contract_id: String,
    pub auction_contract_id: String,
}

impl Network {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "testnet" => Some(Self::Testnet),
            "mainnet" => Some(Self::Mainnet),
            _ => None,
        }
    }

    pub fn config(&self) -> NetworkConfig {
        match self {
            Network::Testnet => NetworkConfig {
                rpc_url: env::var("SOROBAN_RPC_URL")
                    .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string()),
                network_passphrase: env::var("SOROBAN_NETWORK_PASSPHRASE")
                    .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
                registry_contract_id: env::var("REGISTRY_CONTRACT_ID")
                    .unwrap_or_else(|_| "CDAD...TESTNET_REGISTRY".to_string()),
                subdomain_contract_id: env::var("SUBDOMAIN_CONTRACT_ID")
                    .unwrap_or_else(|_| "CDAD...TESTNET_SUBDOMAIN".to_string()),
                bridge_contract_id: env::var("BRIDGE_CONTRACT_ID")
                    .unwrap_or_else(|_| "CDAD...TESTNET_BRIDGE".to_string()),
                auction_contract_id: env::var("AUCTION_CONTRACT_ID")
                    .unwrap_or_else(|_| "CDAD...TESTNET_AUCTION".to_string()),
            },
            Network::Mainnet => NetworkConfig {
                rpc_url: env::var("SOROBAN_RPC_URL")
                    .unwrap_or_else(|_| "https://mainnet.stellar.org:443".to_string()),
                network_passphrase: env::var("SOROBAN_NETWORK_PASSPHRASE")
                    .unwrap_or_else(|_| "Public Global Stellar Network ; October 2015".to_string()),
                registry_contract_id: env::var("REGISTRY_CONTRACT_ID")
                    .unwrap_or_else(|_| "CDAD...MAINNET_REGISTRY".to_string()),
                subdomain_contract_id: env::var("SUBDOMAIN_CONTRACT_ID")
                    .unwrap_or_else(|_| "CDAD...MAINNET_SUBDOMAIN".to_string()),
                bridge_contract_id: env::var("BRIDGE_CONTRACT_ID")
                    .unwrap_or_else(|_| "CDAD...MAINNET_BRIDGE".to_string()),
                auction_contract_id: env::var("AUCTION_CONTRACT_ID")
                    .unwrap_or_else(|_| "CDAD...MAINNET_AUCTION".to_string()),
            },
        }
    }
}
