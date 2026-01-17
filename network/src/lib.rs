#![no_std]

// Network stack for moteOS
// Provides network drivers, TCP/IP stack integration, and network utilities

#[macro_use]
extern crate alloc;

pub mod dhcp;
pub mod dns;
pub mod drivers;
pub mod error;
pub mod http;
pub mod pci;
pub mod stack;
#[cfg(feature = "tls")]
pub mod tls;

// Re-export commonly used types
pub use dhcp::{DhcpState, IpConfig};
pub use dns::{build_query, DnsResponse};
pub use drivers::NetworkDriver;
pub use error::NetError;
pub use http::{parse_url, HttpClient, HttpError, HttpResponse, ParsedUrl, Scheme};
pub use stack::{get_network_stack, init_network_stack, poll_network_stack, NetworkStack};
#[cfg(feature = "tls")]
pub use tls::{set_tls_log_callback, TlsConnection, TlsLogCallback};
