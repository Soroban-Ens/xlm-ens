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

        client.create_auction(&name, &200, &10, &20).unwrap();
        client.place_bid(&name, &alice, &500, &12).unwrap();
        client.place_bid(&name, &bob, &300, &13).unwrap();

        let settlement = client.settle(&name, &21).unwrap().unwrap();
        assert_eq!(settlement.winner, Some(alice));
        assert_eq!(settlement.clearing_price, 300);
    }
}
