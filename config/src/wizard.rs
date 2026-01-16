//! Setup wizard for first-time configuration
//!
//! Provides a state machine-based wizard for configuring network settings
//! and API keys during initial setup.
//!
//! # Architecture
//!
//! This module implements an event-driven state machine that decouples the wizard
//! logic from the UI rendering and network operations. The wizard emits events
//! (via `WizardEvent`) that the caller must handle:
//!
//! - `RequestWifiScan` - Caller should scan for WiFi networks and call `set_wifi_networks()`
//! - `RequestWifiConnect` - Caller should connect to WiFi with provided credentials
//! - `ConfigReady` - Caller should save the configuration (e.g., to EFI variables)
//! - `Complete` - Wizard finished successfully
//!
//! The wizard does NOT directly render UI or save configuration. This is the
//! responsibility of the caller (typically the TUI layer and kernel integration code).
//!
//! # Usage Example
//!
//! ```ignore
//! let mut wizard = SetupWizard::new();
//!
//! loop {
//!     // Render current state (caller's responsibility)
//!     render_wizard_state(wizard.state());
//!
//!     // Get keyboard input
//!     let key = read_keyboard();
//!
//!     // Handle input
//!     match wizard.handle_input(key) {
//!         WizardEvent::RequestWifiScan => {
//!             let networks = scan_wifi();
//!             wizard.set_wifi_networks(networks);
//!         }
//!         WizardEvent::RequestWifiConnect { ssid, password } => {
//!             connect_wifi(&ssid, &password);
//!         }
//!         WizardEvent::ConfigReady(config) => {
//!             storage.save(&config)?;
//!         }
//!         WizardEvent::Complete => break,
//!         _ => {}
//!     }
//! }
//! ```

#![no_std]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{MoteConfig, WifiNetwork, ConnectionType, ProviderConfig};
use crate::crypto;

/// Setup wizard state machine
#[derive(Debug)]
pub struct SetupWizard {
    state: WizardState,
    config: MoteConfig,

    // Input buffer for current field
    input_buffer: String,
    cursor_pos: usize,

    // Network scan results
    available_networks: Vec<WifiNetwork>,
    selected_network_index: usize,

    // API key input tracking
    current_provider: ApiKeyProvider,
}

/// Wizard state
#[derive(Debug, Clone)]
pub enum WizardState {
    /// Welcome screen
    Welcome,

    /// Network type selection (Ethernet or WiFi)
    NetworkTypeSelect,

    /// WiFi network scanning
    NetworkScan { networks: Vec<WifiNetwork> },

    /// WiFi network selection from scanned networks
    NetworkSelect { selected_index: usize },

    /// WiFi password input
    NetworkPassword { ssid: String },

    /// API key configuration selection
    ApiKeyMenu,

    /// API key input for specific provider
    ApiKeyInput { provider: ApiKeyProvider },

    /// Ready screen (summary before saving)
    Ready { config: MoteConfig },

    /// Completion screen
    Complete,
}

/// Provider for API key input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyProvider {
    OpenAI,
    Anthropic,
    Groq,
    XAI,
    Skip,  // Skip to use local model only
}

/// Events emitted by the wizard
#[derive(Debug, Clone)]
pub enum WizardEvent {
    /// No event
    None,

    /// Request WiFi scan
    RequestWifiScan,

    /// Request WiFi connection with credentials
    RequestWifiConnect { ssid: String, password: String },

    /// Configuration is ready to be saved
    ConfigReady(MoteConfig),

    /// Wizard completed
    Complete,

    /// User cancelled wizard
    Cancelled,
}

/// Key input for wizard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Esc,
    F(u8),
}

impl SetupWizard {
    /// Create a new setup wizard
    pub fn new() -> Self {
        Self {
            state: WizardState::Welcome,
            config: MoteConfig::default(),
            input_buffer: String::new(),
            cursor_pos: 0,
            available_networks: Vec::new(),
            selected_network_index: 0,
            current_provider: ApiKeyProvider::Skip,
        }
    }

    /// Get the current state
    pub fn state(&self) -> &WizardState {
        &self.state
    }

    /// Get the current configuration (if complete)
    pub fn get_config(&self) -> Option<&MoteConfig> {
        match &self.state {
            WizardState::Ready { config } | WizardState::Complete => Some(&self.config),
            _ => None,
        }
    }

    /// Get input buffer (for rendering)
    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    /// Get cursor position (for rendering)
    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    /// Get available networks (for rendering)
    pub fn available_networks(&self) -> &[WifiNetwork] {
        &self.available_networks
    }

    /// Get selected network index (for rendering)
    pub fn selected_network_index(&self) -> usize {
        self.selected_network_index
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, key: Key) -> WizardEvent {
        match &self.state {
            WizardState::Welcome => self.handle_welcome_input(key),
            WizardState::NetworkTypeSelect => self.handle_network_type_select(key),
            WizardState::NetworkScan { .. } => self.handle_network_scan_input(key),
            WizardState::NetworkSelect { .. } => self.handle_network_select_input(key),
            WizardState::NetworkPassword { .. } => self.handle_password_input(key),
            WizardState::ApiKeyMenu => self.handle_api_key_menu_input(key),
            WizardState::ApiKeyInput { .. } => self.handle_api_key_input(key),
            WizardState::Ready { .. } => self.handle_ready_input(key),
            WizardState::Complete => WizardEvent::Complete,
        }
    }

    /// Update with WiFi scan results
    pub fn set_wifi_networks(&mut self, networks: Vec<WifiNetwork>) {
        self.available_networks = networks.clone();
        self.selected_network_index = 0;
        self.state = WizardState::NetworkSelect { selected_index: 0 };
    }

    /// Handle welcome screen input
    fn handle_welcome_input(&mut self, key: Key) -> WizardEvent {
        match key {
            Key::Enter => {
                self.state = WizardState::NetworkTypeSelect;
                WizardEvent::None
            }
            Key::Esc => WizardEvent::Cancelled,
            _ => WizardEvent::None,
        }
    }

    /// Handle network type selection
    fn handle_network_type_select(&mut self, key: Key) -> WizardEvent {
        match key {
            Key::Char('1') | Key::Char('e') => {
                // Ethernet selected
                self.config.network.connection_type = ConnectionType::Ethernet;
                self.state = WizardState::ApiKeyMenu;
                WizardEvent::None
            }
            Key::Char('2') | Key::Char('w') => {
                // WiFi selected - request scan
                self.config.network.connection_type = ConnectionType::Wifi;
                self.state = WizardState::NetworkScan { networks: Vec::new() };
                WizardEvent::RequestWifiScan
            }
            Key::Esc => {
                self.state = WizardState::Welcome;
                WizardEvent::None
            }
            _ => WizardEvent::None,
        }
    }

    /// Handle network scan state
    fn handle_network_scan_input(&mut self, key: Key) -> WizardEvent {
        match key {
            Key::Esc => {
                self.state = WizardState::NetworkTypeSelect;
                WizardEvent::None
            }
            _ => WizardEvent::None,
        }
    }

    /// Handle network selection
    fn handle_network_select_input(&mut self, key: Key) -> WizardEvent {
        match key {
            Key::Up => {
                if self.selected_network_index > 0 {
                    self.selected_network_index -= 1;
                }
                WizardEvent::None
            }
            Key::Down => {
                if self.selected_network_index < self.available_networks.len().saturating_sub(1) {
                    self.selected_network_index += 1;
                }
                WizardEvent::None
            }
            Key::Enter => {
                if let Some(network) = self.available_networks.get(self.selected_network_index) {
                    let ssid = network.ssid.clone();
                    self.config.network.wifi_ssid = Some(ssid.clone());
                    self.input_buffer.clear();
                    self.cursor_pos = 0;
                    self.state = WizardState::NetworkPassword { ssid };
                }
                WizardEvent::None
            }
            Key::Esc => {
                self.state = WizardState::NetworkTypeSelect;
                WizardEvent::None
            }
            _ => WizardEvent::None,
        }
    }

    /// Handle password input
    fn handle_password_input(&mut self, key: Key) -> WizardEvent {
        match key {
            Key::Char(ch) => {
                self.input_buffer.push(ch);
                self.cursor_pos += 1;
                WizardEvent::None
            }
            Key::Backspace => {
                if !self.input_buffer.is_empty() && self.cursor_pos > 0 {
                    self.input_buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
                WizardEvent::None
            }
            Key::Enter => {
                if let WizardState::NetworkPassword { ssid } = &self.state {
                    let password = self.input_buffer.clone();
                    let ssid = ssid.clone();

                    // Move to API key menu
                    self.state = WizardState::ApiKeyMenu;
                    self.input_buffer.clear();
                    self.cursor_pos = 0;

                    // Return event to connect
                    return WizardEvent::RequestWifiConnect { ssid, password };
                }
                WizardEvent::None
            }
            Key::Esc => {
                self.state = WizardState::NetworkSelect {
                    selected_index: self.selected_network_index
                };
                self.input_buffer.clear();
                self.cursor_pos = 0;
                WizardEvent::None
            }
            _ => WizardEvent::None,
        }
    }

    /// Handle API key menu
    fn handle_api_key_menu_input(&mut self, key: Key) -> WizardEvent {
        match key {
            Key::Char('1') => {
                self.current_provider = ApiKeyProvider::OpenAI;
                self.input_buffer.clear();
                self.cursor_pos = 0;
                self.state = WizardState::ApiKeyInput { provider: ApiKeyProvider::OpenAI };
                WizardEvent::None
            }
            Key::Char('2') => {
                self.current_provider = ApiKeyProvider::Anthropic;
                self.input_buffer.clear();
                self.cursor_pos = 0;
                self.state = WizardState::ApiKeyInput { provider: ApiKeyProvider::Anthropic };
                WizardEvent::None
            }
            Key::Char('3') => {
                self.current_provider = ApiKeyProvider::Groq;
                self.input_buffer.clear();
                self.cursor_pos = 0;
                self.state = WizardState::ApiKeyInput { provider: ApiKeyProvider::Groq };
                WizardEvent::None
            }
            Key::Char('4') => {
                self.current_provider = ApiKeyProvider::XAI;
                self.input_buffer.clear();
                self.cursor_pos = 0;
                self.state = WizardState::ApiKeyInput { provider: ApiKeyProvider::XAI };
                WizardEvent::None
            }
            Key::Char('s') | Key::Enter => {
                // Skip - use local model only
                self.state = WizardState::Ready {
                    config: self.config.clone()
                };
                WizardEvent::None
            }
            Key::Esc => {
                // Go back to network selection
                if self.config.network.connection_type == ConnectionType::Wifi {
                    self.state = WizardState::NetworkSelect {
                        selected_index: self.selected_network_index
                    };
                } else {
                    self.state = WizardState::NetworkTypeSelect;
                }
                WizardEvent::None
            }
            _ => WizardEvent::None,
        }
    }

    /// Handle API key input
    fn handle_api_key_input(&mut self, key: Key) -> WizardEvent {
        match key {
            Key::Char(ch) => {
                self.input_buffer.push(ch);
                self.cursor_pos += 1;
                WizardEvent::None
            }
            Key::Backspace => {
                if !self.input_buffer.is_empty() && self.cursor_pos > 0 {
                    self.input_buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
                WizardEvent::None
            }
            Key::Enter => {
                // Get the API key from input buffer
                let api_key = self.input_buffer.clone();

                // Encrypt the API key
                match crypto::encrypt_api_key(&api_key) {
                    Ok(encrypted_key) => {
                        // Determine default model for the provider
                        let default_model = match self.current_provider {
                            ApiKeyProvider::OpenAI => "gpt-4o",
                            ApiKeyProvider::Anthropic => "claude-sonnet-4-20250514",
                            ApiKeyProvider::Groq => "llama-3.3-70b-versatile",
                            ApiKeyProvider::XAI => "grok-2",
                            ApiKeyProvider::Skip => "",
                        };

                        // Store encrypted API key in config
                        let provider_config = ProviderConfig {
                            api_key_encrypted: encrypted_key,
                            default_model: String::from(default_model),
                        };

                        match self.current_provider {
                            ApiKeyProvider::OpenAI => {
                                self.config.providers.openai = Some(provider_config);
                            }
                            ApiKeyProvider::Anthropic => {
                                self.config.providers.anthropic = Some(provider_config);
                            }
                            ApiKeyProvider::Groq => {
                                self.config.providers.groq = Some(provider_config);
                            }
                            ApiKeyProvider::XAI => {
                                self.config.providers.xai = Some(provider_config);
                            }
                            ApiKeyProvider::Skip => {}
                        }

                        // Clear sensitive data from input buffer
                        self.input_buffer.clear();
                        self.cursor_pos = 0;

                        // Move to ready state
                        self.state = WizardState::Ready {
                            config: self.config.clone()
                        };
                    }
                    Err(_) => {
                        // Encryption failed - stay in current state
                        // The UI should show an error message
                        // For now, just clear the input and stay here
                        self.input_buffer.clear();
                        self.cursor_pos = 0;
                    }
                }

                WizardEvent::None
            }
            Key::Esc => {
                self.state = WizardState::ApiKeyMenu;
                self.input_buffer.clear();
                self.cursor_pos = 0;
                WizardEvent::None
            }
            _ => WizardEvent::None,
        }
    }

    /// Handle ready screen
    fn handle_ready_input(&mut self, key: Key) -> WizardEvent {
        match key {
            Key::Enter => {
                self.state = WizardState::Complete;
                WizardEvent::ConfigReady(self.config.clone())
            }
            Key::Esc => {
                self.state = WizardState::ApiKeyMenu;
                WizardEvent::None
            }
            _ => WizardEvent::None,
        }
    }
}

impl Default for SetupWizard {
    fn default() -> Self {
        Self::new()
    }
}
