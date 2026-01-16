# Network Stack for moteOS

This crate provides network driver implementations for moteOS, including the virtio-net driver for QEMU/KVM virtual machines.

## Components

### PCI Device Discovery (`pci/mod.rs`)

- Scans PCI bus for devices
- Finds devices by vendor/device ID
- Reads PCI configuration space
- Provides access to Base Address Registers (BARs) and interrupt lines

### virtio-net Driver (`drivers/virtio.rs`)

Implements a complete virtio-net network driver with:

1. **PCI Device Discovery**
   - Scans for virtio-net devices (vendor ID 0x1AF4, device ID 0x1000)
   - Reads PCI configuration space
   - Maps I/O and configuration space

2. **Virtio Feature Negotiation**
   - Reads device features
   - Negotiates supported features (MAC address, status, version 1)
   - Sets device status appropriately

3. **Virtqueue Setup**
   - Allocates and initializes RX and TX virtqueues
   - Sets up descriptor tables, available rings, and used rings
   - Configures queue size and addresses

4. **Packet Transmission**
   - Allocates buffers for outgoing packets
   - Adds packets to TX virtqueue
   - Notifies device of new packets

5. **Packet Reception**
   - Pre-allocates RX buffers
   - Adds buffers to RX virtqueue
   - Processes received packets from used ring
   - Re-cycles buffers back to the queue

6. **Interrupt Handling**
   - Handles virtio device interrupts
   - Processes received packets on interrupt
   - Frees transmitted packet buffers

7. **NetworkDriver Trait Implementation**
   - `send()` - Send Ethernet frames
   - `receive()` - Receive Ethernet frames (non-blocking)
   - `mac_address()` - Get MAC address
   - `is_link_up()` - Check link status
   - `poll()` - Poll for packets and handle completion

## Usage

```rust
use network::drivers::virtio::{init_virtio_net, get_virtio_net};
use network::drivers::NetworkDriver;

// Initialize the driver (call once at boot)
init_virtio_net()?;

// Get the driver instance
let mut driver_guard = get_virtio_net().unwrap();
if let Some(ref mut driver) = driver_guard.as_mut() {
    // Send a packet
    let packet: &[u8] = /* Ethernet frame */;
    driver.send(packet)?;
    
    // Receive packets
    while let Some(packet) = driver.receive()? {
        // Process packet
    }
    
    // Poll for new packets
    driver.poll()?;
    
    // Get MAC address
    let mac = driver.mac_address();
    
    // Check link status
    if driver.is_link_up() {
        // Link is up
    }
}
```

## Integration with Interrupt System

To enable interrupt-driven packet reception, register the interrupt handler:

```rust
use network::drivers::interrupts::register_virtio_net_interrupt;
use network::drivers::virtio::get_virtio_net;

// Get interrupt line from driver
let interrupt_line = {
    let driver_guard = get_virtio_net().unwrap();
    if let Some(ref driver) = driver_guard.as_ref() {
        driver.interrupt_line()
    } else {
        return Err("Driver not initialized");
    }
};

// Register interrupt handler
unsafe {
    register_virtio_net_interrupt(interrupt_line)?;
}
```

## Technical Details

### Virtqueue Structure

Each virtqueue consists of:
- **Descriptor Table**: Array of descriptors pointing to buffers
- **Available Ring**: Ring buffer indicating which descriptors are available to the device
- **Used Ring**: Ring buffer indicating which descriptors have been used by the device

### Memory Management

- Virtqueues must be page-aligned (4096 bytes)
- RX buffers are pre-allocated and recycled
- TX buffers are allocated on-demand and freed after transmission

### Address Translation

Currently uses identity mapping (virtual = physical). In a real implementation with paging, proper page table translation would be needed.

## Status

✅ PCI device discovery
✅ Virtio feature negotiation
✅ Virtqueue setup (RX and TX)
✅ Packet transmission
✅ Packet reception
✅ Interrupt handling infrastructure
✅ NetworkDriver trait implementation

## Future Improvements

- Proper page table translation for virt_to_phys()
- Better buffer tracking and management
- Support for additional virtio features (multiqueue, etc.)
- Integration with smoltcp network stack
- Support for MSI-X interrupts
