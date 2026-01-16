#![no_std]

// smoltcp network stack integration
// Provides TCP/IP networking using the smoltcp library

extern crate alloc;

use crate::dhcp::{self, DhcpState, IpConfig};
use crate::drivers::NetworkDriver;
use crate::error::NetError;
use alloc::boxed::Box;
use alloc::vec::Vec;
use smoltcp::iface::{Config, Interface, Route, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::dhcpv4::{self, Socket as DhcpSocket};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address, Ipv4Cidr};
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
    /// Socket set for TCP/UDP/DHCP sockets
    sockets: SocketSet<'static>,
    /// Device wrapper
    device: DeviceWrapper,
    /// DHCP socket handle (if DHCP is enabled)
    dhcp_handle: Option<smoltcp::iface::SocketHandle>,
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
            dhcp_handle: None,
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

    /// Start DHCP client to acquire IP configuration
    ///
    /// This creates a DHCP socket and initiates the DHCP discovery process.
    /// Call `poll()` regularly and use `dhcp_config()` to check when configuration
    /// is acquired.
    ///
    /// # Returns
    /// * `Ok(())` - DHCP client started successfully
    /// * `Err(NetError)` - Failed to start DHCP client
    pub fn start_dhcp(&mut self) -> Result<(), NetError> {
        // Check if DHCP is already running
        if self.dhcp_handle.is_some() {
            return Ok(());
        }

        // Create DHCP socket
        let dhcp_socket = dhcp::create_socket();

        // Add socket to the socket set
        let dhcp_handle = self.sockets.add(dhcp_socket);
        self.dhcp_handle = Some(dhcp_handle);

        Ok(())
    }

    /// Get the current DHCP state
    ///
    /// # Returns
    /// * `Some(DhcpState)` - Current DHCP state if DHCP is running
    /// * `None` - DHCP is not running
    pub fn dhcp_state(&self) -> Option<DhcpState> {
        self.dhcp_handle.and_then(|handle| {
            self.sockets
                .get::<DhcpSocket>(handle)
                .map(dhcp::socket_to_state)
        })
    }

    /// Get the current DHCP configuration
    ///
    /// # Returns
    /// * `Some(IpConfig)` - IP configuration if DHCP has acquired one
    /// * `None` - No configuration available yet
    pub fn dhcp_config(&self) -> Option<IpConfig> {
        self.dhcp_handle.and_then(|handle| {
            self.sockets
                .get::<DhcpSocket>(handle)
                .and_then(dhcp::extract_config)
        })
    }

    /// Acquire IP configuration from DHCP (blocking with timeout)
    ///
    /// This is a blocking convenience method that:
    /// 1. Starts DHCP if not already running
    /// 2. Polls until configuration is acquired (with timeout)
    /// 3. Applies the configuration to the interface
    ///
    /// **Note**: This method blocks until DHCP completes or timeout occurs.
    /// The caller must provide a time source and optionally a sleep function
    /// to avoid busy-waiting. For non-blocking operation, use start_dhcp() +
    /// poll() + dhcp_config().
    ///
    /// # Arguments
    /// * `timeout_ms` - Maximum time to wait for DHCP in milliseconds
    /// * `get_time_ms` - Function to get current time in milliseconds
    /// * `sleep_ms` - Optional function to sleep/yield (to avoid 100% CPU usage)
    ///
    /// # Returns
    /// * `Ok(IpConfig)` - Successfully acquired configuration
    /// * `Err(NetError)` - Failed to acquire configuration or timeout
    ///
    /// # Examples
    ///
    /// With sleep function (recommended):
    /// ```no_run
    /// # use network::{NetworkStack, IpConfig};
    /// # fn get_system_time_ms() -> i64 { 0 }
    /// # fn sleep_ms(ms: i64) {}
    /// # let mut stack: NetworkStack = todo!();
    /// let config = stack.dhcp_acquire(
    ///     30_000,
    ///     get_system_time_ms,
    ///     Some(sleep_ms)
    /// )?;
    /// # Ok::<(), network::NetError>(())
    /// ```
    ///
    /// Without sleep (spin-wait, high CPU usage):
    /// ```no_run
    /// # use network::{NetworkStack, IpConfig};
    /// # fn get_system_time_ms() -> i64 { 0 }
    /// # let mut stack: NetworkStack = todo!();
    /// let config = stack.dhcp_acquire(30_000, get_system_time_ms, None)?;
    /// # Ok::<(), network::NetError>(())
    /// ```
    pub fn dhcp_acquire<F, S>(
        &mut self,
        timeout_ms: i64,
        mut get_time_ms: F,
        mut sleep_ms: Option<S>,
    ) -> Result<IpConfig, NetError>
    where
        F: FnMut() -> i64,
        S: FnMut(i64),
    {
        // Start DHCP if not already running
        self.start_dhcp()?;

        let start_time = get_time_ms();

        // Poll until we get configuration or timeout
        loop {
            let current_time = get_time_ms();

            // Poll the network stack with current timestamp
            self.poll(current_time)?;

            // Check if we have configuration
            if let Some(config) = self.dhcp_config() {
                // Apply the configuration
                self.apply_dhcp_config(&config)?;
                return Ok(config);
            }

            // Check for timeout
            if current_time - start_time > timeout_ms {
                return Err(NetError::DhcpTimeout(
                    "DHCP configuration not acquired within timeout".into(),
                ));
            }

            // Sleep/yield to avoid 100% CPU usage
            // Recommended: 10ms between polls for responsive DHCP
            if let Some(ref mut sleep_fn) = sleep_ms {
                sleep_fn(10);
            } else {
                // If no sleep function provided, use a minimal CPU hint
                // This is a compiler fence to prevent over-optimization
                core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    /// Apply DHCP configuration to the interface
    ///
    /// This updates the interface with the IP address, gateway, and DNS servers
    /// obtained from DHCP.
    ///
    /// # Arguments
    /// * `config` - IP configuration to apply
    ///
    /// # Returns
    /// * `Ok(())` - Configuration applied successfully
    /// * `Err(NetError)` - Failed to apply configuration
    pub fn apply_dhcp_config(&mut self, config: &IpConfig) -> Result<(), NetError> {
        // Update IP address
        self.iface.update_ip_addrs(|ip_addrs| {
            // Clear existing addresses
            ip_addrs.clear();

            // Add new address from DHCP
            if ip_addrs
                .push(IpCidr::new(
                    IpAddress::Ipv4(config.ip),
                    config.prefix_len,
                ))
                .is_err()
            {
                return Err(NetError::DhcpConfigFailed(
                    "Failed to set IP address".to_string(),
                ));
            }
            Ok(())
        })?;

        // Update default gateway (route)
        if let Some(gateway) = config.gateway {
            self.iface.routes_mut().add_default_ipv4_route(gateway)
                .map_err(|_| NetError::DhcpConfigFailed(
                    "Failed to set default gateway".to_string(),
                ))?;
        }

        // DNS servers are stored in the config but not directly applied to the interface
        // They would be used by a DNS resolver when needed

        Ok(())
    }

    /// Stop DHCP client and remove the DHCP socket
    pub fn stop_dhcp(&mut self) {
        if let Some(handle) = self.dhcp_handle.take() {
            self.sockets.remove(handle);
        }
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
