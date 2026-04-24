#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{AuctionContract, AuctionContractClient};

    #[test]
    fn stores_auctions_in_contract_storage() {
        let env = Env::default();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let name = String::from_str(&env, "vip.xlm");

        client.create_auction(&name, &200, &10, &20);
        client.place_bid(&name, &alice, &500, &12);
        client.place_bid(&name, &bob, &300, &13);

        let settlement = client.settle(&name, &21).unwrap();
        assert_eq!(settlement.winner, Some(alice));
        assert_eq!(settlement.clearing_price, 300);
        assert!(settlement.sold);
    }

    #[test]
    fn test_auction_no_bids() {
        let env = Env::default();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let name = String::from_str(&env, "ghost.xlm");
        client.create_auction(&name, &100, &10, &20);

        let settlement = client.settle(&name, &21);
        assert!(settlement.is_none());
    }

    #[test]
    fn test_auction_reserve_not_met() {
        let env = Env::default();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let alice = Address::generate(&env);
        let name = String::from_str(&env, "cheap.xlm");
        client.create_auction(&name, &1000, &10, &20);
        client.place_bid(&name, &alice, &500, &15);

        let settlement = client.settle(&name, &21).unwrap();
        assert_eq!(settlement.winner, None);
        assert_eq!(settlement.clearing_price, 0);
        assert!(!settlement.sold);
    }

    #[test]
    fn test_auction_tie_behavior() {
        let env = Env::default();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let name = String::from_str(&env, "tie.xlm");
        client.create_auction(&name, &100, &10, &20);

        client.place_bid(&name, &alice, &500, &12);
        client.place_bid(&name, &bob, &500, &13);

        let settlement = client.settle(&name, &21).unwrap();
        // First bidder wins in case of tie in current implementation
        assert_eq!(settlement.winner, Some(alice));
        assert_eq!(settlement.clearing_price, 500);
        assert!(settlement.sold);
    }

    #[test]
    fn test_auction_clearing_price_logic() {
        let env = Env::default();
        let contract_id = env.register(AuctionContract, ());
        let client = AuctionContractClient::new(&env, &contract_id);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let charlie = Address::generate(&env);
        let name = String::from_str(&env, "multi.xlm");
        client.create_auction(&name, &100, &10, &20);

        client.place_bid(&name, &alice, &1000, &12);
        client.place_bid(&name, &bob, &500, &13);
        client.place_bid(&name, &charlie, &750, &14);

        let settlement = client.settle(&name, &21).unwrap();
        assert_eq!(settlement.winner, Some(alice));
        assert_eq!(settlement.clearing_price, 750); // Second highest bid
        assert!(settlement.sold);
    }
}
////
////