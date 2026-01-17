// Error types for network operations

extern crate alloc;

use alloc::string::String;

/// Network-related errors
#[derive(Debug)]
pub enum NetError {
    DriverError(String),

    PciError(String),

    VirtioError(String),

    QueueError(String),

    InvalidPacket(String),

    DeviceNotFound,

    DeviceNotInitialized,

    BufferTooSmall,

    NotSupported,

    SmoltcpError(String),

    DhcpTimeout(String),

    DhcpConfigFailed(String),

    DhcpNotConfigured,

    DnsError(String),

    DnsTimeout,

    DnsMalformedResponse(String),

    DnsNameNotFound,

    DnsServerFailure,

    TlsError(String),

    TlsHandshakeFailed(String),

    TlsCertificateError(String),

    TlsInvalidServerName(String),

    TlsUnsupportedCipherSuite,

    TlsConnectionClosed,

    TlsProtocolError(String),

    TcpConnectionFailed(String),

    TcpSocketNotFound,

    TcpSendBufferFull,

    TcpReceiveError,
}

impl core::fmt::Display for NetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NetError::DriverError(s) => write!(f, "Driver error: {s}"),
            NetError::PciError(s) => write!(f, "PCI error: {s}"),
            NetError::VirtioError(s) => write!(f, "Virtio error: {s}"),
            NetError::QueueError(s) => write!(f, "Queue error: {s}"),
            NetError::InvalidPacket(s) => write!(f, "Invalid packet: {s}"),
            NetError::DeviceNotFound => write!(f, "Device not found"),
            NetError::DeviceNotInitialized => write!(f, "Device not initialized"),
            NetError::BufferTooSmall => write!(f, "Buffer too small"),
            NetError::NotSupported => write!(f, "Operation not supported"),
            NetError::SmoltcpError(s) => write!(f, "smoltcp error: {s}"),
            NetError::DhcpTimeout(s) => write!(f, "DHCP timeout: {s}"),
            NetError::DhcpConfigFailed(s) => write!(f, "DHCP configuration failed: {s}"),
            NetError::DhcpNotConfigured => write!(f, "DHCP not configured"),
            NetError::DnsError(s) => write!(f, "DNS error: {s}"),
            NetError::DnsTimeout => write!(f, "DNS timeout"),
            NetError::DnsMalformedResponse(s) => write!(f, "DNS malformed response: {s}"),
            NetError::DnsNameNotFound => write!(f, "DNS name not found"),
            NetError::DnsServerFailure => write!(f, "DNS server failure"),
            NetError::TlsError(s) => write!(f, "TLS error: {s}"),
            NetError::TlsHandshakeFailed(s) => write!(f, "TLS handshake failed: {s}"),
            NetError::TlsCertificateError(s) => {
                write!(f, "TLS certificate verification failed: {s}")
            }
            NetError::TlsInvalidServerName(s) => write!(f, "TLS invalid server name: {s}"),
            NetError::TlsUnsupportedCipherSuite => write!(f, "TLS unsupported cipher suite"),
            NetError::TlsConnectionClosed => write!(f, "TLS connection closed"),
            NetError::TlsProtocolError(s) => write!(f, "TLS protocol error: {s}"),
            NetError::TcpConnectionFailed(s) => write!(f, "TCP connection failed: {s}"),
            NetError::TcpSocketNotFound => write!(f, "TCP socket not found"),
            NetError::TcpSendBufferFull => write!(f, "TCP send buffer full"),
            NetError::TcpReceiveError => write!(f, "TCP receive error"),
        }
    }
}
