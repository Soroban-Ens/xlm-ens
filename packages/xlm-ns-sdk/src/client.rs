use crate::errors::SdkError;
use crate::types::{
    AddControllerRequest, BridgeRoute, BuildMessageRequest, FeeBreakdown, NftRecord,
    RegisterChainRequest, RegisterParentRequest, RegistrationQuote, RegistrationReceipt,
    RegistrationRequest, RenewalReceipt, RenewalRequest, ResolutionResult, ReverseResolution,
    SubmissionStatus, TextRecord, TextRecordUpdate, TransactionSubmission, TransferRequest,
    TransferSubdomainRequest, DEFAULT_FEE_CURRENCY,
};

const MOCK_REFERENCE_TIMESTAMP: u64 = 1_682_200_000;
const SECONDS_PER_YEAR: u64 = 31_536_000;
const BASE_FEE_PER_YEAR: u64 = 10;
const PREMIUM_FEE: u64 = 0;
const NETWORK_FEE: u64 = 1;

#[derive(Debug, Clone)]
pub struct XlmNsClient {
    pub rpc_url: String,
    pub network_passphrase: Option<String>,
    pub registry_contract_id: Option<String>,
    pub registrar_contract_id: Option<String>,
    pub resolver_contract_id: Option<String>,
    pub auction_contract_id: Option<String>,
    pub bridge_contract_id: Option<String>,
    pub subdomain_contract_id: Option<String>,
    pub nft_contract_id: Option<String>,
}

impl XlmNsClient {
    pub fn new(
        rpc_url: impl Into<String>,
        passphrase: Option<String>,
        registry_contract_id: Option<String>,
        subdomain_contract_id: Option<String>,
        bridge_contract_id: Option<String>,
    ) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            network_passphrase: passphrase,
            registry_contract_id,
            registrar_contract_id: None,
            resolver_contract_id: None,
            auction_contract_id: None,
            bridge_contract_id,
            subdomain_contract_id,
            nft_contract_id: None,
        }
    }

    pub fn with_registrar(mut self, registrar_contract_id: impl Into<String>) -> Self {
        self.registrar_contract_id = Some(registrar_contract_id.into());
        self
    }

    pub fn with_resolver(mut self, resolver_contract_id: impl Into<String>) -> Self {
        self.resolver_contract_id = Some(resolver_contract_id.into());
        self
    }

    pub fn with_auction(mut self, auction_contract_id: impl Into<String>) -> Self {
        self.auction_contract_id = Some(auction_contract_id.into());
        self
    }

    pub fn with_nft(mut self, nft_contract_id: impl Into<String>) -> Self {
        self.nft_contract_id = Some(nft_contract_id.into());
        self
    }

    pub fn resolve(&self, name: &str) -> Result<ResolutionResult, SdkError> {
        if name.trim().is_empty() {
            return Err(SdkError::InvalidRequest("name must not be empty".into()));
        }

        Ok(ResolutionResult {
            name: name.to_string(),
            address: Some("GDRA...MOCK_ADDR".to_string()),
            resolver: self.resolver_contract_id.clone(),
            expires_at: Some(MOCK_REFERENCE_TIMESTAMP + SECONDS_PER_YEAR),
        })
    }

    pub fn get_registration(&self, name: &str) -> Result<Option<ResolutionResult>, SdkError> {
        if name.trim().is_empty() {
            return Err(SdkError::InvalidRequest("name must not be empty".into()));
        }

        if name == "notfound.xlm" {
            return Ok(None);
        }

        Ok(Some(ResolutionResult {
            name: name.to_string(),
            address: Some("GDRA...OWNER_ADDR".to_string()),
            resolver: self.resolver_contract_id.clone(),
            expires_at: Some(MOCK_REFERENCE_TIMESTAMP + SECONDS_PER_YEAR),
        }))
    }

    pub fn reverse_resolve(&self, address: &str) -> Result<ReverseResolution, SdkError> {
        if address.trim().is_empty() {
            return Err(SdkError::InvalidRequest("address must not be empty".into()));
        }

        Ok(ReverseResolution {
            address: address.to_string(),
            primary_name: Some("reverse.xlm".to_string()),
            resolver: self.resolver_contract_id.clone(),
        })
    }

    pub fn get_text_record(&self, name: &str, key: &str) -> Result<TextRecord, SdkError> {
        if name.trim().is_empty() {
            return Err(SdkError::InvalidRequest("name must not be empty".into()));
        }
        if key.trim().is_empty() {
            return Err(SdkError::InvalidRequest("key must not be empty".into()));
        }

        Ok(TextRecord {
            name: name.to_string(),
            key: key.to_string(),
            value: Some(format!("mock:{key}")),
        })
    }

    pub fn set_text_record(&self, update: TextRecordUpdate) -> Result<TransactionSubmission, SdkError> {
        if update.name.trim().is_empty() {
            return Err(SdkError::InvalidRequest("name must not be empty".into()));
        }
        if update.key.trim().is_empty() {
            return Err(SdkError::InvalidRequest("key must not be empty".into()));
        }

        Ok(TransactionSubmission {
            tx_hash: "tx_text_record_mock".to_string(),
            status: SubmissionStatus::Submitted,
            ledger: None,
            submitted_at: MOCK_REFERENCE_TIMESTAMP,
            contract_id: self.resolver_contract_id.clone(),
            network_passphrase: self.network_passphrase.clone(),
            signer: update.signer,
        })
    }

    pub fn quote_registration(
        &self,
        label: &str,
        duration_years: u32,
    ) -> Result<RegistrationQuote, SdkError> {
        if label.trim().is_empty() {
            return Err(SdkError::InvalidRequest("label must not be empty".into()));
        }
        if duration_years == 0 {
            return Err(SdkError::InvalidRequest(
                "duration_years must be greater than zero".into(),
            ));
        }

        let years = u64::from(duration_years);
        let fee_breakdown = FeeBreakdown {
            base_fee: BASE_FEE_PER_YEAR.saturating_mul(years),
            premium_fee: PREMIUM_FEE,
            network_fee: NETWORK_FEE,
        };
        let total_fee = fee_breakdown.total();
        let expires_at = MOCK_REFERENCE_TIMESTAMP + years * SECONDS_PER_YEAR;

        Ok(RegistrationQuote {
            label: label.to_string(),
            duration_years,
            fee_breakdown,
            total_fee,
            fee_currency: DEFAULT_FEE_CURRENCY.to_string(),
            expires_at,
            quoted_at: MOCK_REFERENCE_TIMESTAMP,
            contract_id: self.registrar_contract_id.clone(),
        })
    }

    pub fn register(&self, request: RegistrationRequest) -> Result<RegistrationReceipt, SdkError> {
        if request.label.trim().is_empty() {
            return Err(SdkError::InvalidRequest("label must not be empty".into()));
        }
        if request.owner.trim().is_empty() {
            return Err(SdkError::InvalidRequest("owner must not be empty".into()));
        }
        if request.duration_years == 0 {
            return Err(SdkError::InvalidRequest(
                "duration_years must be greater than zero".into(),
            ));
        }

        let quote = self.quote_registration(&request.label, request.duration_years)?;
        let submission = TransactionSubmission {
            tx_hash: "tx_abc123789xyz".to_string(),
            status: SubmissionStatus::Submitted,
            ledger: None,
            submitted_at: MOCK_REFERENCE_TIMESTAMP,
            contract_id: self.registrar_contract_id.clone(),
            network_passphrase: self.network_passphrase.clone(),
            signer: request.signer.clone(),
        };

        Ok(RegistrationReceipt {
            name: format!("{}.xlm", request.label),
            owner: request.owner,
            duration_years: request.duration_years,
            expires_at: quote.expires_at,
            fee_paid: quote.total_fee,
            submission,
        })
    }

    pub fn renew(&self, request: RenewalRequest) -> Result<RenewalReceipt, SdkError> {
        if request.name.trim().is_empty() {
            return Err(SdkError::InvalidRequest("name must not be empty".into()));
        }
        if request.additional_years == 0 {
            return Err(SdkError::InvalidRequest(
                "additional_years must be greater than zero".into(),
            ));
        }

        let years = u64::from(request.additional_years);
        let fee_paid = BASE_FEE_PER_YEAR
            .saturating_mul(years)
            .saturating_add(NETWORK_FEE);
        let new_expiry = MOCK_REFERENCE_TIMESTAMP + years * SECONDS_PER_YEAR;
        let submission = TransactionSubmission {
            tx_hash: "tx_renew_mock".to_string(),
            status: SubmissionStatus::Submitted,
            ledger: None,
            submitted_at: MOCK_REFERENCE_TIMESTAMP,
            contract_id: self.registrar_contract_id.clone(),
            network_passphrase: self.network_passphrase.clone(),
            signer: request.signer.clone(),
        };

        Ok(RenewalReceipt {
            name: request.name,
            additional_years: request.additional_years,
            new_expiry,
            fee_paid,
            submission,
        })
    }

    pub fn transfer(&self, request: TransferRequest) -> Result<TransactionSubmission, SdkError> {
        if request.name.trim().is_empty() {
            return Err(SdkError::InvalidRequest("name must not be empty".into()));
        }
        if request.new_owner.trim().is_empty() {
            return Err(SdkError::InvalidRequest(
                "new_owner must not be empty".into(),
            ));
        }

        Ok(TransactionSubmission {
            tx_hash: "tx_transfer_mock".to_string(),
            status: SubmissionStatus::Submitted,
            ledger: None,
            submitted_at: MOCK_REFERENCE_TIMESTAMP,
            contract_id: self.registry_contract_id.clone(),
            network_passphrase: self.network_passphrase.clone(),
            signer: request.signer,
        })
    }

    pub fn register_parent(&self, request: RegisterParentRequest) -> Result<(), SdkError> {
        if request.parent.trim().is_empty() {
            return Err(SdkError::InvalidRequest("parent must not be empty".into()));
        }
        if request.owner.trim().is_empty() {
            return Err(SdkError::InvalidRequest("owner must not be empty".into()));
        }

        Ok(())
    }

    pub fn add_controller(&self, request: AddControllerRequest) -> Result<(), SdkError> {
        if request.parent.trim().is_empty() {
            return Err(SdkError::InvalidRequest("parent must not be empty".into()));
        }
        if request.controller.trim().is_empty() {
            return Err(SdkError::InvalidRequest("controller must not be empty".into()));
        }

        Ok(())
    }

    pub fn create_subdomain(
        &self,
        request: crate::types::CreateSubdomainRequest,
    ) -> Result<String, SdkError> {
        if request.label.trim().is_empty() {
            return Err(SdkError::InvalidRequest("label must not be empty".into()));
        }
        if request.parent.trim().is_empty() {
            return Err(SdkError::InvalidRequest("parent must not be empty".into()));
        }
        if request.owner.trim().is_empty() {
            return Err(SdkError::InvalidRequest("owner must not be empty".into()));
        }

        Ok(format!("{}.{}", request.label, request.parent))
    }

    pub fn transfer_subdomain(&self, request: TransferSubdomainRequest) -> Result<(), SdkError> {
        if request.fqdn.trim().is_empty() {
            return Err(SdkError::InvalidRequest("fqdn must not be empty".into()));
        }
        if request.new_owner.trim().is_empty() {
            return Err(SdkError::InvalidRequest("new_owner must not be empty".into()));
        }

        Ok(())
    }

    pub fn register_chain(&self, request: RegisterChainRequest) -> Result<(), SdkError> {
        if request.chain.trim().is_empty() {
            return Err(SdkError::InvalidRequest("chain must not be empty".into()));
        }

        match request.chain.as_str() {
            "base" | "ethereum" | "arbitrum" => Ok(()),
            _ => Err(SdkError::InvalidRequest(format!(
                "unsupported chain: {}",
                request.chain
            ))),
        }
    }

    pub fn get_route(&self, chain: &str) -> Result<Option<BridgeRoute>, SdkError> {
        if chain.trim().is_empty() {
            return Err(SdkError::InvalidRequest("chain must not be empty".into()));
        }

        let route = match chain {
            "base" => Some(BridgeRoute {
                destination_chain: "base".to_string(),
                destination_resolver: "0xbaseResolver".to_string(),
                gateway: "0xbaseGateway".to_string(),
            }),
            "ethereum" => Some(BridgeRoute {
                destination_chain: "ethereum".to_string(),
                destination_resolver: "0xethResolver".to_string(),
                gateway: "0xethGateway".to_string(),
            }),
            "arbitrum" => Some(BridgeRoute {
                destination_chain: "arbitrum".to_string(),
                destination_resolver: "0xarbResolver".to_string(),
                gateway: "0xarbGateway".to_string(),
            }),
            _ => None,
        };

        Ok(route)
    }

    pub fn build_message(&self, request: BuildMessageRequest) -> Result<String, SdkError> {
        if request.name.trim().is_empty() {
            return Err(SdkError::InvalidRequest("name must not be empty".into()));
        }
        if request.chain.trim().is_empty() {
            return Err(SdkError::InvalidRequest("chain must not be empty".into()));
        }
        if self.get_route(&request.chain)?.is_none() {
            return Err(SdkError::InvalidRequest(format!(
                "unsupported chain: {}",
                request.chain
            )));
        }

        let resolver = match request.chain.as_str() {
            "base" => "0xbaseResolver",
            "ethereum" => "0xethResolver",
            "arbitrum" => "0xarbResolver",
            _ => unreachable!(),
        };

        Ok(format!(
            "{{\"type\":\"xlm-ns-resolution\",\"name\":\"{}\",\"destination_chain\":\"{}\",\"resolver\":\"{}\"}}",
            request.name, request.chain, resolver
        ))
    }

    pub fn get_nft_record(&self, token_id: &str) -> Result<NftRecord, SdkError> {
        if token_id.trim().is_empty() {
            return Err(SdkError::InvalidRequest("token_id must not be empty".into()));
        }

        Ok(NftRecord {
            token_id: token_id.to_string(),
            owner: "GDRA...NFT_OWNER".to_string(),
            metadata_uri: Some(format!("ipfs://mock-metadata/{token_id}")),
        })
    }
}
