# Setup Wizard Implementation

This document describes the setup wizard implementation for moteOS, as specified in Section 3.7.8 of the Technical Specifications.

## Overview

The setup wizard provides a state machine-based interface for first-time configuration of:
- Network connectivity (Ethernet or WiFi)
- WiFi credentials (if applicable)
- LLM provider API keys
- System preferences

## Architecture

### State Machine

The wizard is implemented as a state machine with the following states:

```rust
pub enum WizardState {
    Welcome,                            // Initial welcome screen
    NetworkTypeSelect,                  // Choose Ethernet or WiFi
    NetworkScan { networks },           // Scanning for WiFi networks
    NetworkSelect { selected_index },   // Select from available networks
    NetworkPassword { ssid },           // Enter WiFi password
    ApiKeyMenu,                         // Choose provider or skip
    ApiKeyInput { provider },           // Enter API key for provider
    Ready { config },                   // Review configuration
    Complete,                           // Wizard complete
}
```

### Key Components

#### 1. `types.rs` - Configuration Types
Defines the main configuration structures:
- `MoteConfig` - Root configuration object
- `NetworkConfig` - Network settings (Ethernet/WiFi)
- `ProviderConfigs` - API keys for all providers
- `Preferences` - User preferences (theme, temperature, etc.)
- `WifiNetwork` - WiFi network information

#### 2. `wizard.rs` - State Machine
Implements the setup wizard logic:
- `SetupWizard` - Main wizard state machine
- `WizardEvent` - Events emitted by wizard
- `Key` - Keyboard input abstraction
- State transition handlers for each state

#### 3. `crypto.rs` - API Key Encryption
Provides encryption/decryption for API keys:
- `encrypt_api_key()` - Encrypts API key with AES-256-GCM
- `decrypt_api_key()` - Decrypts API key
- Hardware-derived encryption key when available

## Usage

### Creating a Wizard

```rust
use config::SetupWizard;

let mut wizard = SetupWizard::new();
```

### Handling Input

```rust
use config::Key;

let event = wizard.handle_input(Key::Enter);
match event {
    WizardEvent::RequestWifiScan => {
        // Trigger WiFi scan
    }
    WizardEvent::RequestWifiConnect { ssid, password } => {
        // Connect to WiFi network
    }
    WizardEvent::ConfigReady(config) => {
        // Save configuration
    }
    WizardEvent::Complete => {
        // Wizard finished
    }
    _ => {}
}
```

### Providing WiFi Networks

After receiving `RequestWifiScan` event:

```rust
let networks = scan_wifi_networks();
wizard.set_wifi_networks(networks);
```

### Getting Configuration

```rust
if let Some(config) = wizard.get_config() {
    // Configuration is ready
    storage.save(config)?;
}
```

## State Flow

```
Welcome
  ↓
NetworkTypeSelect
  ├─→ Ethernet → ApiKeyMenu
  └─→ WiFi → NetworkScan
               ↓
             NetworkSelect
               ↓
             NetworkPassword
               ↓
             ApiKeyMenu
               ↓
             ApiKeyInput (optional)
               ↓
             Ready
               ↓
             Complete
```

## Keyboard Navigation

### Welcome Screen
- `Enter` - Continue to network selection
- `Esc` - Cancel wizard

### Network Type Select
- `1` or `e` - Select Ethernet
- `2` or `w` - Select WiFi
- `Esc` - Go back

### Network Select
- `Up/Down` - Navigate network list
- `Enter` - Select network
- `Esc` - Go back

### Password Input
- `Char(c)` - Type character
- `Backspace` - Delete character
- `Enter` - Submit password
- `Esc` - Go back

### API Key Menu
- `1-4` - Select provider (OpenAI, Anthropic, Groq, xAI)
- `s` or `Enter` - Skip (use local model only)
- `Esc` - Go back

### API Key Input
- `Char(c)` - Type character
- `Backspace` - Delete character
- `Enter` - Submit API key
- `Esc` - Go back

### Ready Screen
- `Enter` - Save and complete
- `Esc` - Go back to modify

## Security

### API Key Encryption

API keys are encrypted before storage using AES-256-GCM:
- Encryption key derived from hardware (CPUID, TPM, etc.) when available
- Falls back to static key if hardware unavailable (development only)
- Nonce prepended to ciphertext
- Authentication tag ensures integrity

**Note**: The current implementation uses placeholder encryption. Production code should implement proper AES-256-GCM using the `aes-gcm` crate.

### Storage

Configuration is stored in:
1. EFI variables (primary) - `MoteOS-Config` variable
2. Boot partition file (fallback) - `/boot/mote.toml`

## Integration with TUI

The wizard will be rendered by the TUI layer. Each state provides:
- Current state via `wizard.state()`
- Input buffer via `wizard.input_buffer()`
- Cursor position via `wizard.cursor_pos()`
- Available networks via `wizard.available_networks()`
- Selected index via `wizard.selected_network_index()`

The TUI layer should:
1. Render appropriate UI for current state
2. Forward keyboard input to wizard
3. Handle wizard events (scan, connect, save)
4. Display status/error messages

## Example Screen Layouts

### Welcome Screen
```
╔══════════════════════════════════════════╗
║     Welcome to moteOS Setup Wizard       ║
║                                          ║
║  This wizard will help you configure:    ║
║  • Network connectivity                  ║
║  • LLM provider API keys                 ║
║  • System preferences                    ║
║                                          ║
║  Press Enter to continue or Esc to exit  ║
╚══════════════════════════════════════════╝
```

### Network Type Select
```
╔══════════════════════════════════════════╗
║        Network Configuration             ║
║                                          ║
║  Select network type:                    ║
║                                          ║
║  [1] Ethernet (wired connection)         ║
║  [2] WiFi (wireless connection)          ║
║                                          ║
║  Esc: Back                               ║
╚══════════════════════════════════════════╝
```

### Network Select
```
╔══════════════════════════════════════════╗
║      Available WiFi Networks             ║
║                                          ║
║  > MyHomeNetwork    [▂▄▆█] WPA2         ║
║    GuestNetwork     [▂▄__] WPA2         ║
║    CoffeeShop       [▂___] Open         ║
║                                          ║
║  ↑↓: Navigate  Enter: Select  Esc: Back  ║
╚══════════════════════════════════════════╝
```

### Password Input
```
╔══════════════════════════════════════════╗
║         WiFi Password                    ║
║                                          ║
║  Network: MyHomeNetwork                  ║
║                                          ║
║  Password: ********                      ║
║            ^                             ║
║                                          ║
║  Enter: Connect  Esc: Back               ║
╚══════════════════════════════════════════╝
```

### API Key Menu
```
╔══════════════════════════════════════════╗
║         API Key Configuration            ║
║                                          ║
║  Configure LLM provider (optional):      ║
║                                          ║
║  [1] OpenAI (GPT-4, GPT-3.5)            ║
║  [2] Anthropic (Claude)                  ║
║  [3] Groq (LLaMA, Mixtral)              ║
║  [4] xAI (Grok)                         ║
║  [s] Skip (use local model)             ║
║                                          ║
║  Esc: Back                               ║
╚══════════════════════════════════════════╝
```

### API Key Input
```
╔══════════════════════════════════════════╗
║         OpenAI API Key                   ║
║                                          ║
║  Enter your OpenAI API key:              ║
║                                          ║
║  sk-proj-********************************║
║         ^                                ║
║                                          ║
║  Enter: Save  Esc: Cancel                ║
╚══════════════════════════════════════════╝
```

### Ready Screen
```
╔══════════════════════════════════════════╗
║      Configuration Summary               ║
║                                          ║
║  Network: WiFi (MyHomeNetwork)           ║
║  Provider: OpenAI                        ║
║  Model: gpt-4o                          ║
║  Theme: Dark                             ║
║                                          ║
║  Press Enter to save and continue        ║
║  Press Esc to go back and modify         ║
╚══════════════════════════════════════════╝
```

## Testing

### Manual Testing

To test the wizard:
1. Boot moteOS without existing configuration
2. Wizard should launch automatically
3. Navigate through each state
4. Verify state transitions
5. Verify configuration saved correctly

### Unit Tests

Run unit tests with:
```bash
cargo test -p config
```

Tests verify:
- State transitions
- Input handling
- API key encryption/decryption
- Configuration serialization

## Future Enhancements

Potential improvements for future versions:
1. Validation UI
   - Check network connectivity after WiFi setup
   - Validate API keys before saving
   - Test LLM provider connection

2. Advanced Options
   - Static IP configuration
   - Custom DNS servers
   - Proxy settings
   - Temperature/sampling parameters

3. Multi-language Support
   - Translate wizard UI text
   - Support for non-ASCII passwords

4. Progress Indicators
   - Show scanning progress
   - Connection timeout indicators
   - API validation status

## Dependencies

The wizard implementation has minimal dependencies:
- `alloc` - For heap-allocated strings and vectors
- `uefi` - For EFI variable storage (via `storage::efi`)

All code is `no_std` compatible and runs in the bare-metal unikernel environment.

## Files

- `config/src/types.rs` - Configuration type definitions
- `config/src/wizard.rs` - Setup wizard state machine
- `config/src/crypto.rs` - API key encryption utilities
- `config/src/error.rs` - Error types (extended)
- `config/src/lib.rs` - Module exports

## References

- Technical Specifications: Section 3.7.8 (Setup Wizard)
- Technical Specifications: Section 7.2 (Config Types)
- Technical Specifications: Section 7.7 (API Key Encryption)
- TOML Parser: `config/src/toml.rs`
- EFI Storage: `config/src/storage/efi.rs`
