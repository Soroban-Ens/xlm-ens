#[cfg(test)]
mod tests {
    use crate::client::XlmNsClient;
    use crate::types::{
        RegistrationRequest, RenewalRequest, SubmissionStatus, TextRecordUpdate, TransferRequest,
    };

    fn client() -> XlmNsClient {
        XlmNsClient::builder("http://localhost")
            .network_passphrase("Test SDF Network ; September 2015")
            .registry("CDAD...REGISTRY")
            .subdomain("CDAD...SUBDOMAIN")
            .bridge("CDAD...BRIDGE")
            .auction("CDAD...AUCTION")
            .registrar("CDAD...REGISTRAR")
            .resolver("CDAD...RESOLVER")
            .build()
    }

    #[tokio::test]
    async fn renewal_returns_rich_receipt() {
        let receipt = client()
            .renew(RenewalRequest {
                name: "test.xlm".into(),
                additional_years: 2,
                signer: Some("alice".into()),
            })
            .await
            .unwrap();

        assert_eq!(receipt.fee_paid, 21);
        assert_eq!(receipt.additional_years, 2);
        assert_eq!(receipt.submission.status, SubmissionStatus::Submitted);
        assert_eq!(receipt.submission.signer.as_deref(), Some("alice"));
        assert!(receipt.new_expiry > 1_682_200_000);
    }

    #[tokio::test]
    async fn registration_quote_exposes_breakdown() {
        let quote = client().quote_registration("alpha", 3).await.unwrap();
        assert_eq!(quote.label, "alpha");
        assert_eq!(quote.duration_years, 3);
        assert_eq!(quote.fee_breakdown.base_fee, 30);
        assert_eq!(quote.fee_breakdown.network_fee, 1);
        assert_eq!(quote.total_fee, 31);
        assert_eq!(quote.fee_currency, "XLM");
        assert!(quote.contract_id.is_some());
    }

    #[tokio::test]
    async fn registration_receipt_carries_submission_metadata() {
        let receipt = client()
            .register(RegistrationRequest {
                label: "beta".into(),
                owner: "GDRA...OWNER".into(),
                duration_years: 1,
                signer: Some("treasury".into()),
            })
            .await
            .unwrap();

        assert_eq!(receipt.name, "beta.xlm");
        assert_eq!(receipt.duration_years, 1);
        assert_eq!(receipt.fee_paid, 11);
        assert_eq!(receipt.submission.signer.as_deref(), Some("treasury"));
        assert!(receipt.submission.network_passphrase.is_some());
    }

    #[tokio::test]
    async fn reverse_resolution_rejects_empty_address() {
        assert!(client().reverse_resolve("").await.is_err());
    }

    #[tokio::test]
    async fn text_record_round_trip() {
        let client = client();
        let record = client.get_text_record("foo.xlm", "url").await.unwrap();
        assert_eq!(record.name, "foo.xlm");
        assert_eq!(record.key, "url");

        let submission = client
            .set_text_record(TextRecordUpdate {
                name: "foo.xlm".into(),
                key: "url".into(),
                value: Some("https://example.xyz".into()),
                signer: Some("owner".into()),
            })
            .await
            .unwrap();
        assert_eq!(submission.status, SubmissionStatus::Submitted);
        assert_eq!(submission.signer.as_deref(), Some("owner"));
    }

    #[tokio::test]
    async fn transfer_returns_submission() {
        let submission = client()
            .transfer(TransferRequest {
                name: "foo.xlm".into(),
                new_owner: "GDRA...NEW".into(),
                signer: Some("alice".into()),
            })
            .await
            .unwrap();
        assert_eq!(submission.status, SubmissionStatus::Submitted);
        assert_eq!(submission.signer.as_deref(), Some("alice"));
    }

    #[tokio::test]
    async fn builder_default_config_is_applied() {
        let client = client();
        assert_eq!(client.config.timeout, crate::config::DEFAULT_TIMEOUT);
        assert!(client.config.user_agent.starts_with("xlm-ns-sdk/"));
    }

    #[tokio::test]
    async fn builder_accepts_custom_config() {
        use crate::config::ClientConfig;
        use std::time::Duration;

        let client = XlmNsClient::builder("http://localhost")
            .registry("CDAD...REGISTRY")
            .config(
                ClientConfig::default()
                    .with_timeout(Duration::from_secs(2))
                    .with_max_retries(0)
                    .with_user_agent("integration-test/1.0"),
            )
            .build();

        assert_eq!(client.config.timeout, Duration::from_secs(2));
        assert_eq!(client.config.retry.max_retries, 0);
        assert_eq!(client.config.user_agent, "integration-test/1.0");
    }
}
