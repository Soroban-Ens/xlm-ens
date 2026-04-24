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
