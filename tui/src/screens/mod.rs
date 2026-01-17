//! Screen implementations for different application views
//!
//! This module contains full-screen UI implementations like the chat screen,
//! configuration screen, and setup wizard.

pub mod chat;

// Re-export screens
pub use chat::{ChatEvent, ChatScreen, ConnectionStatus};
