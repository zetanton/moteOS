#![no_std]

// smoltcp network stack integration
// Provides TCP/IP networking using the smoltcp library

extern crate alloc;

use crate::drivers::NetworkDriver;
use crate::error::NetError;
use alloc::boxed::Box;
use alloc::vec::Vec;
use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};
use spin::Mutex;

/// Device wrapper that adapts our NetworkDriver trait to smoltcp's Device trait
struct DeviceWrapper {
    driver: Box<dyn NetworkDriver>,
}

impl DeviceWrapper {
    fn new(driver: Box<dyn NetworkDriver>) -> Self {
        Self { driver }
    }
}

/// RX token implementation for smoltcp
struct RxTokenWrapper {
    buffer: Vec<u8>,
}

impl RxToken for RxTokenWrapper {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.buffer)
    }
}

/// TX token implementation for smoltcp
struct TxTokenWrapper<'a> {
    driver: &'a mut Box<dyn NetworkDriver>,
}

impl<'a> TxToken for TxTokenWrapper<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = Vec::with_capacity(len);
        buffer.resize(len, 0);
        let result = f(&mut buffer);

        // Send the packet through the driver
        // Ignore errors here as smoltcp doesn't have a good way to propagate them
        let _ = self.driver.send(&buffer);

        result
    }
}

impl Device for DeviceWrapper {
    type RxToken<'a> = RxTokenWrapper where Self: 'a;
    type TxToken<'a> = TxTokenWrapper<'a> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // Try to receive a packet from the driver
        match self.driver.receive() {
            Ok(Some(packet)) => {
                Some((
                    RxTokenWrapper { buffer: packet },
                    TxTokenWrapper {
                        driver: &mut self.driver,
                    },
                ))
            }
            Ok(None) => None,
            Err(_) => None,
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        // Always allow transmission
        Some(TxTokenWrapper {
            driver: &mut self.driver,
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1526; // Standard Ethernet MTU
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}

/// Network stack using smoltcp
///
/// This struct provides TCP/IP networking functionality by integrating
/// smoltcp with our network drivers.
pub struct NetworkStack {
    /// smoltcp interface
    iface: Interface,
    /// Socket set for TCP/UDP sockets
    sockets: SocketSet<'static>,
    /// Device wrapper
    device: DeviceWrapper,
}

impl NetworkStack {
    /// Create a new NetworkStack instance
    ///
    /// # Arguments
    /// * `driver` - Network driver implementing the NetworkDriver trait
    /// * `ip_config` - Optional IP configuration (if None, use 0.0.0.0)
    ///
    /// # Returns
    /// * `Ok(NetworkStack)` - Successfully created network stack
    /// * `Err(NetError)` - Failed to create network stack
    pub fn new(
        driver: Box<dyn NetworkDriver>,
        ip_config: Option<(Ipv4Address, u8)>,
    ) -> Result<Self, NetError> {
        let mac = driver.mac_address();
        let mac_address = EthernetAddress::from_bytes(&mac);

        // Create device wrapper
        let mut device = DeviceWrapper::new(driver);

        // Create interface configuration
        let config = Config::new(HardwareAddress::Ethernet(mac_address));

        // Create interface
        let mut iface = Interface::new(config, &mut device);

        // Configure IP address if provided
        if let Some((ip, prefix_len)) = ip_config {
            iface.update_ip_addrs(|ip_addrs| {
                if ip_addrs.push(IpCidr::new(IpAddress::Ipv4(ip), prefix_len)).is_err() {
                    return Err(NetError::DriverError("Failed to add IP address".to_string()));
                }
                Ok(())
            })?;
        } else {
            // Use 0.0.0.0/0 as default (will need DHCP)
            iface.update_ip_addrs(|ip_addrs| {
                if ip_addrs.push(IpCidr::new(IpAddress::Ipv4(Ipv4Address::UNSPECIFIED), 0)).is_err() {
                    return Err(NetError::DriverError("Failed to add default IP address".to_string()));
                }
                Ok(())
            })?;
        }

        // Create socket set
        let sockets = SocketSet::new(Vec::new());

        Ok(NetworkStack {
            iface,
            sockets,
            device,
        })
    }

    /// Poll the network stack
    ///
    /// This should be called regularly (e.g., every 10ms) to:
    /// - Process incoming packets
    /// - Handle TCP state machine
    /// - Send outgoing packets
    /// - Process timeouts
    ///
    /// # Arguments
    /// * `timestamp` - Current timestamp in milliseconds since boot
    ///
    /// # Returns
    /// * `Ok(())` - Polling succeeded
    /// * `Err(NetError)` - Polling failed
    pub fn poll(&mut self, timestamp_ms: i64) -> Result<(), NetError> {
        // Convert milliseconds to smoltcp Instant
        let timestamp = Instant::from_millis(timestamp_ms);

        // Poll the driver first
        self.device.driver.poll()?;

        // Poll the smoltcp interface
        match self.iface.poll(timestamp, &mut self.device, &mut self.sockets) {
            Ok(_) => Ok(()),
            Err(e) => Err(NetError::DriverError(format!("smoltcp poll error: {:?}", e))),
        }
    }

    /// Get a reference to the interface
    pub fn interface(&self) -> &Interface {
        &self.iface
    }

    /// Get a mutable reference to the interface
    pub fn interface_mut(&mut self) -> &mut Interface {
        &mut self.iface
    }

    /// Get a reference to the socket set
    pub fn sockets(&self) -> &SocketSet<'static> {
        &self.sockets
    }

    /// Get a mutable reference to the socket set
    pub fn sockets_mut(&mut self) -> &mut SocketSet<'static> {
        &mut self.sockets
    }

    /// Get the MAC address of the interface
    pub fn mac_address(&self) -> [u8; 6] {
        self.device.driver.mac_address()
    }

    /// Check if the network link is up
    pub fn is_link_up(&self) -> bool {
        self.device.driver.is_link_up()
    }
}

/// Global network stack instance (protected by mutex)
static NETWORK_STACK: Mutex<Option<NetworkStack>> = Mutex::new(None);

/// Initialize the global network stack
///
/// # Arguments
/// * `driver` - Network driver implementing the NetworkDriver trait
/// * `ip_config` - Optional IP configuration (if None, use 0.0.0.0)
///
/// # Returns
/// * `Ok(())` - Successfully initialized network stack
/// * `Err(NetError)` - Failed to initialize network stack
pub fn init_network_stack(
    driver: Box<dyn NetworkDriver>,
    ip_config: Option<(Ipv4Address, u8)>,
) -> Result<(), NetError> {
    let stack = NetworkStack::new(driver, ip_config)?;

    let mut global = NETWORK_STACK.lock();
    *global = Some(stack);
    Ok(())
}

/// Get the global network stack instance
pub fn get_network_stack() -> spin::MutexGuard<'static, Option<NetworkStack>> {
    NETWORK_STACK.lock()
}

/// Poll the global network stack
///
/// This should be called regularly from the main loop.
///
/// # Arguments
/// * `timestamp_ms` - Current timestamp in milliseconds since boot
pub fn poll_network_stack(timestamp_ms: i64) -> Result<(), NetError> {
    let mut stack = NETWORK_STACK.lock();
    if let Some(ref mut stack) = *stack {
        stack.poll(timestamp_ms)
    } else {
        Err(NetError::DeviceNotInitialized)
    }
}
