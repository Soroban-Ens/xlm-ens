#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{ResolverContract, ResolverContractClient};

    #[test]
    fn persists_forward_and_reverse_resolution_records() {
        let env = Env::default();
        let contract_id = env.register(ResolverContract, ());
        let client = ResolverContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "timmy.xlm");
        let address = String::from_str(&env, "GABC");

        client.set_record(&name, &owner, &address, &100).unwrap();
        client
            .set_text_record(
                &name,
                &owner,
                &String::from_str(&env, "com.twitter"),
                &String::from_str(&env, "@timmy"),
                &101,
            )
            .unwrap();
        client.set_primary_name(&address, &owner, &name).unwrap();

        let record = client.resolve(&name).unwrap();
        assert_eq!(record.address, address);
        assert_eq!(client.reverse(&String::from_str(&env, "GABC")), Some(name));
    }
}
