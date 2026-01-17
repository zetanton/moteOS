//! TUI Framework for moteOS
//!
//! Provides framebuffer rendering, color system, themes, and widgets for
//! building the terminal user interface.

#![no_std]

pub mod colors;
pub mod font;
pub mod framebuffer;
pub mod screen;
pub mod screens;
pub mod theme;
pub mod types;
pub mod widget;
pub mod widgets;

// Re-export commonly used types
pub use colors::{Color, ColorError};
pub use framebuffer::{Framebuffer, FramebufferInfo, PixelFormat};
pub use screen::{BoxStyle, Screen};
pub use theme::{Theme, DARK_THEME, LIGHT_THEME};
pub use types::{CursorDirection, Key, Point, Rect, WidgetEvent};
pub use widget::Widget;
pub use widgets::{InputWidget, MessageRole, MessageWidget};
pub use screens::{ChatEvent, ChatScreen, ConnectionStatus};
