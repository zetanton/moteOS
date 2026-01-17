//! Widget system for the TUI framework
//!
//! Provides the base Widget trait that all UI components must implement.

use crate::screen::Screen;
use crate::types::{Key, Rect, WidgetEvent};

/// Base trait for all UI widgets
///
/// Widgets are responsible for rendering themselves to a Screen and handling
/// input events. This trait provides the minimal interface needed for the
/// widget system.
pub trait Widget {
    /// Render the widget to the given screen within the specified rectangle
    ///
    /// # Arguments
    ///
    /// * `screen` - The screen to render to
    /// * `rect` - The rectangular region the widget should render within
    fn render(&self, screen: &mut Screen, rect: Rect);

    /// Handle a keyboard input event
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was pressed
    ///
    /// # Returns
    ///
    /// A WidgetEvent indicating what action should be taken
    fn handle_input(&mut self, key: Key) -> WidgetEvent;

    /// Get the preferred size of the widget as (width, height)
    ///
    /// This is a hint for layout systems. Returning (0, 0) means
    /// the widget has no preferred size and will use whatever space
    /// is allocated to it.
    fn size_hint(&self) -> (usize, usize);
}
