// Example: Basic NetworkStack usage with virtio-net driver
//
// This example demonstrates how to initialize and use the NetworkStack
// with a virtio-net driver for QEMU/KVM virtual machines.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use network::{
    drivers::virtio::{get_virtio_net, init_virtio_net},
    init_network_stack, poll_network_stack, NetworkStack,
};
use smoltcp::wire::Ipv4Address;

// Example initialization function
// This would be called from kernel_main() after heap initialization
pub fn init_network() -> Result<(), network::NetError> {
    // Step 1: Initialize the virtio-net driver
    // This scans for a virtio-net PCI device and initializes it
    init_virtio_net()?;

    // Step 2: Get the driver instance
    let mut virtio_guard = get_virtio_net().ok_or(network::NetError::DeviceNotFound)?;

    let virtio_driver = virtio_guard
        .take()
        .ok_or(network::NetError::DeviceNotInitialized)?;

    // Step 3: Create a boxed driver for the network stack
    let driver: Box<dyn network::NetworkDriver> = Box::new(virtio_driver);

    // Step 4: Initialize the network stack
    // Option 1: With static IP configuration
    let ip_config = Some((
        Ipv4Address::new(192, 168, 1, 100), // IP address
        24,                                 // Prefix length (netmask)
    ));

    // Option 2: Without IP (for DHCP - to be implemented)
    // let ip_config = None;

    init_network_stack(driver, ip_config)?;

    Ok(())
}

// Example polling function
// This should be called regularly from the main event loop
pub fn network_poll_loop() {
    let mut timestamp_ms: i64 = 0;

    loop {
        // Poll the network stack (should be called every ~10ms)
        if let Err(e) = poll_network_stack(timestamp_ms) {
            // Handle error (log it, display in UI, etc.)
            // For now, just continue
        }

        // Update timestamp (in a real system, this would come from a timer)
        timestamp_ms += 10;

        // Sleep for 10ms (in a real system, use a timer interrupt)
        // sleep_ms(10);
    }
}

// Example: Sending a packet
pub fn send_example() -> Result<(), network::NetError> {
    use network::get_network_stack;

    let mut stack = get_network_stack();
    if let Some(ref mut stack) = *stack {
        // Access the interface and sockets
        let iface = stack.interface_mut();
        let sockets = stack.sockets_mut();

        // Here you would:
        // 1. Create a TCP or UDP socket
        // 2. Add it to the socket set
        // 3. Connect/bind the socket
        // 4. Send/receive data
        // (Implementation depends on your use case)

        Ok(())
    } else {
        Err(network::NetError::DeviceNotInitialized)
    }
}

// Example: Getting network status
pub fn get_status() -> Result<NetworkStatus, network::NetError> {
    use network::get_network_stack;

    let stack = get_network_stack();
    if let Some(ref stack) = *stack {
        let mac = stack.mac_address();
        let link_up = stack.is_link_up();

        Ok(NetworkStatus {
            mac_address: mac,
            link_up,
        })
    } else {
        Err(network::NetError::DeviceNotInitialized)
    }
}

pub struct NetworkStatus {
    pub mac_address: [u8; 6],
    pub link_up: bool,
}
