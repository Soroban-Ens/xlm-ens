#[cfg(test)]
mod tests {
    use crate::client::XlmNsClient;
    use crate::types::RenewalRequest;

    #[test]
    fn test_renewal_mock() {
        let client = XlmNsClient::new("http://localhost", None, None);
        let result = client
            .renew(RenewalRequest {
                name: "test.xlm".into(),
                additional_years: 2,
            })
            .unwrap();

        assert_eq!(result.fee_paid, 20);
        assert!(result.new_expiry > 1682200000);
    }
}
