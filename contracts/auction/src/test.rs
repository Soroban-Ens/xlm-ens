#[cfg(test)]
mod tests {
    use crate::bid::Bid;
    use crate::AuctionContract;

    #[test]
    fn settles_using_second_price() {
        let mut auction = AuctionContract::default();
        auction.place_bid(
            "vip.xlm",
            Bid {
                bidder: "alice".into(),
                amount: 500,
            },
        );
        auction.place_bid(
            "vip.xlm",
            Bid {
                bidder: "bob".into(),
                amount: 300,
            },
        );

        let settlement = auction.settle("vip.xlm").unwrap();
        assert_eq!(settlement.winner, "alice");
        assert_eq!(settlement.clearing_price, 300);
    }
}
