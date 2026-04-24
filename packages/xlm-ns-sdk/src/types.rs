#[derive(Debug, Clone)]
pub struct RegistrationRequest {
    pub label: String,
    pub owner: String,
    pub duration_years: u32,
}

#[derive(Debug, Clone)]
pub struct RegistrationQuote {
    pub fee: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct RenewalRequest {
    pub name: String,
    pub additional_years: u32,
}

#[derive(Debug, Clone)]
pub struct RenewalResult {
    pub name: String,
    pub new_expiry: u64,
    pub fee_paid: u64,
}

#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub name: String,
    pub new_owner: String,
}

// Subdomain types
#[derive(Debug, Clone)]
pub struct RegisterParentRequest {
    pub parent: String,
    pub owner: String,
}

#[derive(Debug, Clone)]
pub struct AddControllerRequest {
    pub parent: String,
    pub controller: String,
}

#[derive(Debug, Clone)]
pub struct CreateSubdomainRequest {
    pub label: String,
    pub parent: String,
    pub owner: String,
}

#[derive(Debug, Clone)]
pub struct TransferSubdomainRequest {
    pub fqdn: String,
    pub new_owner: String,
}

#[derive(Debug, Clone)]
pub struct ParentDomain {
    pub owner: String,
    pub controllers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SubdomainRecord {
    pub parent: String,
    pub owner: String,
    pub created_at: u64,
}

// Bridge types
#[derive(Debug, Clone)]
pub struct RegisterChainRequest {
    pub chain: String,
}

#[derive(Debug, Clone)]
pub struct BuildMessageRequest {
    pub name: String,
    pub chain: String,
}

#[derive(Debug, Clone)]
pub struct BridgeRoute {
    pub destination_chain: String,
    pub destination_resolver: String,
    pub gateway: String,
}

// Contract types for RPC calls
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub owner: String,
    pub resolver: Option<String>,
    pub target_address: Option<String>,
    pub metadata_uri: Option<String>,
    pub ttl_seconds: u64,
    pub registered_at: u64,
    pub expires_at: u64,
    pub grace_period_ends_at: u64,
    pub transfer_count: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResolutionRecord {
    pub owner: String,
    pub address: String,
    pub text_records: std::collections::HashMap<String, String>,
    pub updated_at: u64,
}
