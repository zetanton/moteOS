#![no_std]

//! DHCP client implementation for moteOS
//!
//! This module provides DHCP (Dynamic Host Configuration Protocol) client
//! functionality using smoltcp's DHCPv4 socket implementation.
//!
//! The DHCP client follows the standard DHCP flow:
//! 1. DISCOVER - Broadcast to find DHCP servers
//! 2. OFFER - Receive IP address offer from server
//! 3. REQUEST - Request the offered IP address
//! 4. ACK - Receive acknowledgment and configuration
//!
//! See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.7 for details.

extern crate alloc;

use alloc::vec::Vec;
use smoltcp::socket::dhcpv4::{Event, Socket};
use smoltcp::wire::Ipv4Address;

pub type DhcpSocket = Socket<'static>;

/// IP configuration obtained from DHCP server
#[derive(Debug, Clone, PartialEq)]
pub struct IpConfig {
    /// Assigned IP address
    pub ip: Ipv4Address,
    /// Gateway/router IP address
    pub gateway: Option<Ipv4Address>,
    /// DNS server addresses
    pub dns: Vec<Ipv4Address>,
    /// Subnet mask (prefix length)
    pub prefix_len: u8,
}

impl IpConfig {
    /// Create a new IpConfig with the given IP address and prefix length
    pub fn new(ip: Ipv4Address, prefix_len: u8) -> Self {
        Self {
            ip,
            gateway: None,
            dns: Vec::new(),
            prefix_len,
        }
    }

    /// Set the gateway address
    pub fn with_gateway(mut self, gateway: Ipv4Address) -> Self {
        self.gateway = Some(gateway);
        self
    }

    /// Add a DNS server address
    pub fn add_dns(&mut self, dns: Ipv4Address) {
        self.dns.push(dns);
    }
}

/// DHCP client state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpState {
    /// Initial state, no configuration
    Init,
    /// Discovering DHCP servers (DISCOVER sent)
    Discovering,
    /// Received OFFER, sending REQUEST
    Requesting,
    /// Configuration acquired (ACK received)
    Configured,
    /// Renewing lease
    Renewing,
    /// Rebinding lease
    Rebinding,
    /// Error occurred
    Error,
}

impl core::fmt::Display for DhcpState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DhcpState::Init => write!(f, "Init"),
            DhcpState::Discovering => write!(f, "Discovering"),
            DhcpState::Requesting => write!(f, "Requesting"),
            DhcpState::Configured => write!(f, "Configured"),
            DhcpState::Renewing => write!(f, "Renewing"),
            DhcpState::Rebinding => write!(f, "Rebinding"),
            DhcpState::Error => write!(f, "Error"),
        }
    }
}

/// Extract IP configuration from DHCP socket after successful acquisition
///
/// # Arguments
/// * `socket` - DHCP socket to extract configuration from
///
/// # Returns
/// * `Some(IpConfig)` - Configuration if available
/// * `None` - No configuration available yet
pub fn extract_config(socket: &mut DhcpSocket) -> Option<IpConfig> {
    match socket.poll()? {
        Event::Configured(config) => {
    // Extract IP address and prefix length
            let ip = config.address.address();
            let prefix_len = config.address.prefix_len();

    let mut ip_config = IpConfig::new(ip, prefix_len);

    // Extract gateway (router)
    if let Some(router) = config.router {
        ip_config.gateway = Some(router);
    }

    // Extract DNS servers
            for dns in config.dns_servers.iter() {
            ip_config.dns.push(*dns);
            }

            Some(ip_config)
        }
        Event::Deconfigured => None,
    }
}

/// Map DHCP socket state to our DhcpState enum
pub fn socket_to_state(socket: &mut DhcpSocket) -> DhcpState {
    match socket.poll() {
        Some(Event::Configured(_)) => DhcpState::Configured,
        Some(Event::Deconfigured) => DhcpState::Init,
        None => DhcpState::Discovering,
    }
}

/// Process DHCP events and update state
///
/// # Arguments
/// * `socket` - DHCP socket to process events from
///
/// # Returns
/// * `Some(IpConfig)` - New configuration if DHCP completed successfully
/// * `None` - No new configuration (still in progress or no change)
pub fn process_events(socket: &mut DhcpSocket) -> Option<IpConfig> {
    extract_config(socket)
}

/// Create a new DHCP socket with default configuration
///
/// # Returns
/// A new DHCP socket ready to be added to the socket set
pub fn create_socket() -> DhcpSocket {
    DhcpSocket::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_config_creation() {
        let ip = Ipv4Address::new(192, 168, 1, 100);
        let config = IpConfig::new(ip, 24);

        assert_eq!(config.ip, ip);
        assert_eq!(config.prefix_len, 24);
        assert_eq!(config.gateway, None);
        assert_eq!(config.dns.len(), 0);
    }

    #[test]
    fn test_ip_config_with_gateway() {
        let ip = Ipv4Address::new(192, 168, 1, 100);
        let gateway = Ipv4Address::new(192, 168, 1, 1);
        let config = IpConfig::new(ip, 24).with_gateway(gateway);

        assert_eq!(config.gateway, Some(gateway));
    }

    #[test]
    fn test_ip_config_add_dns() {
        let ip = Ipv4Address::new(192, 168, 1, 100);
        let mut config = IpConfig::new(ip, 24);

        let dns1 = Ipv4Address::new(8, 8, 8, 8);
        let dns2 = Ipv4Address::new(8, 8, 4, 4);

        config.add_dns(dns1);
        config.add_dns(dns2);

        assert_eq!(config.dns.len(), 2);
        assert_eq!(config.dns[0], dns1);
        assert_eq!(config.dns[1], dns2);
    }

    #[test]
    fn test_dhcp_state_display() {
        assert_eq!(format!("{}", DhcpState::Init), "Init");
        assert_eq!(format!("{}", DhcpState::Discovering), "Discovering");
        assert_eq!(format!("{}", DhcpState::Requesting), "Requesting");
        assert_eq!(format!("{}", DhcpState::Configured), "Configured");
        assert_eq!(format!("{}", DhcpState::Renewing), "Renewing");
        assert_eq!(format!("{}", DhcpState::Rebinding), "Rebinding");
        assert_eq!(format!("{}", DhcpState::Error), "Error");
    }
}
