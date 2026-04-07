#[cfg(test)]
mod tests {
    use crate::SubdomainContract;

    #[test]
    fn creates_subdomain_names() {
        let mut contract = SubdomainContract::default();
        contract.register_parent("timmy.xlm", "alice").unwrap();
        contract.add_controller("timmy.xlm", "alice", "controller").unwrap();
        let fqdn = contract
            .create("pay", "timmy.xlm", "controller", "bob", 100)
            .unwrap();

        assert_eq!(fqdn, "pay.timmy.xlm");
        assert!(contract.exists("pay.timmy.xlm"));
        assert_eq!(contract.record("pay.timmy.xlm").unwrap().owner, "bob");
    }
}
