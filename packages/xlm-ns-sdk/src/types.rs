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
