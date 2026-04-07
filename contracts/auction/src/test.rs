#[cfg(test)]
mod tests {
    use crate::bid::Bid;
    use crate::AuctionContract;

    #[test]
    fn settles_using_second_price() {
        let mut auction = AuctionContract::default();
        auction.create_auction("vip.xlm", 200, 10, 20).unwrap();
        auction
            .place_bid(
                "vip.xlm",
                Bid::new("alice", 500, 12),
                12,
            )
            .unwrap();
        auction
            .place_bid(
                "vip.xlm",
                Bid::new("bob", 300, 14),
                14,
            )
            .unwrap();

        let settlement = auction.settle("vip.xlm", 21).unwrap().unwrap();
        assert_eq!(settlement.winner, Some("alice".to_string()));
        assert_eq!(settlement.clearing_price, 300);
    }
}
