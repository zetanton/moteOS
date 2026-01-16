#![no_std]

// Network stack for moteOS
// Provides network drivers, TCP/IP stack integration, and network utilities

extern crate alloc;

pub mod drivers;
pub mod error;
pub mod pci;
pub mod stack;

// Re-export commonly used types
pub use drivers::NetworkDriver;
pub use error::NetError;
pub use stack::{NetworkStack, init_network_stack, get_network_stack, poll_network_stack};
