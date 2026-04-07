#[cfg(test)]
mod tests {
    use crate::SubdomainContract;

    #[test]
    fn creates_subdomain_names() {
        let mut contract = SubdomainContract::default();
        assert!(contract.create("pay", "timmy.xlm"));
        assert!(contract.exists("pay.timmy.xlm"));
    }
}
