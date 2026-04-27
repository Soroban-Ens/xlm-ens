use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractErrorCode {
    NameNotFound = 1,
    NotOwner = 2,
    Expired = 3,
    InvalidLabel = 4,
    Other = 99,
}

#[derive(Debug)]
pub enum SdkError {
    InvalidRequest(String),
    Transport(String),
    ContractError(ContractErrorCode),
    ContractInvocationFailed {
        operation: &'static str,
        reason: String,
        tx_hash: Option<String>,
    },
    SimulationFailed {
        operation: &'static str,
        reason: String,
    },
    InsufficientFee {
        operation: &'static str,
        required: i64,
        available: i64,
    },
    TransactionTimeout {
        operation: &'static str,
        ledger_submitted: u32,
    },
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid request: {message}"),
            Self::Transport(message) => write!(f, "transport error: {message}"),
            Self::ContractError(code) => write!(f, "contract error: {code:?}"),
            Self::ContractInvocationFailed {
                operation,
                reason,
                tx_hash,
            } => {
                write!(f, "contract invocation failed for {operation}: {reason}")?;
                if let Some(hash) = tx_hash {
                    write!(f, " (tx: {hash})")?;
                }
                Ok(())
            }
            Self::SimulationFailed { operation, reason } => {
                write!(f, "simulation failed for {operation}: {reason}")
            }
            Self::InsufficientFee {
                operation,
                required,
                available,
            } => {
                write!(
                    f,
                    "insufficient fee for {operation}: required {required}, available {available}"
                )
            }
            Self::TransactionTimeout {
                operation,
                ledger_submitted,
            } => {
                write!(
                    f,
                    "transaction timeout for {operation} (submitted at ledger {ledger_submitted})"
                )
            }
        }
    }
}

impl std::error::Error for SdkError {}

pub fn decode_error(code: u32) -> ContractErrorCode {
    match code {
        1 => ContractErrorCode::NameNotFound,
        2 => ContractErrorCode::NotOwner,
        3 => ContractErrorCode::Expired,
        4 => ContractErrorCode::InvalidLabel,
        _ => ContractErrorCode::Other,
    }
}

