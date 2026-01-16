//! Configuration types for moteOS
//!
//! Defines the main configuration structures used throughout the system.

#![no_std]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Main configuration structure for moteOS
#[derive(Debug, Clone)]
pub struct MoteConfig {
    pub network: NetworkConfig,
    pub providers: ProviderConfigs,
    pub preferences: Preferences,
}

impl Default for MoteConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            providers: ProviderConfigs::default(),
            preferences: Preferences::default(),
        }
    }
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub connection_type: ConnectionType,
    pub wifi_ssid: Option<String>,
    pub wifi_password_encrypted: Option<Vec<u8>>,
    pub static_ip: Option<IpConfig>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Ethernet,
            wifi_ssid: None,
            wifi_password_encrypted: None,
            static_ip: None,
        }
    }
}

/// Type of network connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Ethernet,
    Wifi,
}

/// IP configuration (for static IP)
#[derive(Debug, Clone)]
pub struct IpConfig {
    pub ip: [u8; 4],
    pub gateway: [u8; 4],
    pub dns: Vec<[u8; 4]>,
    pub subnet_mask: [u8; 4],
}

/// Provider configurations for all supported LLM providers
#[derive(Debug, Clone, Default)]
pub struct ProviderConfigs {
    pub openai: Option<ProviderConfig>,
    pub anthropic: Option<ProviderConfig>,
    pub groq: Option<ProviderConfig>,
    pub xai: Option<ProviderConfig>,
    pub ollama: Option<LocalProviderConfig>,
    pub local: Option<LocalProviderConfig>,
}

/// Configuration for a cloud LLM provider
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub api_key_encrypted: Vec<u8>,
    pub default_model: String,
}

/// Configuration for a local provider (Ollama or bundled model)
#[derive(Debug, Clone)]
pub struct LocalProviderConfig {
    pub endpoint: String,
    pub default_model: String,
}

/// User preferences
#[derive(Debug, Clone)]
pub struct Preferences {
    pub default_provider: String,
    pub default_model: String,
    pub theme: ThemeChoice,
    pub temperature: f32,
    pub stream_responses: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            default_provider: String::from("local"),
            default_model: String::from("smollm-360m"),
            theme: ThemeChoice::Dark,
            temperature: 0.7,
            stream_responses: true,
        }
    }
}

/// Theme choice
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    Dark,
    Light,
}

/// WiFi network information (used during setup)
#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: [u8; 6],
    pub signal_strength: i8,  // dBm
    pub security: SecurityType,
    pub channel: u8,
    pub frequency: u16,  // MHz
}

/// WiFi security type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityType {
    Open,
    WPA2Personal,
    WPA3Personal,
}
