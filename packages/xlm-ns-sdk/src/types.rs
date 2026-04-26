use core::fmt;

pub const DEFAULT_FEE_CURRENCY: &str = "XLM";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationRequest {
    pub label: String,
    pub owner: String,
    pub duration_years: u32,
    pub signer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeBreakdown {
    pub base_fee: u64,
    pub premium_fee: u64,
    pub network_fee: u64,
}

impl FeeBreakdown {
    pub fn total(&self) -> u64 {
        self.base_fee
            .saturating_add(self.premium_fee)
            .saturating_add(self.network_fee)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationQuote {
    pub label: String,
    pub duration_years: u32,
    pub fee_breakdown: FeeBreakdown,
    pub total_fee: u64,
    pub fee_currency: String,
    pub expires_at: u64,
    pub quoted_at: u64,
    pub contract_id: Option<String>,
}

impl RegistrationQuote {
    /// Backwards-friendly accessor: the headline fee a caller should pay.
    pub fn fee(&self) -> u64 {
        self.total_fee
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalRequest {
    pub name: String,
    pub additional_years: u32,
    pub signer: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionStatus {
    Simulated,
    Submitted,
    Confirmed,
    Failed,
}

impl fmt::Display for SubmissionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simulated => f.write_str("simulated"),
            Self::Submitted => f.write_str("submitted"),
            Self::Confirmed => f.write_str("confirmed"),
            Self::Failed => f.write_str("failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionSubmission {
    pub tx_hash: String,
    pub status: SubmissionStatus,
    pub ledger: Option<u32>,
    pub submitted_at: u64,
    pub contract_id: Option<String>,
    pub network_passphrase: Option<String>,
    pub signer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationReceipt {
    pub name: String,
    pub owner: String,
    pub duration_years: u32,
    pub expires_at: u64,
    pub fee_paid: u64,
    pub submission: TransactionSubmission,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewalReceipt {
    pub name: String,
    pub additional_years: u32,
    pub new_expiry: u64,
    pub fee_paid: u64,
    pub submission: TransactionSubmission,
}

/// Retained for backwards compatibility with callers that only need
/// the raw renewal outcome.
pub type RenewalResult = RenewalReceipt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionResult {
    pub name: String,
    pub address: Option<String>,
    pub resolver: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReverseResolution {
    pub address: String,
    pub primary_name: Option<String>,
    pub resolver: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRecord {
    pub name: String,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRecordUpdate {
    pub name: String,
    pub key: String,
    pub value: Option<String>,
    pub signer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequest {
    pub name: String,
    pub new_owner: String,
    pub signer: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftRecord {
    pub token_id: String,
    pub owner: String,
    pub metadata_uri: Option<String>,
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

// Auction types
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuctionInfo {
    pub name: String,
    pub owner: String,
    pub reserve_price: u64,
    pub highest_bid: u64,
    pub highest_bidder: Option<String>,
    pub ends_at: u64,
    pub status: AuctionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum AuctionStatus {
    Active,
    Ended,
    Settled,
}

impl fmt::Display for AuctionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => f.write_str("active"),
            Self::Ended => f.write_str("ended"),
            Self::Settled => f.write_str("settled"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuctionCreateRequest {
    pub name: String,
    pub reserve_price: u64,
    pub duration_seconds: u64,
    pub signer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BidRequest {
    pub name: String,
    pub amount: u64,
    pub signer: Option<String>,
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
