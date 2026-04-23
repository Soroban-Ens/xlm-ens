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
