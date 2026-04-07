#[cfg(test)]
mod tests {
    use crate::NftContract;

    #[test]
    fn mints_and_transfers_tokens() {
        let mut nft = NftContract::default();
        assert!(nft.mint("timmy.xlm", "alice"));
        assert!(nft.transfer("timmy.xlm", "bob"));
        assert_eq!(nft.owner_of("timmy.xlm"), Some("bob"));
    }
}
