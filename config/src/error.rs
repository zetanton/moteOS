
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
    StorageError(String),
    SerializationError(String),
    DeserializationError(String),
    EfiError(String),
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

    pub fn storage_error(msg: &str) -> Self {
        ConfigError::StorageError(String::from(msg))
    }

    pub fn serialization_error(msg: &str) -> Self {
        ConfigError::SerializationError(String::from(msg))
    }

    pub fn deserialization_error(msg: &str) -> Self {
        ConfigError::DeserializationError(String::from(msg))
    }

    pub fn efi_error(msg: &str) -> Self {
        ConfigError::EfiError(String::from(msg))
    }
}
