#![no_std]

extern crate alloc;

pub mod toml;
pub mod error;
pub mod storage;
pub mod types;
pub mod wizard;
pub mod crypto;

pub use toml::{TomlParser, Value};
pub use error::ConfigError;
pub use storage::{ConfigStorage, efi::EfiConfigStorage};
pub use types::{
    MoteConfig, NetworkConfig, ConnectionType, IpConfig,
    ProviderConfigs, ProviderConfig, LocalProviderConfig,
    Preferences, ThemeChoice, WifiNetwork, SecurityType,
};
pub use wizard::{SetupWizard, WizardState, WizardEvent, Key, ApiKeyProvider};
pub use crypto::{encrypt_api_key, decrypt_api_key};
