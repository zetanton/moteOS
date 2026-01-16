// Error types for network operations

use thiserror::Error;

/// Network-related errors
#[derive(Debug, Error)]
pub enum NetError {
    #[error("Driver error: {0}")]
    DriverError(String),
    
    #[error("PCI error: {0}")]
    PciError(String),
    
    #[error("Virtio error: {0}")]
    VirtioError(String),
    
    #[error("Queue error: {0}")]
    QueueError(String),
    
    #[error("Invalid packet: {0}")]
    InvalidPacket(String),
    
    #[error("Device not found")]
    DeviceNotFound,
    
    #[error("Device not initialized")]
    DeviceNotInitialized,
    
    #[error("Buffer too small")]
    BufferTooSmall,
    
    #[error("Operation not supported")]
    NotSupported,

    #[error("smoltcp error: {0}")]
    SmoltcpError(String),
}
