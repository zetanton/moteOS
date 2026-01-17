//! TLS/HTTPS connection example
//!
//! This example demonstrates how to:
//! 1. Set up the network stack with DHCP
//! 2. Resolve a hostname using DNS
//! 3. Establish a TLS 1.3 connection
//! 4. Send an HTTPS request
//! 5. Receive and parse the response
//!
//! This example requires a network driver (e.g., virtio-net in QEMU)
//! and internet connectivity.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::ToString;
use core::panic::PanicInfo;
use network::{drivers::NetworkDriver, NetworkStack, TlsConnection};
use smoltcp::wire::Ipv4Address;

// Mock time functions for this example
// In a real implementation, these would use hardware timers
static mut MOCK_TIME: i64 = 0;

fn get_time_ms() -> i64 {
    unsafe {
        MOCK_TIME += 10; // Simulate 10ms per call
        MOCK_TIME
    }
}

fn sleep_ms(ms: i64) {
    // In a real implementation, this would yield CPU or use hardware timer
    // For this example, we just advance the mock time
    unsafe {
        MOCK_TIME += ms;
    }
}

/// Example: Connect to api.openai.com over HTTPS
fn example_https_connection(stack: &mut NetworkStack) -> Result<(), network::NetError> {
    // 1. Configure network with DHCP (if not already configured)
    let ip_config = stack.dhcp_acquire(30_000, get_time_ms, Some(sleep_ms))?;

    // 2. Resolve hostname using DNS
    let dns_server = ip_config
        .dns
        .first()
        .copied()
        .unwrap_or(Ipv4Address::new(8, 8, 8, 8));

    let hostname = "api.openai.com";
    let ip = stack.dns_resolve(hostname, dns_server, 5000, get_time_ms, Some(sleep_ms))?;

    // 3. Establish TLS connection
    let mut tls = TlsConnection::connect(
        stack,
        hostname,
        ip,
        443, // HTTPS port
        10_000,
        get_time_ms,
        Some(sleep_ms),
    )?;

    // 4. Send HTTPS request
    let request = format!(
        "GET /v1/models HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: moteOS/0.1.0\r\n\
         Accept: application/json\r\n\
         Connection: close\r\n\
         \r\n",
        hostname
    );

    tls.write(stack, request.as_bytes(), get_time_ms, Some(sleep_ms))?;

    // 5. Read response
    let mut response = alloc::vec::Vec::new();
    let mut buffer = [0u8; 4096];

    loop {
        match tls.read(stack, &mut buffer, get_time_ms, Some(sleep_ms)) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                response.extend_from_slice(&buffer[..n]);
            }
            Err(e) => {
                // Handle error
                return Err(e);
            }
        }

        // Limit response size to prevent memory issues
        if response.len() > 65536 {
            break;
        }
    }

    // 6. Parse HTTP response (simplified)
    if let Ok(response_str) = core::str::from_utf8(&response) {
        // Find the status line
        if let Some(first_line) = response_str.lines().next() {
            // Check if status is 200 OK
            if first_line.contains("200 OK") {
                // Success!
                // In a real application, you would parse the JSON body here
            }
        }
    }

    // 7. Close connection
    tls.close(stack);

    Ok(())
}

/// Example: Test TLS connection to example.com
fn example_simple_https() -> Result<(), network::NetError> {
    // This would normally be initialized from boot
    // For this example, we'll assume the driver is set up

    // Note: You need to create a real driver instance here
    // For example:
    // let driver = virtio::VirtioNetDriver::new()?;
    // let mut stack = NetworkStack::new(Box::new(driver), None)?;

    // Simulated example - in real code, use actual driver
    // let mut stack = create_network_stack()?;

    // example_https_connection(&mut stack)?;

    Ok(())
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Entry point for the example
// Note: This is a no_std environment, so there's no main() function
// In a real moteOS kernel, this would be called from kernel_main()
