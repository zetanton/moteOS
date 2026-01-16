
use alloc::string::String;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    ParseError(String),
    InvalidValue(String),
    MissingKey(String),
    InvalidTablePath(String),
    InvalidArray(String),
    UnexpectedEof,
    InvalidNumber(String),
    InvalidString(String),
    InvalidBoolean(String),
}

impl ConfigError {
    pub fn parse_error(msg: &str) -> Self {
        ConfigError::ParseError(String::from(msg))
    }

    pub fn invalid_value(msg: &str) -> Self {
        ConfigError::InvalidValue(String::from(msg))
    }

    pub fn missing_key(key: &str) -> Self {
        ConfigError::MissingKey(String::from(key))
    }
}
