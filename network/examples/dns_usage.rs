//! DNS resolver usage example
//!
//! This example demonstrates how to use the DNS resolver to resolve hostnames
//! to IPv4 addresses.
//!
//! To run this example (in a hosted environment):
//! ```
//! cargo run --example dns_usage
//! ```

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use network::{drivers::NetworkDriver, NetError, NetworkStack};
use smoltcp::wire::Ipv4Address;

// Mock time source for example
fn get_time_ms() -> i64 {
    // In a real kernel, this would return actual system time
    // For this example, we'll use a simple counter
    static mut TIME: i64 = 0;
    unsafe {
        TIME += 1;
        TIME
    }
}

// Mock sleep function for example
fn sleep_ms(ms: i64) {
    // In a real kernel, this would sleep/yield for the specified time
    // For this example, we'll just increment the time counter
    unsafe {
        for _ in 0..ms {
            let _ = get_time_ms();
        }
    }
}

/// Example: Resolve a hostname using DNS
///
/// This demonstrates the complete flow:
/// 1. Initialize network stack (assumed to be done with DHCP)
/// 2. Get DNS server from DHCP configuration
/// 3. Resolve hostname to IP address
fn dns_resolve_example(stack: &mut NetworkStack) -> Result<(), NetError> {
    // Example DNS servers
    let google_dns = Ipv4Address::new(8, 8, 8, 8);
    let cloudflare_dns = Ipv4Address::new(1, 1, 1, 1);

    // Example 1: Resolve example.com using Google DNS
    println!("Resolving example.com using Google DNS (8.8.8.8)...");
    match stack.dns_resolve("example.com", google_dns, 5000, get_time_ms, Some(sleep_ms)) {
        Ok(ip) => {
            println!("  ✓ Resolved to: {}", ip);
        }
        Err(e) => {
            println!("  ✗ Failed: {:?}", e);
        }
    }

    // Example 2: Resolve api.openai.com using Cloudflare DNS
    println!("\nResolving api.openai.com using Cloudflare DNS (1.1.1.1)...");
    match stack.dns_resolve(
        "api.openai.com",
        cloudflare_dns,
        5000,
        get_time_ms,
        Some(sleep_ms),
    ) {
        Ok(ip) => {
            println!("  ✓ Resolved to: {}", ip);
        }
        Err(e) => {
            println!("  ✗ Failed: {:?}", e);
        }
    }

    // Example 3: Resolve api.anthropic.com
    println!("\nResolving api.anthropic.com using Google DNS (8.8.8.8)...");
    match stack.dns_resolve(
        "api.anthropic.com",
        google_dns,
        5000,
        get_time_ms,
        Some(sleep_ms),
    ) {
        Ok(ip) => {
            println!("  ✓ Resolved to: {}", ip);
        }
        Err(e) => {
            println!("  ✗ Failed: {:?}", e);
        }
    }

    // Example 4: Try to resolve a non-existent domain
    println!("\nResolving nonexistent.example.invalid (should fail)...");
    match stack.dns_resolve(
        "nonexistent.example.invalid",
        google_dns,
        5000,
        get_time_ms,
        Some(sleep_ms),
    ) {
        Ok(ip) => {
            println!("  ✓ Resolved to: {} (unexpected)", ip);
        }
        Err(e) => {
            println!("  ✗ Failed as expected: {:?}", e);
        }
    }

    Ok(())
}

/// Example: Using DNS with DHCP-provided DNS servers
fn dns_with_dhcp_example(stack: &mut NetworkStack) -> Result<(), NetError> {
    println!("\n=== DNS with DHCP Example ===\n");

    // First, acquire DHCP configuration (assumed to have DNS servers)
    println!("Acquiring DHCP configuration...");
    let config = stack.dhcp_acquire(30_000, get_time_ms, Some(sleep_ms))?;

    println!("  ✓ DHCP configuration acquired:");
    println!("    IP: {}", config.ip);
    if let Some(gateway) = config.gateway {
        println!("    Gateway: {}", gateway);
    }
    println!("    DNS servers: {} server(s)", config.dns.len());
    for (i, dns) in config.dns.iter().enumerate() {
        println!("      DNS {}: {}", i + 1, dns);
    }

    // Use the first DNS server from DHCP
    if let Some(dns_server) = config.dns.first() {
        println!(
            "\nResolving hostname using DHCP-provided DNS ({})...",
            dns_server
        );

        match stack.dns_resolve(
            "example.com",
            *dns_server,
            5000,
            get_time_ms,
            Some(sleep_ms),
        ) {
            Ok(ip) => {
                println!("  ✓ Resolved example.com to: {}", ip);
            }
            Err(e) => {
                println!("  ✗ Failed: {:?}", e);
            }
        }
    } else {
        println!("\n  ✗ No DNS servers provided by DHCP");
    }

    Ok(())
}

/// Example: Complete network initialization with DNS
fn complete_network_setup_example() -> Result<(), NetError> {
    println!("\n=== Complete Network Setup Example ===\n");

    // 1. Initialize network driver (mock for example)
    // In a real kernel, this would initialize virtio-net or other driver
    // let driver = Box::new(VirtioNetDriver::new()?);

    // 2. Create network stack with no IP initially
    // let mut stack = NetworkStack::new(driver, None)?;
    // println!("✓ Network stack initialized");

    // 3. Acquire DHCP configuration
    // println!("Acquiring DHCP configuration...");
    // let config = stack.dhcp_acquire(30_000, get_time_ms, Some(sleep_ms))?;
    // println!("  ✓ IP configured: {}", config.ip);

    // 4. Resolve hostname
    // let dns_server = config.dns.first().copied().unwrap_or(Ipv4Address::new(8, 8, 8, 8));
    // println!("\nResolving api.openai.com...");
    // let api_ip = stack.dns_resolve("api.openai.com", dns_server, 5000, get_time_ms, Some(sleep_ms))?;
    // println!("  ✓ Resolved to: {}", api_ip);

    // 5. Now ready to make HTTP/HTTPS requests to api_ip
    // println!("\nNetwork setup complete! Ready to make API requests.");

    println!("(Skipped in example - requires actual network hardware)");

    Ok(())
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Mock entry point for example
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("=== DNS Resolver Usage Examples ===\n");
    println!("These examples demonstrate DNS resolution in moteOS.\n");

    // Note: In a real environment, you would:
    // 1. Initialize the network driver
    // 2. Create the network stack
    // 3. Run DHCP to get network configuration
    // 4. Use DNS to resolve hostnames before making API calls

    complete_network_setup_example().ok();

    loop {}
}

// Placeholder for println in no_std
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        // In a real kernel, this would write to serial or framebuffer
        // For now, this is a no-op in the example
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_compiles() {
        // This test just ensures the example compiles correctly
        assert!(true);
    }
}
