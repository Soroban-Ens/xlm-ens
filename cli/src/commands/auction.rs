use crate::config::Network;

pub fn run_auction(network: Network, name: &str, reserve: u64) {
    let environment = match network {
        Network::Testnet => "testnet",
        Network::Mainnet => "mainnet",
    };

    println!("prepare auction for {name} on {environment} with reserve {reserve}");
}
