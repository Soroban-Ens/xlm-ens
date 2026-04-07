#[cfg(test)]
mod tests {
    use crate::ResolverContract;

    #[test]
    fn stores_forward_and_reverse_records() {
        let mut resolver = ResolverContract::default();
        resolver.set_record("timmy.xlm", "alice", "GABC", 100).unwrap();
        resolver
            .set_text_record("timmy.xlm", "alice", "com.twitter", "@timmy", 101)
            .unwrap();
        resolver.set_primary_name("GABC", "alice", "timmy.xlm").unwrap();

        let resolved = resolver.resolve("timmy.xlm").unwrap();
        assert_eq!(resolved.address, "GABC");
        assert_eq!(resolved.text_records.get("com.twitter"), Some(&"@timmy".to_string()));
        assert_eq!(resolver.reverse("GABC"), Some("timmy.xlm"));
        assert!(resolver.has_forward_record("timmy.xlm"));
        assert!(resolver.has_reverse_record("GABC"));
    }
}
