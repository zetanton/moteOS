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

    #[error("DHCP timeout: {0}")]
    DhcpTimeout(String),

    #[error("DHCP configuration failed: {0}")]
    DhcpConfigFailed(String),

    #[error("DHCP not configured")]
    DhcpNotConfigured,

    #[error("DNS error: {0}")]
    DnsError(String),

    #[error("DNS timeout")]
    DnsTimeout,

    #[error("DNS malformed response: {0}")]
    DnsMalformedResponse(String),

    #[error("DNS name not found")]
    DnsNameNotFound,

    #[error("DNS server failure")]
    DnsServerFailure,

    #[error("TLS error: {0}")]
    TlsError(String),

    #[error("TLS handshake failed: {0}")]
    TlsHandshakeFailed(String),

    #[error("TLS certificate verification failed: {0}")]
    TlsCertificateError(String),

    #[error("TLS invalid server name: {0}")]
    TlsInvalidServerName(String),

    #[error("TLS unsupported cipher suite")]
    TlsUnsupportedCipherSuite,

    #[error("TLS connection closed")]
    TlsConnectionClosed,

    #[error("TLS protocol error: {0}")]
    TlsProtocolError(String),

    #[error("TCP connection failed: {0}")]
    TcpConnectionFailed(String),

    #[error("TCP socket not found")]
    TcpSocketNotFound,

    #[error("TCP send buffer full")]
    TcpSendBufferFull,

    #[error("TCP receive error")]
    TcpReceiveError,
}
