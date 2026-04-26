use core::fmt;

#[derive(Debug)]
pub enum SdkError {
    InvalidRequest(String),
    Transport(String),
    NotFound(String),
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid request: {message}"),
            Self::Transport(message) => write!(f, "transport error: {message}"),
            Self::NotFound(message) => write!(f, "not found: {message}"),
        }
    }
}

impl std::error::Error for SdkError {}
