use crate::errors::SdkError;
use crate::types::{
    RegistrationQuote, RegistrationRequest, RenewalRequest, RenewalResult, ResolutionResult,
    TransferRequest,
};

#[derive(Debug, Clone)]
pub struct XlmNsClient {
    pub rpc_url: String,
    pub network_passphrase: Option<String>,
    pub registry_contract_id: Option<String>,
}

impl XlmNsClient {
    pub fn new(
        rpc_url: impl Into<String>,
        passphrase: Option<String>,
        contract_id: Option<String>,
    ) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            network_passphrase: passphrase,
            registry_contract_id: contract_id,
        }
    }

    pub fn resolve(&self, name: &str) -> Result<ResolutionResult, SdkError> {
        // Mock resolution: return a dummy address for any name
        Ok(ResolutionResult {
            name: name.to_string(),
            address: Some("GDRA...MOCK_ADDR".to_string()),
        })
    }

    pub fn get_registration(&self, name: &str) -> Result<Option<ResolutionResult>, SdkError> {
        // Mock fetching registration
        if name == "notfound.xlm" {
            Ok(None)
        } else {
            Ok(Some(ResolutionResult {
                name: name.to_string(),
                address: Some("GDRA...OWNER_ADDR".to_string()),
            }))
        }
    }

    pub fn quote_registration(
        &self,
        label: &str,
        duration_years: u32,
    ) -> Result<RegistrationQuote, SdkError> {
        if label.trim().is_empty() {
            return Err(SdkError::InvalidRequest("label must not be empty".into()));
        }

        // Mock logic: 10 XLM per year
        let fee = (duration_years as u64) * 10;
        let expires_at = 1682200000 + (duration_years as u64 * 31536000);

        Ok(RegistrationQuote { fee, expires_at })
    }

    pub fn register(&self, request: RegistrationRequest) -> Result<String, SdkError> {
        if request.label.trim().is_empty() {
            return Err(SdkError::InvalidRequest("label must not be empty".into()));
        }

        // Mock successful transaction hash
        Ok("tx_abc123789xyz".to_string())
    }

    pub fn renew(&self, request: RenewalRequest) -> Result<RenewalResult, SdkError> {
        if request.additional_years == 0 {
            return Err(SdkError::InvalidRequest(
                "additional_years must be greater than zero".into(),
            ));
        }

        // Mock renewal logic
        let fee_paid = (request.additional_years as u64) * 10;
        let new_expiry = 1682200000 + (request.additional_years as u64 * 31536000);

        Ok(RenewalResult {
            name: request.name,
            new_expiry,
            fee_paid,
        })
    }

    pub fn transfer(&self, request: TransferRequest) -> Result<(), SdkError> {
        if request.new_owner.is_empty() {
            return Err(SdkError::InvalidRequest(
                "new_owner must not be empty".into(),
            ));
        }

        // Mock transfer logic
        Ok(())
    }
}
