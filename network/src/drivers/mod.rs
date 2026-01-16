// Network driver implementations

pub mod virtio;
#[cfg(target_arch = "x86_64")]
pub mod interrupts;

use crate::error::NetError;

/// Trait for network drivers
/// 
/// All network drivers must implement this trait to be used with the network stack.
pub trait NetworkDriver: Send {
    /// Send a raw Ethernet frame
    /// 
    /// # Arguments
    /// * `packet` - The Ethernet frame to send (including Ethernet header)
    /// 
    /// # Returns
    /// * `Ok(())` if the packet was successfully queued for transmission
    /// * `Err(NetError)` if transmission failed
    fn send(&mut self, packet: &[u8]) -> Result<(), NetError>;
    
    /// Receive a raw Ethernet frame (non-blocking)
    /// 
    /// # Returns
    /// * `Ok(Some(packet))` if a packet was received
    /// * `Ok(None)` if no packet is available
    /// * `Err(NetError)` if an error occurred
    fn receive(&mut self) -> Result<Option<alloc::vec::Vec<u8>>, NetError>;
    
    /// Get the MAC address of the network interface
    /// 
    /// # Returns
    /// The 6-byte MAC address
    fn mac_address(&self) -> [u8; 6];
    
    /// Check if the network link is up
    /// 
    /// # Returns
    /// `true` if the link is up, `false` otherwise
    fn is_link_up(&self) -> bool;
    
    /// Poll for new packets (must be called regularly)
    /// 
    /// This function should be called periodically to:
    /// - Process received packets
    /// - Handle interrupts
    /// - Update link status
    /// 
    /// # Returns
    /// * `Ok(())` if polling succeeded
    /// * `Err(NetError)` if an error occurred
    fn poll(&mut self) -> Result<(), NetError>;
}
