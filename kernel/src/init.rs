//! Kernel initialization functions
//!
//! This module contains functions for initializing various kernel components
//! including the heap allocator, network stack, and LLM providers.

use alloc::boxed::Box;
use alloc::string::String;
use config::{decrypt_api_key, MoteConfig};
use linked_list_allocator::LockedHeap;
use llm::{AnthropicClient, GroqClient, LlmProvider, OpenAiClient, XaiClient};
use network::{init_network_stack, NetworkStack, NetError};
use smoltcp::wire::Ipv4Address;

/// Global heap allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the heap allocator
///
/// Sets up the global heap allocator with the given start address and size.
/// This must be called before any heap allocations are made.
///
/// # Arguments
///
/// * `heap_start` - Physical address of the heap start
/// * `heap_size` - Size of the heap in bytes
///
/// # Safety
///
/// This function is safe to call once during kernel initialization.
/// The heap memory region must be valid and not used for anything else.
pub fn init_heap(heap_start: usize, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

/// Initialize network stack
///
/// Sets up the network stack based on configuration.
/// Returns None if network is not configured or initialization fails.
pub fn init_network(config: &MoteConfig) -> Result<NetworkStack, NetError> {
    // For now, network initialization is a placeholder
    // In a full implementation, this would:
    // 1. Detect network hardware (virtio-net, e1000, etc.)
    // 2. Initialize the network driver
    // 3. Create the network stack
    // 4. Configure IP (DHCP or static)
    // 5. Return the network stack
    
    // TODO: Implement actual network initialization
    // For now, return an error to indicate network is not available
    Err(NetError::DriverError("Network initialization not yet implemented".into()))
}

/// Get DNS server from network config or use default
///
/// Returns the first DNS server from the config, or a default (8.8.8.8) if none is configured.
fn get_dns_server(network: Option<&NetworkStack>) -> Ipv4Address {
    // Try to get DNS from network stack DHCP config
    if let Some(net) = network {
        if let Some(config) = net.dhcp_config() {
            if let Some(dns) = config.dns.first() {
                return Ipv4Address::new(dns[0], dns[1], dns[2], dns[3]);
            }
        }
    }
    
    // Default to Google DNS
    Ipv4Address::new(8, 8, 8, 8)
}

/// Get current time in milliseconds
///
/// This is a simple implementation that uses ticks.
/// In a full implementation, this would use a proper timer.
pub fn get_time_ms() -> i64 {
    // TODO: Use proper timer
    // For now, return a placeholder
    boot::timer::get_ticks() as i64 * 10 // Assume 100Hz = 10ms per tick
}

/// Sleep for the specified number of milliseconds
///
/// This uses the timer's sleep function.
pub fn sleep_ms(ms: i64) {
    boot::timer::sleep_ms(ms as u64);
}

/// Initialize LLM provider from configuration
///
/// Creates and returns the configured LLM provider along with its name and default model.
///
/// # Arguments
///
/// * `config` - The moteOS configuration
/// * `network` - Optional network stack (required for cloud providers)
///
/// # Returns
///
/// Returns a tuple of (provider, provider_name, model_name) on success.
pub fn init_provider(
    config: &MoteConfig,
    network: Option<&NetworkStack>,
) -> Result<(Box<dyn LlmProvider>, String, String), String> {
    let provider_name = &config.preferences.default_provider;
    let dns_server = get_dns_server(network);
    
    match provider_name.as_str() {
        "openai" => {
            let provider_config = config
                .providers
                .openai
                .as_ref()
                .ok_or("OpenAI provider not configured")?;
            
            let api_key = decrypt_api_key(&provider_config.api_key_encrypted)
                .map_err(|_| "Failed to decrypt OpenAI API key")?;
            
            let client = OpenAiClient::new(api_key, dns_server, get_time_ms, Some(sleep_ms));
            let model = provider_config.default_model.clone();
            
            Ok((Box::new(client), "OpenAI".to_string(), model))
        }
        
        "anthropic" => {
            let provider_config = config
                .providers
                .anthropic
                .as_ref()
                .ok_or("Anthropic provider not configured")?;
            
            let api_key = decrypt_api_key(&provider_config.api_key_encrypted)
                .map_err(|_| "Failed to decrypt Anthropic API key")?;
            
            let client = AnthropicClient::new(api_key, dns_server, get_time_ms, Some(sleep_ms));
            let model = provider_config.default_model.clone();
            
            Ok((Box::new(client), "Anthropic".to_string(), model))
        }
        
        "groq" => {
            let provider_config = config
                .providers
                .groq
                .as_ref()
                .ok_or("Groq provider not configured")?;
            
            let api_key = decrypt_api_key(&provider_config.api_key_encrypted)
                .map_err(|_| "Failed to decrypt Groq API key")?;
            
            let client = GroqClient::new(api_key, dns_server, get_time_ms, Some(sleep_ms));
            let model = provider_config.default_model.clone();
            
            Ok((Box::new(client), "Groq".to_string(), model))
        }
        
        "xai" => {
            let provider_config = config
                .providers
                .xai
                .as_ref()
                .ok_or("xAI provider not configured")?;
            
            let api_key = decrypt_api_key(&provider_config.api_key_encrypted)
                .map_err(|_| "Failed to decrypt xAI API key")?;
            
            let client = XaiClient::new(api_key, dns_server, get_time_ms, Some(sleep_ms));
            let model = provider_config.default_model.clone();
            
            Ok((Box::new(client), "xAI".to_string(), model))
        }
        
        "local" | "ollama" => {
            // TODO: Implement local provider initialization
            // For now, return an error
            Err("Local provider not yet implemented".to_string())
        }
        
        _ => {
            // Default to OpenAI if provider name is unknown
            // Try to initialize OpenAI as fallback
            if let Some(provider_config) = &config.providers.openai {
                if let Ok(api_key) = decrypt_api_key(&provider_config.api_key_encrypted) {
                    let client = OpenAiClient::new(api_key, dns_server, get_time_ms, Some(sleep_ms));
                    let model = provider_config.default_model.clone();
                    return Ok((Box::new(client), "OpenAI".to_string(), model));
                }
            }
            
            Err(format!("Unknown provider: {}", provider_name))
        }
    }
}
