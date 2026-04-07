use crate::config::Network;
use xlm_ns_auction::bid::Bid;
use xlm_ns_auction::AuctionContract;

pub fn run_auction(network: Network, name: &str, reserve: u64) {
    let mut auction = AuctionContract::default();
    let bidder = match network {
        Network::Testnet => "testnet-bidder",
        Network::Mainnet => "mainnet-bidder",
    };

    auction.create_auction(name, reserve, 0, 100).expect("auction creation should succeed");
    auction
        .place_bid(name, Bid::new(bidder, reserve, 1), 1)
        .expect("bid placement should succeed");

    println!("created auction placeholder for {name} with reserve {reserve}");
}
