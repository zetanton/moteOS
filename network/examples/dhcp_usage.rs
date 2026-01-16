//! Example demonstrating DHCP client usage
//!
//! This example shows how to use the DHCP client to automatically
//! acquire IP configuration from a DHCP server.
//!
//! # Usage
//!
//! ```no_run
//! use network::{NetworkStack, drivers::virtio::VirtioNetDriver};
//!
//! // Step 1: Initialize the network driver
//! let driver = VirtioNetDriver::new().expect("Failed to initialize driver");
//!
//! // Step 2: Create network stack without static IP (for DHCP)
//! let mut stack = NetworkStack::new(Box::new(driver), None)
//!     .expect("Failed to create network stack");
//!
//! // Step 3: Start DHCP client
//! stack.start_dhcp().expect("Failed to start DHCP");
//!
//! // Step 4: Poll the network stack regularly
//! // In a real application, this would be in the main loop
//! let mut timestamp = 0i64;
//! loop {
//!     // Poll network stack
//!     stack.poll(timestamp).expect("Failed to poll network");
//!
//!     // Check DHCP state
//!     if let Some(state) = stack.dhcp_state() {
//!         println!("DHCP State: {}", state);
//!
//!         // Check if we got configuration
//!         if let Some(config) = stack.dhcp_config() {
//!             println!("IP Address: {}", config.ip);
//!             println!("Subnet Mask: /{}", config.prefix_len);
//!
//!             if let Some(gateway) = config.gateway {
//!                 println!("Gateway: {}", gateway);
//!             }
//!
//!             for (i, dns) in config.dns.iter().enumerate() {
//!                 println!("DNS {}: {}", i + 1, dns);
//!             }
//!
//!             // Apply the configuration
//!             stack.apply_dhcp_config(&config)
//!                 .expect("Failed to apply DHCP config");
//!
//!             break;
//!         }
//!     }
//!
//!     // Increment timestamp (in reality, get from system timer)
//!     timestamp += 10;
//!
//!     // Add small delay
//!     // In reality: timer::sleep_ms(10);
//! }
//! ```
//!
//! # Alternative: Using dhcp_acquire (convenience method)
//!
//! ```no_run
//! use network::{NetworkStack, drivers::virtio::VirtioNetDriver};
//!
//! // Initialize driver and stack
//! let driver = VirtioNetDriver::new().expect("Failed to initialize driver");
//! let mut stack = NetworkStack::new(Box::new(driver), None)
//!     .expect("Failed to create network stack");
//!
//! // Function to get current system time in milliseconds
//! fn get_system_time_ms() -> i64 {
//!     // In a real implementation, read from system timer
//!     // For example: timer::get_ticks_ms()
//!     0 // Placeholder
//! }
//!
//! // Acquire DHCP configuration with 30 second timeout
//! let config = stack.dhcp_acquire(30_000, get_system_time_ms)
//!     .expect("Failed to acquire DHCP configuration");
//!
//! println!("Successfully configured:");
//! println!("  IP: {}/{}", config.ip, config.prefix_len);
//! if let Some(gateway) = config.gateway {
//!     println!("  Gateway: {}", gateway);
//! }
//! for dns in &config.dns {
//!     println!("  DNS: {}", dns);
//! }
//! ```
//!
//! # DHCP Flow
//!
//! The DHCP client implements the standard 4-way handshake:
//!
//! 1. **DISCOVER**: Client broadcasts to find DHCP servers
//! 2. **OFFER**: DHCP server offers an IP address
//! 3. **REQUEST**: Client requests the offered IP address
//! 4. **ACK**: Server acknowledges and provides full configuration
//!
//! After successful DHCP acquisition, the interface will be configured with:
//! - IP address and subnet mask
//! - Default gateway (router)
//! - DNS server addresses
//!
//! # Implementation Details
//!
//! The DHCP client uses smoltcp's built-in DHCPv4 socket implementation.
//! The socket handles all protocol details internally, and the application
//! just needs to:
//! - Create and add the DHCP socket to the socket set
//! - Poll the network stack regularly
//! - Check for and apply received configuration
//!
//! See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.7 for more details.

fn main() {
    println!("This is a documentation example. See the code above for usage.");
    println!("To actually run DHCP, you need to:");
    println!("  1. Initialize a network driver (e.g., VirtioNetDriver)");
    println!("  2. Create a NetworkStack with None for IP config");
    println!("  3. Call start_dhcp() or dhcp_acquire()");
    println!("  4. Poll regularly until configuration is acquired");
}
