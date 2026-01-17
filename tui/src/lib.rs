//! TUI Framework for moteOS
//!
//! Provides framebuffer rendering, color system, themes, and widgets for
//! building the terminal user interface.

#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]

pub mod colors;
pub mod theme;

// Re-export commonly used types
pub use colors::{Color, ColorError};
pub use theme::{Theme, DARK_THEME, LIGHT_THEME};
