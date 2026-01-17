//! TLS/HTTPS test integration for kernel
//!
//! This module provides functions to test TLS connections from the kernel
//! with real HTTPS endpoints, including certificate verification logging.

use alloc::format;
use network::{set_tls_log_callback, NetworkStack, TlsConnection};
use smoltcp::wire::Ipv4Address;

/// Test TLS connection to a real HTTPS endpoint
///
/// This function performs a complete TLS 1.3 handshake with certificate
/// verification and sends an HTTPS request to the specified hostname.
///
/// # Arguments
/// * `stack` - Mutable reference to the network stack
/// * `hostname` - Hostname to connect to (e.g., "example.com")
/// * `get_time_ms` - Function to get current time in milliseconds
/// * `sleep_ms` - Optional function to sleep/yield
///
/// # Returns
/// * `Ok(())` - TLS test completed successfully
/// * `Err(NetError)` - TLS test failed
pub fn test_tls_https<F, S>(
    stack: &mut NetworkStack,
    hostname: &str,
    mut get_time_ms: F,
    mut sleep_ms: Option<S>,
) -> Result<(), network::NetError>
where
    F: FnMut() -> i64,
    S: FnMut(i64),
{
    use crate::serial;

    // Set up TLS logging to use kernel serial output
    unsafe {
        set_tls_log_callback(Some(|level: &str, message: &str| {
            serial::println(&format!("[TLS {}] {}", level, message));
        }));
    }

    serial::println(&format!("=== TLS/HTTPS Test Starting ==="));
    serial::println(&format!("Target: https://{}", hostname));

    // 1. Configure network with DHCP (if not already configured)
    serial::println("Acquiring IP address via DHCP...");
    let ip_config = match stack.dhcp_acquire(30_000, &mut get_time_ms, sleep_ms.as_mut()) {
        Ok(config) => {
            serial::println(&format!(
                "DHCP configured: IP={}, Gateway={}, DNS={:?}",
                config.ip, config.gateway, config.dns
            ));
            config
        }
        Err(e) => {
            serial::println(&format!("DHCP failed: {:?}", e));
            return Err(e);
        }
    };

    // 2. Resolve hostname using DNS
    let dns_server = ip_config
        .dns
        .first()
        .copied()
        .unwrap_or(Ipv4Address::new(8, 8, 8, 8));

    serial::println(&format!(
        "Resolving {} using DNS server {}",
        hostname, dns_server
    ));
    let ip = match stack.dns_resolve(hostname, dns_server, 5000, &mut get_time_ms, sleep_ms.as_mut()) {
        Ok(addr) => {
            serial::println(&format!("DNS resolution successful: {} -> {}", hostname, addr));
            addr
        }
        Err(e) => {
            serial::println(&format!("DNS resolution failed: {:?}", e));
            return Err(e);
        }
    };

    // 3. Establish TLS connection
    serial::println(&format!("Connecting to {}:443 via TLS 1.3...", hostname));
    let mut tls = match TlsConnection::connect(
        stack,
        hostname,
        ip,
        443, // HTTPS port
        10_000,
        get_time_ms,
        sleep_ms,
    ) {
        Ok(conn) => {
            serial::println("TLS connection established successfully");
            conn
        }
        Err(e) => {
            serial::println(&format!("TLS connection failed: {:?}", e));
            return Err(e);
        }
    };

    // 4. Send HTTPS request
    let request = format!(
        "GET / HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: moteOS-TLS-Test/1.0\r\n\
         Accept: text/html\r\n\
         Connection: close\r\n\
         \r\n",
        hostname
    );

    serial::println("Sending HTTPS request...");
    match tls.write(stack, request.as_bytes(), &mut get_time_ms, sleep_ms.as_mut()) {
        Ok(bytes_written) => {
            serial::println(&format!("Sent {} bytes", bytes_written));
        }
        Err(e) => {
            serial::println(&format!("Failed to send request: {:?}", e));
            tls.close(stack);
            return Err(e);
        }
    }

    // 5. Read response
    serial::println("Reading HTTPS response...");
    let mut response = alloc::vec::Vec::new();
    let mut buffer = [0u8; 4096];
    let mut total_read = 0;

    loop {
        match tls.read(stack, &mut buffer, &mut get_time_ms, sleep_ms.as_mut()) {
            Ok(0) => {
                serial::println("Connection closed by server");
                break;
            }
            Ok(n) => {
                total_read += n;
                response.extend_from_slice(&buffer[..n]);
                if total_read % 1024 == 0 {
                    serial::println(&format!("Read {} bytes so far...", total_read));
                }
            }
            Err(e) => {
                serial::println(&format!("Read error: {:?}", e));
                tls.close(stack);
                return Err(e);
            }
        }

        // Limit response size to prevent memory issues
        if response.len() > 65536 {
            serial::println("Response size limit reached");
            break;
        }
    }

    serial::println(&format!("Received {} total bytes", total_read));

    // 6. Parse HTTP response (simplified)
    if let Ok(response_str) = core::str::from_utf8(&response) {
        // Find the status line
        if let Some(first_line) = response_str.lines().next() {
            serial::println(&format!("HTTP Status: {}", first_line));
            // Check if status is 200 OK
            if first_line.contains("200") {
                serial::println("✓ HTTPS request completed successfully!");
            } else {
                serial::println(&format!("⚠ Unexpected HTTP status: {}", first_line));
            }
        }

        // Log first few lines of response for debugging
        let preview: alloc::vec::Vec<&str> = response_str.lines().take(5).collect();
        serial::println(&format!("Response preview: {:?}", preview));
    } else {
        serial::println("⚠ Response is not valid UTF-8");
    }

    // 7. Close connection
    serial::println("Closing TLS connection...");
    tls.close(stack);

    serial::println("=== TLS test completed successfully ===");
    Ok(())
}
