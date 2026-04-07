pub mod bid;
pub mod settle;
pub mod test;

use std::collections::HashMap;

use bid::Bid;
use settle::Settlement;

#[derive(Debug, Default)]
pub struct AuctionContract {
    auctions: HashMap<String, Vec<Bid>>,
}

impl AuctionContract {
    pub fn place_bid(&mut self, name: impl Into<String>, bid: Bid) {
        self.auctions.entry(name.into()).or_default().push(bid);
    }

    pub fn settle(&self, name: &str) -> Option<Settlement> {
        self.auctions.get(name).and_then(|bids| settle::settle_vickrey(bids))
    }
}
