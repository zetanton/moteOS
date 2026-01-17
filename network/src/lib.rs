#![no_std]

// Network stack for moteOS
// Provides network drivers, TCP/IP stack integration, and network utilities

extern crate alloc;

pub mod dhcp;
pub mod dns;
pub mod drivers;
pub mod error;
pub mod pci;
pub mod stack;
pub mod tls;

// Re-export commonly used types
pub use dhcp::{DhcpState, IpConfig};
pub use dns::{DnsResponse, build_query};
pub use drivers::NetworkDriver;
pub use error::NetError;
pub use stack::{NetworkStack, init_network_stack, get_network_stack, poll_network_stack};
pub use tls::TlsConnection;
