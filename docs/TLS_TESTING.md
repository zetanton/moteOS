# TLS/HTTPS Testing Guide

This document describes how to test TLS 1.3 connections with real HTTPS endpoints in moteOS.

## Overview

The TLS implementation in moteOS supports:
- TLS 1.3 handshake
- Full certificate verification using webpki and Mozilla root CAs
- Hostname verification (SNI)
- Certificate chain validation
- Detailed logging of handshake and verification steps

## Features

### Logging

The TLS module includes comprehensive logging that captures:
- TLS handshake initiation and completion
- Certificate parsing and validation
- Hostname verification
- Certificate chain verification against trusted root CAs
- Signature verification steps

Logs are output via the kernel's serial interface and can be captured when running in QEMU.

## Testing with Real HTTPS Endpoints

### Prerequisites

1. Network stack initialized with a working network driver (e.g., virtio-net)
2. Internet connectivity (via QEMU user networking or bridge)
3. TLS feature enabled in the network crate

### Using the Test Function

The kernel provides a test function in `kernel::tls_test::test_tls_https()`:

```rust
use kernel::tls_test;
use network::NetworkStack;

// Assuming you have a network stack and time functions
let mut stack: NetworkStack = /* ... */;
let mut get_time_ms = || /* get current time */;
let mut sleep_ms = |ms| { /* sleep for ms milliseconds */ };

// Test TLS connection to example.com
test_tls_https(&mut stack, "example.com", get_time_ms, Some(sleep_ms))?;
```

### Running the Test Script

A test script is provided to run TLS tests in QEMU:

```bash
./tools/test-tls.sh
```

This script:
1. Builds the ISO if needed
2. Starts QEMU with network support
3. Captures serial output to `/tmp/moteos-tls-test.log`
4. Analyzes the logs for TLS handshake and certificate verification success

### Expected Output

When the TLS test runs successfully, you should see logs like:

```
[TLS INFO] Starting TLS handshake with example.com
[TLS DEBUG] TLS config created with SNI and RSA signatures enabled
[TLS DEBUG] WebPKI verifier created, hostname verification enabled
[TLS INFO] Initiating TLS handshake...
[TLS INFO] Certificate verification started
[TLS DEBUG] Certificate size: 1234 bytes
[TLS DEBUG] X.509 certificate parsed successfully
[TLS DEBUG] Certificate subject: CN=example.com, ...
[TLS INFO] Verifying hostname: example.com
[TLS INFO] Hostname verification passed
[TLS INFO] Verifying certificate chain against trusted root CAs
[TLS INFO] Certificate chain verification passed
[TLS INFO] TLS handshake completed successfully
```

## Test Endpoints

Recommended test endpoints:
- `example.com` - Simple, well-known HTTPS endpoint
- `httpbin.org` - HTTP testing service with HTTPS support
- `www.google.com` - Large, well-maintained certificate chain

## Certificate Verification Details

The TLS implementation performs the following verification steps:

1. **Certificate Parsing**: Parses the X.509 certificate from the server
2. **Subject Extraction**: Extracts certificate subject information for logging
3. **Hostname Verification**: Verifies the certificate matches the requested hostname
4. **Chain Verification**: Validates the certificate chain against Mozilla root CAs
5. **Signature Verification**: Validates the CertificateVerify signature

All steps are logged with appropriate detail levels (INFO, DEBUG, ERROR).

## Troubleshooting

### Certificate Verification Fails

If certificate verification fails, check:
- System time is correct (certificates have expiration dates)
- Network connectivity is working
- DNS resolution is successful

The current implementation uses a fixed time (year 2030) to avoid expiration issues during development. In production, use actual system time.

### Handshake Timeout

If the TLS handshake times out:
- Check network connectivity
- Verify the endpoint is reachable
- Increase timeout values if needed
- Check that the network stack is being polled regularly

### No Logs Appearing

If TLS logs don't appear:
- Ensure TLS logging callback is set: `network::set_tls_log_callback(Some(callback))`
- Verify the kernel's serial output is being captured
- Check that the TLS feature is enabled in Cargo.toml

## Integration Example

To integrate TLS testing into your kernel initialization:

```rust
use kernel::tls_test;
use network::NetworkStack;

fn kernel_main(boot_info: BootInfo) -> ! {
    // ... initialization code ...
    
    // Initialize network
    if let Ok(mut network) = init_network(&config) {
        // Test TLS connection
        if let Err(e) = test_tls_https(
            &mut network,
            "example.com",
            get_time_ms,
            Some(sleep_ms),
        ) {
            serial::println(&format!("TLS test failed: {:?}", e));
        }
    }
    
    // ... rest of kernel ...
}
```

## Log Analysis

The test script automatically analyzes logs for:
- TLS handshake completion
- Certificate verification success
- HTTPS request success

Check `/tmp/moteos-tls-test.log` for full details.

## Next Steps

- Add support for client certificates
- Implement certificate pinning
- Add TLS session resumption
- Support for additional cipher suites
