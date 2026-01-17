//! Input widget for text entry with cursor management
//!
//! Provides a text input field with support for character insertion/deletion,
//! cursor movement, and focus management.

extern crate alloc;
use alloc::string::String;

use crate::screen::Screen;
use crate::types::{CursorDirection, Key, Rect, WidgetEvent};
use crate::widget::Widget;

/// Text input widget with cursor support
///
/// This widget displays a text input field with a cursor that can be moved
/// around. It supports character insertion and deletion at the cursor position,
/// and visual feedback when focused.
///
/// # Example
///
/// ```no_run
/// # use tui::widgets::InputWidget;
/// let mut input = InputWidget::new("Enter your message...".into());
/// input.set_focused(true);
/// input.insert_char('H');
/// input.insert_char('i');
/// assert_eq!(input.get_text(), "Hi");
/// ```
pub struct InputWidget {
    /// Current text content
    text: String,
    /// Cursor position (byte offset in the string)
    cursor_pos: usize,
    /// Placeholder text shown when empty
    placeholder: String,
    /// Whether the widget has focus
    focused: bool,
}

impl InputWidget {
    /// Create a new input widget with the given placeholder text
    ///
    /// # Arguments
    ///
    /// * `placeholder` - Text to display when the input is empty
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use tui::widgets::InputWidget;
    /// let input = InputWidget::new("Type here...".into());
    /// ```
    pub fn new(placeholder: String) -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
            placeholder,
            focused: false,
        }
    }

    /// Insert a character at the current cursor position
    ///
    /// The cursor will be moved to after the inserted character.
    ///
    /// # Arguments
    ///
    /// * `ch` - The character to insert
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use tui::widgets::InputWidget;
    /// let mut input = InputWidget::new("".into());
    /// input.insert_char('a');
    /// input.insert_char('b');
    /// assert_eq!(input.get_text(), "ab");
    /// ```
    pub fn insert_char(&mut self, ch: char) {
        // Find the character position (not byte position)
        let char_idx = self.text.chars().take(self.cursor_pos).count();

        // Convert back to byte position for insertion
        let byte_pos = self.text.char_indices()
            .nth(char_idx)
            .map(|(pos, _)| pos)
            .unwrap_or(self.text.len());

        self.text.insert(byte_pos, ch);
        self.cursor_pos += 1;
    }

    /// Delete the character before the cursor (backspace behavior)
    ///
    /// If the cursor is at the beginning, this is a no-op.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use tui::widgets::InputWidget;
    /// let mut input = InputWidget::new("".into());
    /// input.insert_char('a');
    /// input.insert_char('b');
    /// input.delete_char();
    /// assert_eq!(input.get_text(), "a");
    /// ```
    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 && !self.text.is_empty() {
            let char_idx = self.cursor_pos - 1;

            // Find the byte position of the character to remove
            if let Some((byte_pos, ch)) = self.text.char_indices().nth(char_idx) {
                self.text.remove(byte_pos);
                self.cursor_pos -= 1;
            }
        }
    }

    /// Delete the character at the cursor position (delete key behavior)
    ///
    /// If the cursor is at the end, this is a no-op.
    pub fn delete_char_forward(&mut self) {
        if self.cursor_pos < self.text.chars().count() {
            let char_idx = self.cursor_pos;

            // Find the byte position of the character to remove
            if let Some((byte_pos, _)) = self.text.char_indices().nth(char_idx) {
                self.text.remove(byte_pos);
                // Cursor position stays the same
            }
        }
    }

    /// Move the cursor in the specified direction
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction to move the cursor
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use tui::widgets::InputWidget;
    /// # use tui::types::CursorDirection;
    /// let mut input = InputWidget::new("".into());
    /// input.insert_char('a');
    /// input.insert_char('b');
    /// input.move_cursor(CursorDirection::Left);
    /// input.insert_char('x');
    /// assert_eq!(input.get_text(), "axb");
    /// ```
    pub fn move_cursor(&mut self, direction: CursorDirection) {
        let char_count = self.text.chars().count();

        match direction {
            CursorDirection::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            CursorDirection::Right => {
                if self.cursor_pos < char_count {
                    self.cursor_pos += 1;
                }
            }
            CursorDirection::Start => {
                self.cursor_pos = 0;
            }
            CursorDirection::End => {
                self.cursor_pos = char_count;
            }
            // Up/Down don't make sense for single-line input
            CursorDirection::Up | CursorDirection::Down => {}
        }
    }

    /// Get the current text content
    ///
    /// # Returns
    ///
    /// A string slice of the current text
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Clear all text from the input
    ///
    /// This resets the text to an empty string and moves the cursor to the beginning.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
    }

    /// Set the focus state of the widget
    ///
    /// # Arguments
    ///
    /// * `focused` - Whether the widget should be focused
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Check if the widget is focused
    ///
    /// # Returns
    ///
    /// `true` if the widget is focused, `false` otherwise
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Set the text content directly
    ///
    /// This replaces the current text and moves the cursor to the end.
    ///
    /// # Arguments
    ///
    /// * `text` - The new text content
    pub fn set_text(&mut self, text: String) {
        self.cursor_pos = text.chars().count();
        self.text = text;
    }

    /// Get the cursor position as a character index
    ///
    /// # Returns
    ///
    /// The cursor position (0-indexed character position)
    pub fn cursor_position(&self) -> usize {
        self.cursor_pos
    }
}

impl Widget for InputWidget {
    fn render(&self, screen: &mut Screen, rect: Rect) {
        let theme = screen.theme();

        // Determine colors based on focus state
        let bg_color = if self.focused {
            theme.surface
        } else {
            theme.background
        };
        let text_color = if self.focused {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        let border_color = if self.focused {
            theme.accent_primary
        } else {
            theme.border
        };

        // Clear the input area with background color
        screen.clear_rect(rect, bg_color);

        // Draw border (simple line at bottom)
        if rect.height > 0 {
            screen.draw_hline(rect.x, rect.y + rect.height - 1, rect.width, border_color);
        }

        // Calculate text rendering position (with some padding)
        let text_y = rect.y + 1;
        let text_x = rect.x + 2;

        // Render text or placeholder
        if self.text.is_empty() {
            // Show placeholder in a dimmer color
            screen.draw_text(text_x, text_y, &self.placeholder, theme.text_tertiary);
        } else {
            screen.draw_text(text_x, text_y, &self.text, text_color);
        }

        // Draw cursor if focused
        if self.focused {
            // Calculate cursor pixel position
            if let Some((char_width, char_height)) = screen.char_size() {
                let cursor_x = text_x + (self.cursor_pos * char_width);
                let cursor_y = text_y;

                // Draw cursor as a vertical line
                if cursor_x < rect.x + rect.width {
                    screen.draw_vline(cursor_x, cursor_y, char_height, theme.accent_primary);
                }
            }
        }
    }

    fn handle_input(&mut self, key: Key) -> WidgetEvent {
        match key {
            Key::Char(ch) => {
                self.insert_char(ch);
                WidgetEvent::Changed
            }
            Key::Backspace => {
                self.delete_char();
                WidgetEvent::Changed
            }
            Key::Delete => {
                self.delete_char_forward();
                WidgetEvent::Changed
            }
            Key::Left => {
                self.move_cursor(CursorDirection::Left);
                WidgetEvent::Changed
            }
            Key::Right => {
                self.move_cursor(CursorDirection::Right);
                WidgetEvent::Changed
            }
            Key::Home => {
                self.move_cursor(CursorDirection::Start);
                WidgetEvent::Changed
            }
            Key::End => {
                self.move_cursor(CursorDirection::End);
                WidgetEvent::Changed
            }
            Key::Enter => {
                WidgetEvent::Submit
            }
            Key::Escape => {
                WidgetEvent::Close
            }
            _ => WidgetEvent::None,
        }
    }

    fn size_hint(&self) -> (usize, usize) {
        // Suggest 3 lines tall (text + padding + border)
        // No width preference (will use allocated width)
        (0, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_input() {
        let input = InputWidget::new("placeholder".into());
        assert_eq!(input.get_text(), "");
        assert_eq!(input.cursor_position(), 0);
        assert!(!input.is_focused());
    }

    #[test]
    fn test_insert_char() {
        let mut input = InputWidget::new("".into());
        input.insert_char('a');
        assert_eq!(input.get_text(), "a");
        assert_eq!(input.cursor_position(), 1);

        input.insert_char('b');
        assert_eq!(input.get_text(), "ab");
        assert_eq!(input.cursor_position(), 2);
    }

    #[test]
    fn test_delete_char() {
        let mut input = InputWidget::new("".into());
        input.insert_char('a');
        input.insert_char('b');
        input.delete_char();
        assert_eq!(input.get_text(), "a");
        assert_eq!(input.cursor_position(), 1);
    }

    #[test]
    fn test_delete_char_forward() {
        let mut input = InputWidget::new("".into());
        input.insert_char('a');
        input.insert_char('b');
        input.move_cursor(CursorDirection::Left);
        input.delete_char_forward();
        assert_eq!(input.get_text(), "a");
        assert_eq!(input.cursor_position(), 1);
    }

    #[test]
    fn test_move_cursor() {
        let mut input = InputWidget::new("".into());
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');

        input.move_cursor(CursorDirection::Left);
        assert_eq!(input.cursor_position(), 2);

        input.move_cursor(CursorDirection::Left);
        assert_eq!(input.cursor_position(), 1);

        input.move_cursor(CursorDirection::Right);
        assert_eq!(input.cursor_position(), 2);

        input.move_cursor(CursorDirection::Start);
        assert_eq!(input.cursor_position(), 0);

        input.move_cursor(CursorDirection::End);
        assert_eq!(input.cursor_position(), 3);
    }

    #[test]
    fn test_clear() {
        let mut input = InputWidget::new("".into());
        input.insert_char('a');
        input.insert_char('b');
        input.clear();
        assert_eq!(input.get_text(), "");
        assert_eq!(input.cursor_position(), 0);
    }

    #[test]
    fn test_set_text() {
        let mut input = InputWidget::new("".into());
        input.set_text("hello".into());
        assert_eq!(input.get_text(), "hello");
        assert_eq!(input.cursor_position(), 5);
    }

    #[test]
    fn test_focus() {
        let mut input = InputWidget::new("".into());
        assert!(!input.is_focused());

        input.set_focused(true);
        assert!(input.is_focused());

        input.set_focused(false);
        assert!(!input.is_focused());
    }

    #[test]
    fn test_insert_at_middle() {
        let mut input = InputWidget::new("".into());
        input.insert_char('a');
        input.insert_char('c');
        input.move_cursor(CursorDirection::Left);
        input.insert_char('b');
        assert_eq!(input.get_text(), "abc");
    }
}
