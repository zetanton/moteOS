# Network Module

The network module provides TCP/IP networking for moteOS using the smoltcp library. It includes network driver support (virtio-net, e1000, RTL8139) and a complete network stack implementation.

## Features

- **Network Drivers**: Support for virtio-net (QEMU/KVM), e1000, and RTL8139 devices
- **TCP/IP Stack**: Full TCP/IP networking via smoltcp
- **Protocol Support**: IPv4, IPv6, TCP, UDP, ICMP, DHCP, DNS
- **No-std Compatible**: Fully functional in a bare-metal environment
- **Safe Abstractions**: Type-safe wrappers around low-level network operations

## Architecture

```
┌─────────────────────────────────────┐
│      Application Layer              │
│  (HTTP clients, LLM API, etc.)      │
├─────────────────────────────────────┤
│      NetworkStack                   │
│  - Interface management             │
│  - Socket management                │
│  - Polling mechanism                │
├─────────────────────────────────────┤
│      smoltcp Library                │
│  - TCP/IP protocol stack            │
│  - Socket implementations           │
│  - Packet processing                │
├─────────────────────────────────────┤
│      Device Layer                   │
│  - DeviceWrapper (smoltcp adapter)  │
├─────────────────────────────────────┤
│      Network Drivers                │
│  - virtio-net                       │
│  - e1000                            │
│  - RTL8139                          │
└─────────────────────────────────────┘
```

## Components

### 1. NetworkDriver Trait

The `NetworkDriver` trait defines the interface that all network drivers must implement.

### 2. NetworkStack

The `NetworkStack` struct integrates smoltcp with our network drivers.

### 3. Device Wrapper

The `DeviceWrapper` adapts our `NetworkDriver` trait to smoltcp's `Device` trait.

## Implementation Details

### Section 3.2.6 Compliance

This implementation follows the technical specifications in Section 3.2.6:

1. **NetworkStack Structure** ✓
   - Implements NetworkDriver trait integration
   - Sets up smoltcp Interface with proper configuration
   - Manages neighbor cache via smoltcp's built-in mechanisms
   - Handles routing through smoltcp's routing table

2. **Polling Mechanism** ✓
   - Regular polling via `poll()` method
   - Timestamp-based event processing
   - Driver polling integrated with smoltcp polling

3. **Driver Integration** ✓
   - DeviceWrapper adapts NetworkDriver to smoltcp Device trait
   - RxToken/TxToken for zero-copy packet handling
   - Proper error handling and propagation

See examples/basic_usage.rs for detailed usage examples.
