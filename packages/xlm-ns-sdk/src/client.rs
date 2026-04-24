use crate::errors::SdkError;
use crate::types::{
    AddControllerRequest, CreateSubdomainRequest, RegisterParentRequest, RegistrationQuote, RegistrationRequest, RegistryEntry, RenewalRequest, RenewalResult, ResolutionRecord, ResolutionResult,
    TransferRequest, TransferSubdomainRequest,
};
use soroban_rpc::Client;
use soroban_sdk::{xdr::{ScVal, ScVec, ScMap, ScMapEntry, ScString, Hash}, Address, Env, String as SorobanString};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone)]
pub struct XlmNsClient {
    pub rpc_url: String,
    pub network_passphrase: Option<String>,
    pub registry_contract_id: Option<String>,
    pub subdomain_contract_id: Option<String>,
}

impl XlmNsClient {
    pub fn new(
        rpc_url: impl Into<String>,
        passphrase: Option<String>,
        registry_contract_id: Option<String>,
        subdomain_contract_id: Option<String>,
    ) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            network_passphrase: passphrase,
            registry_contract_id,
            subdomain_contract_id,
        }
    }

    pub async fn resolve(&self, name: &str) -> Result<ResolutionResult, SdkError> {
        let client = Client::new(&self.rpc_url).map_err(|e| SdkError::Transport(e.to_string()))?;
        
        let registry_id = self.registry_contract_id.as_ref()
            .ok_or_else(|| SdkError::InvalidRequest("registry contract ID not configured".into()))?;
        
        // First, query the registry for the entry
        let registry_entry = self.query_registry(&client, registry_id, name).await?;
        
        // If no resolver, return not found
        let resolver_id = registry_entry.resolver
            .ok_or_else(|| SdkError::NotFound(format!("no resolver set for {}", name)))?;
        
        // Query the resolver for the record
        let record = self.query_resolver(&client, &resolver_id, name).await?;
        
        Ok(ResolutionResult {
            name: name.to_string(),
            address: record.map(|r| r.address),
        })
    }

    async fn query_registry(&self, client: &Client, contract_id: &str, name: &str) -> Result<RegistryEntry, SdkError> {
        // For now, make a real RPC call to get network info to test transport
        let _network = client.get_network().await
            .map_err(|e| SdkError::Transport(format!("failed to get network: {}", e)))?;

        // Mock the registry entry for now
        let entry = RegistryEntry {
            name: name.to_string(),
            owner: "mock_owner".to_string(),
            resolver: Some("mock_resolver_id".to_string()),
            target_address: None,
            metadata_uri: None,
            ttl_seconds: 3600,
            registered_at: 0,
            expires_at: 2000000000,
            grace_period_ends_at: 2000003600,
            transfer_count: 0,
        };

        Ok(entry)
    }

    async fn query_resolver(&self, client: &Client, contract_id: &str, name: &str) -> Result<Option<ResolutionRecord>, SdkError> {
        // Make another RPC call to test transport
        let _network = client.get_network().await
            .map_err(|e| SdkError::Transport(format!("failed to get network: {}", e)))?;

        // Mock the record
        let record = ResolutionRecord {
            owner: "mock_owner".to_string(),
            address: "GDRA...REAL_ADDR".to_string(),
            text_records: std::collections::HashMap::new(),
            updated_at: 0,
        };

        Ok(Some(record))
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

    // Subdomain methods
    pub fn register_parent(&self, request: RegisterParentRequest) -> Result<(), SdkError> {
        if request.parent.trim().is_empty() {
            return Err(SdkError::InvalidRequest("parent must not be empty".into()));
        }
        if request.owner.trim().is_empty() {
            return Err(SdkError::InvalidRequest("owner must not be empty".into()));
        }
        // Mock implementation
        Ok(())
    }

    pub fn add_controller(&self, request: AddControllerRequest) -> Result<(), SdkError> {
        if request.parent.trim().is_empty() {
            return Err(SdkError::InvalidRequest("parent must not be empty".into()));
        }
        if request.controller.trim().is_empty() {
            return Err(SdkError::InvalidRequest("controller must not be empty".into()));
        }
        // Mock implementation
        Ok(())
    }

    pub fn create_subdomain(&self, request: CreateSubdomainRequest) -> Result<String, SdkError> {
        if request.label.trim().is_empty() {
            return Err(SdkError::InvalidRequest("label must not be empty".into()));
        }
        if request.parent.trim().is_empty() {
            return Err(SdkError::InvalidRequest("parent must not be empty".into()));
        }
        if request.owner.trim().is_empty() {
            return Err(SdkError::InvalidRequest("owner must not be empty".into()));
        }
        // Mock implementation
        Ok(format!("{}.{}", request.label, request.parent))
    }

    pub fn transfer_subdomain(&self, request: TransferSubdomainRequest) -> Result<(), SdkError> {
        if request.fqdn.trim().is_empty() {
            return Err(SdkError::InvalidRequest("fqdn must not be empty".into()));
        }
        if request.new_owner.trim().is_empty() {
            return Err(SdkError::InvalidRequest("new_owner must not be empty".into()));
        }
        // Mock implementation
        Ok(())
    }
}
