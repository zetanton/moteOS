#![no_std]

extern crate alloc;

pub mod crypto;
pub mod error;
pub mod storage;
pub mod toml;
pub mod types;
pub mod wizard;

pub use crypto::{decrypt_api_key, encrypt_api_key};
pub use error::ConfigError;
pub use storage::{efi::EfiConfigStorage, ConfigStorage};
pub use toml::{TomlParser, Value};
pub use types::{
    ConnectionType, IpConfig, LocalProviderConfig, MoteConfig, NetworkConfig, Preferences,
    ProviderConfig, ProviderConfigs, SecurityType, ThemeChoice, WifiNetwork,
};
pub use wizard::{ApiKeyProvider, Key, SetupWizard, WizardEvent, WizardState};
