//! Widget implementations
//!
//! This module contains the built-in widgets for the TUI framework.

pub mod input;

// Re-export the Widget trait for convenience
pub use crate::widget::Widget;

// Re-export widgets
pub use input::InputWidget;
