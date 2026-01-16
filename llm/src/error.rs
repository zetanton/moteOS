extern crate alloc;

use alloc::string::String;
use core::fmt;

/// Errors that can occur when interacting with LLM providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmError {
    /// Network-related error (connection failed, timeout, etc.)
    NetworkError(String),
    /// HTTP error with status code and response body.
    HttpError {
        status: u16,
        body: String,
    },
    /// Authentication error (invalid API key, etc.)
    AuthError(String),
    /// Rate limit error with optional retry-after seconds.
    RateLimitError {
        retry_after: Option<u64>,
    },
    /// Invalid model identifier.
    InvalidModel(String),
    /// Error parsing response or request data.
    ParseError(String),
    /// Request timed out.
    Timeout,
    /// Other error with description.
    Other(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            LlmError::HttpError { status, body } => {
                write!(f, "HTTP error {}: {}", status, body)
            }
            LlmError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            LlmError::RateLimitError { retry_after } => {
                if let Some(seconds) = retry_after {
                    write!(f, "Rate limit exceeded. Retry after {} seconds", seconds)
                } else {
                    write!(f, "Rate limit exceeded")
                }
            }
            LlmError::InvalidModel(model) => write!(f, "Invalid model: {}", model),
            LlmError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LlmError::Timeout => write!(f, "Request timed out"),
            LlmError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}
