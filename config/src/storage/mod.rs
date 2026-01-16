// Configuration storage abstraction
// Provides trait for reading/writing configuration from various storage backends

pub mod efi;

use crate::error::ConfigError;
use crate::toml::Value;

/// Trait for configuration storage backends
pub trait ConfigStorage {
    /// Load configuration from storage
    /// Returns Ok(None) if no configuration exists
    fn load(&self) -> Result<Option<Value>, ConfigError>;

    /// Save configuration to storage
    fn save(&mut self, config: &Value) -> Result<(), ConfigError>;

    /// Check if configuration exists in storage
    fn exists(&self) -> bool;
}
