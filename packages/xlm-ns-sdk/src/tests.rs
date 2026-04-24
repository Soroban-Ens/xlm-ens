#[cfg(test)]
mod tests {
    use crate::client::XlmNsClient;
    use crate::types::{
        RegistrationRequest, RenewalRequest, SubmissionStatus, TextRecordUpdate, TransferRequest,
    };

    fn client() -> XlmNsClient {
        XlmNsClient::new(
            "http://localhost",
            Some("Test SDF Network ; September 2015".into()),
            Some("CDAD...REGISTRY".into()),
        )
        .with_resolver("CDAD...RESOLVER")
    }

    #[test]
    fn renewal_returns_rich_receipt() {
        let receipt = client()
            .renew(RenewalRequest {
                name: "test.xlm".into(),
                additional_years: 2,
                signer: Some("alice".into()),
            })
            .unwrap();

        assert_eq!(receipt.fee_paid, 21);
        assert_eq!(receipt.additional_years, 2);
        assert_eq!(receipt.submission.status, SubmissionStatus::Submitted);
        assert_eq!(receipt.submission.signer.as_deref(), Some("alice"));
        assert!(receipt.new_expiry > 1_682_200_000);
    }

    #[test]
    fn registration_quote_exposes_breakdown() {
        let quote = client().quote_registration("alpha", 3).unwrap();
        assert_eq!(quote.label, "alpha");
        assert_eq!(quote.duration_years, 3);
        assert_eq!(quote.fee_breakdown.base_fee, 30);
        assert_eq!(quote.fee_breakdown.network_fee, 1);
        assert_eq!(quote.total_fee, 31);
        assert_eq!(quote.fee_currency, "XLM");
        assert!(quote.contract_id.is_some());
    }

    #[test]
    fn registration_receipt_carries_submission_metadata() {
        let receipt = client()
            .register(RegistrationRequest {
                label: "beta".into(),
                owner: "GDRA...OWNER".into(),
                duration_years: 1,
                signer: Some("treasury".into()),
            })
            .unwrap();

        assert_eq!(receipt.name, "beta.xlm");
        assert_eq!(receipt.duration_years, 1);
        assert_eq!(receipt.fee_paid, 11);
        assert_eq!(receipt.submission.signer.as_deref(), Some("treasury"));
        assert!(receipt.submission.network_passphrase.is_some());
    }

    #[test]
    fn reverse_resolution_rejects_empty_address() {
        assert!(client().reverse_resolve("").is_err());
    }

    #[test]
    fn text_record_round_trip() {
        let client = client();
        let record = client.get_text_record("foo.xlm", "url").unwrap();
        assert_eq!(record.name, "foo.xlm");
        assert_eq!(record.key, "url");

        let submission = client
            .set_text_record(TextRecordUpdate {
                name: "foo.xlm".into(),
                key: "url".into(),
                value: Some("https://example.xyz".into()),
                signer: Some("owner".into()),
            })
            .unwrap();
        assert_eq!(submission.status, SubmissionStatus::Submitted);
        assert_eq!(submission.signer.as_deref(), Some("owner"));
    }

    #[test]
    fn transfer_returns_submission() {
        let submission = client()
            .transfer(TransferRequest {
                name: "foo.xlm".into(),
                new_owner: "GDRA...NEW".into(),
                signer: Some("alice".into()),
            })
            .unwrap();
        assert_eq!(submission.status, SubmissionStatus::Submitted);
        assert_eq!(submission.signer.as_deref(), Some("alice"));
    }
}
