# DHCP Client Implementation

This document describes the DHCP (Dynamic Host Configuration Protocol) client implementation in moteOS.

## Overview

The DHCP client provides automatic IP address configuration for network interfaces. It implements the standard DHCP 4-way handshake to acquire IP configuration from a DHCP server.

## Implementation Details

### Architecture

The DHCP implementation consists of two main components:

1. **`dhcp.rs`** - DHCP-specific types and utilities
   - `IpConfig` - Stores IP configuration (IP, gateway, DNS, subnet mask)
   - `DhcpState` - Enum representing DHCP states
   - Helper functions for creating sockets and extracting configuration

2. **`stack.rs`** - NetworkStack DHCP methods
   - `start_dhcp()` - Initialize DHCP client
   - `dhcp_state()` - Get current DHCP state
   - `dhcp_config()` - Get acquired configuration
   - `dhcp_acquire()` - Convenience method with timeout
   - `apply_dhcp_config()` - Apply configuration to interface
   - `stop_dhcp()` - Stop DHCP client

### DHCP Flow

```
┌─────────────┐         ┌─────────────┐
│   Client    │         │    Server   │
└──────┬──────┘         └──────┬──────┘
       │                       │
       │    DISCOVER (broadcast)
       ├──────────────────────>│
       │                       │
       │    OFFER              │
       │<──────────────────────┤
       │                       │
       │    REQUEST            │
       ├──────────────────────>│
       │                       │
       │    ACK                │
       │<──────────────────────┤
       │                       │
    [Configured]            [Bound]
```

### State Machine

The DHCP client goes through the following states:

- **Init** - Initial state, no DHCP socket created
- **Discovering** - DISCOVER packet sent, waiting for OFFER
- **Requesting** - OFFER received, REQUEST sent, waiting for ACK
- **Configured** - ACK received, IP configuration acquired
- **Renewing** - Lease renewal in progress
- **Rebinding** - Rebinding lease with any server
- **Error** - Error occurred during DHCP process

### Integration with smoltcp

The implementation uses smoltcp's built-in `DhcpSocket` which handles all protocol details:

- Packet formatting (DISCOVER, REQUEST messages)
- Server response parsing (OFFER, ACK messages)
- Option parsing (IP address, gateway, DNS, lease time, etc.)
- Timing and retransmission
- Lease renewal and rebinding

The application layer only needs to:
1. Create and add the DHCP socket
2. Poll the network stack regularly
3. Check for acquired configuration
4. Apply configuration to the interface

## API Reference

### Starting DHCP

```rust
// Method 1: Manual control
stack.start_dhcp()?;
loop {
    stack.poll(timestamp)?;
    if let Some(config) = stack.dhcp_config() {
        stack.apply_dhcp_config(&config)?;
        break;
    }
}

// Method 2: Convenience method with timeout
// Provide a function to get current system time in milliseconds
let config = stack.dhcp_acquire(30_000, || timer::get_ticks_ms())?;
// Configuration is automatically applied
```

### Checking DHCP State

```rust
if let Some(state) = stack.dhcp_state() {
    match state {
        DhcpState::Init => println!("DHCP not started"),
        DhcpState::Discovering => println!("Looking for DHCP server..."),
        DhcpState::Requesting => println!("Requesting IP address..."),
        DhcpState::Configured => println!("IP configured!"),
        DhcpState::Renewing => println!("Renewing lease..."),
        DhcpState::Rebinding => println!("Rebinding lease..."),
        DhcpState::Error => println!("DHCP error"),
    }
}
```

### Accessing Configuration

```rust
if let Some(config) = stack.dhcp_config() {
    println!("IP: {}/{}", config.ip, config.prefix_len);

    if let Some(gateway) = config.gateway {
        println!("Gateway: {}", gateway);
    }

    for dns in &config.dns {
        println!("DNS: {}", dns);
    }
}
```

### Stopping DHCP

```rust
stack.stop_dhcp(); // Removes DHCP socket and stops client
```

## Configuration Applied

When DHCP configuration is acquired, the following are configured:

1. **IP Address** - Set as the primary address on the interface
2. **Subnet Mask** - Stored as prefix length (e.g., /24)
3. **Default Gateway** - Added as the default IPv4 route
4. **DNS Servers** - Stored in IpConfig for use by DNS resolver

## Error Handling

DHCP-specific errors:

- `NetError::DhcpTimeout` - Configuration not acquired within timeout
- `NetError::DhcpConfigFailed` - Failed to apply configuration
- `NetError::DhcpNotConfigured` - Attempted operation requires DHCP configuration

## Performance Considerations

- **Polling Frequency**: Poll at least every 10ms for responsive DHCP
- **Timeout**: Typical DHCP timeout is 30-60 seconds
- **Lease Renewal**: smoltcp handles automatic lease renewal
- **Memory**: DHCP socket uses ~1KB of memory
- **Timing**: The `dhcp_acquire()` method requires a function that returns current time in milliseconds. This should read from the system timer (HPET, TSC, or ARM Generic Timer)

## Testing

### Testing with QEMU

1. QEMU provides a built-in DHCP server when using `-netdev user`:
   ```bash
   qemu-system-x86_64 \
       -netdev user,id=net0 \
       -device virtio-net,netdev=net0
   ```

2. The DHCP server typically assigns:
   - IP: 10.0.2.15/24
   - Gateway: 10.0.2.2
   - DNS: 10.0.2.3

### Testing with Real Network

1. Boot moteOS on hardware with Ethernet connection
2. DHCP will automatically acquire configuration from network DHCP server
3. Common DHCP servers:
   - Home routers
   - Corporate DHCP servers
   - ISC DHCP Server (Linux)
   - Windows DHCP Server

## Compliance

The implementation complies with:

- RFC 2131 - Dynamic Host Configuration Protocol
- RFC 2132 - DHCP Options and BOOTP Vendor Extensions

Supported DHCP options:
- Option 1: Subnet Mask
- Option 3: Router (Gateway)
- Option 6: Domain Name Server
- Option 51: IP Address Lease Time
- Option 53: DHCP Message Type
- Option 54: Server Identifier
- Option 58: Renewal Time Value
- Option 59: Rebinding Time Value

## Future Enhancements

Potential improvements:

- [ ] DHCPv6 support (IPv6)
- [ ] Option 121: Classless Static Route
- [ ] Option 119: Domain Search List
- [ ] DHCP Inform for stateless configuration
- [ ] Rapid Commit (RFC 4039)
- [ ] DHCP authentication (RFC 3118)
- [ ] Lease persistence across reboots

## References

- [Technical Specifications](../../docs/TECHNICAL_SPECIFICATIONS.md) - Section 3.2.7
- [smoltcp Documentation](https://docs.rs/smoltcp/) - DHCPv4 socket
- [RFC 2131](https://tools.ietf.org/html/rfc2131) - DHCP specification
- [RFC 2132](https://tools.ietf.org/html/rfc2132) - DHCP options

## Example Usage

See [`examples/dhcp_usage.rs`](examples/dhcp_usage.rs) for complete examples of using the DHCP client.
