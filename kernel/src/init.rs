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
/// Attempts to detect and initialize network hardware, then creates the network stack.
/// The stack is stored in the global network stack for HTTP client access.
///
/// # Arguments
///
/// * `config` - The moteOS configuration
///
/// # Returns
///
/// * `Ok(NetworkStack)` - Successfully initialized network stack
/// * `Err(NetError)` - Failed to initialize (network not available or configuration error)
pub fn init_network(config: &MoteConfig) -> Result<NetworkStack, NetError> {
    use alloc::boxed::Box;
    use network::drivers::NetworkDriver;
    
    // Try to detect and initialize a network driver
    // Priority: virtio-net (for VMs) > e1000 > RTL8139
    
    // Try virtio-net first (common in QEMU/KVM)
    #[cfg(target_arch = "x86_64")]
    {
        use network::drivers::virtio::VirtioNet;
        
        match VirtioNet::new() {
            Ok(driver) => {
                let driver: Box<dyn NetworkDriver> = Box::new(driver);
                
                // Determine IP configuration
                let ip_config = if let Some(static_ip) = &config.network.static_ip {
                    // Use static IP configuration
                    let ip = Ipv4Address::new(
                        static_ip.ip[0],
                        static_ip.ip[1],
                        static_ip.ip[2],
                        static_ip.ip[3],
                    );
                    // Calculate prefix length from subnet mask
                    let prefix = subnet_mask_to_prefix(&static_ip.subnet_mask);
                    Some((ip, prefix))
                } else {
                    // Use DHCP (will be configured later)
                    None
                };
                
                // Create the network stack
                // Note: We'll store this in KernelState for polling
                // HTTP clients will use the global network stack
                // Since we can't easily share the stack, we'll initialize the global
                // with the same driver. In a full implementation, we'd use Arc or similar.
                let stack = NetworkStack::new(driver, ip_config)?;
                
                // Also initialize the global network stack for HTTP client access
                // Note: This creates a second driver instance which may not work
                // if the PCI device can only be initialized once.
                // For now, we'll attempt it and handle errors gracefully.
                // In production, we'd use shared ownership (Arc) or a different design.
                if let Ok(global_driver) = VirtioNet::new() {
                    let global_driver: Box<dyn NetworkDriver> = Box::new(global_driver);
                    // Try to initialize global stack (HTTP clients use this)
                    let _ = network::init_network_stack(global_driver, ip_config);
                    // If this fails, HTTP clients won't work, but polling will
                }
                
                // Start DHCP if not using static IP
                if ip_config.is_none() {
                    // DHCP will be started when needed via start_dhcp()
                }
                
                return Ok(stack);
            }
            Err(_) => {
                // virtio-net not available, try other drivers
            }
        }
    }
    
    // No network driver found
    // Return error - network is optional, so this is acceptable
    Err(NetError::DriverError("No network driver available".into()))
}

/// Convert subnet mask to prefix length
///
/// # Arguments
///
/// * `mask` - Subnet mask as [u8; 4]
///
/// # Returns
///
/// Prefix length (0-32)
fn subnet_mask_to_prefix(mask: &[u8; 4]) -> u8 {
    let mut prefix = 0;
    for &byte in mask.iter() {
        if byte == 0xFF {
            prefix += 8;
        } else {
            // Count leading ones in the byte
            let mut remaining = byte;
            while remaining & 0x80 != 0 {
                prefix += 1;
                remaining <<= 1;
            }
            break;
        }
    }
    prefix
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
