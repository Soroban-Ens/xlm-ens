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
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid request: {message}"),
            Self::Transport(message) => write!(f, "transport error: {message}"),
            Self::ContractError(code) => write!(f, "contract error: {code:?}"),
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

