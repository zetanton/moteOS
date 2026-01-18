//! Message widget for rendering user and assistant message bubbles
//!
//! Provides text wrapping, timestamp display, and distinct styling
//! for user and assistant messages.

extern crate alloc;

use crate::colors::Color;
use crate::screen::{BoxStyle, Screen};
use crate::theme::Theme;
use crate::types::{Key, Rect, WidgetEvent};
use crate::widget::Widget;

use alloc::string::String;
use alloc::vec::Vec;

/// Message role indicating who sent the message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    /// Message from the user
    User,
    /// Message from the assistant
    Assistant,
    /// System message (help, notifications, etc.)
    System,
}

/// Message widget for displaying chat messages
///
/// Supports text wrapping, timestamp display, and distinct styling
/// for user and assistant messages.
pub struct MessageWidget {
    /// The role of the message sender
    pub role: MessageRole,
    /// The message content
    pub content: String,
    /// Optional timestamp in seconds since epoch
    pub timestamp: Option<u64>,
}

impl MessageWidget {
    /// Create a new message widget
    ///
    /// # Arguments
    ///
    /// * `role` - The role of the message sender
    /// * `content` - The message content
    /// * `timestamp` - Optional timestamp in seconds since epoch
    pub fn new(role: MessageRole, content: String, timestamp: Option<u64>) -> Self {
        Self {
            role,
            content,
            timestamp,
        }
    }

    /// Update the message content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    /// Format a timestamp as a human-readable string
    ///
    /// Format: "HH:MM:SS" or "HH:MM" if seconds are 0
    fn format_timestamp(timestamp: u64) -> String {
        let hours = timestamp / 3600;
        let minutes = (timestamp % 3600) / 60;
        let seconds = timestamp % 60;

        use alloc::string::{String, ToString};
        
        // Pad number with leading zero if needed
        fn pad_two(n: u64) -> String {
            if n < 10 {
                let mut s = String::from("0");
                s.push_str(&n.to_string());
                s
            } else {
                n.to_string()
            }
        }
        
        let h_padded = pad_two(hours % 24);
        let m_padded = pad_two(minutes);
        let s_padded = pad_two(seconds);

        if seconds == 0 {
            let mut result = h_padded;
            result.push_str(":");
            result.push_str(&m_padded);
            result
        } else {
            let mut result = h_padded;
            result.push_str(":");
            result.push_str(&m_padded);
            result.push_str(":");
            result.push_str(&s_padded);
            result
        }
    }

    /// Wrap text to fit within the given width in characters
    ///
    /// Returns a vector of lines, each line being a string that fits
    /// within the specified width.
    pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return Vec::new();
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in text.split_whitespace() {
            let word_len = word.chars().count();

            // If the word itself is longer than the width, we need to break it
            if word_len > width {
                // First, add the current line if it has content
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                    current_width = 0;
                }

                // Break the long word into chunks
                let mut chars = word.chars();
                let mut chunk = String::new();
                let mut chunk_len = 0;

                while let Some(ch) = chars.next() {
                    if chunk_len >= width {
                        lines.push(chunk);
                        chunk = String::new();
                        chunk_len = 0;
                    }
                    chunk.push(ch);
                    chunk_len += 1;
                }

                if !chunk.is_empty() {
                    current_line = chunk;
                    current_width = chunk_len;
                }
            } else {
                // Check if adding this word would exceed the width
                let space_needed = if current_width > 0 {
                    word_len + 1 // +1 for space
                } else {
                    word_len
                };

                if current_width + space_needed > width {
                    // Start a new line
                    if !current_line.is_empty() {
                        lines.push(current_line);
                        current_line = String::new();
                        current_width = 0;
                    }
                }

                // Add the word to the current line
                if current_width > 0 {
                    current_line.push(' ');
                    current_width += 1;
                }
                current_line.push_str(word);
                current_width += word_len;
            }
        }

        // Add the last line if it has content
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // If no lines were created (empty text), return at least one empty line
        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Get the background color for the message bubble based on role and theme
    fn get_bubble_color(&self, theme: &Theme) -> Color {
        match self.role {
            MessageRole::User => {
                // User messages: slightly lighter than surface
                theme.surface
            }
            MessageRole::Assistant => {
                // Assistant messages: use accent color with reduced opacity
                // Blend accent_assistant with surface
                theme.accent_assistant.blend(theme.surface, 0.15)
            }
            MessageRole::System => {
                // System messages: use a subtle info color
                theme.accent_primary.blend(theme.surface, 0.10)
            }
        }
    }

    /// Get the text color for the message based on role and theme
    fn get_text_color(&self, theme: &Theme) -> Color {
        match self.role {
            MessageRole::User => theme.text_primary,
            MessageRole::Assistant => theme.text_primary,
            MessageRole::System => theme.text_secondary,
        }
    }

    /// Get the timestamp color based on theme
    fn get_timestamp_color(&self, theme: &Theme) -> Color {
        theme.text_tertiary
    }

    /// Render the message widget to the screen
    ///
    /// This method handles:
    /// - Drawing the message bubble background
    /// - Wrapping text to fit within the available width
    /// - Rendering the message content
    /// - Displaying the timestamp if available
    fn render_internal(&self, screen: &mut Screen, rect: Rect, theme: &Theme) {
        // Get character dimensions for text layout
        let Some((char_width, char_height)) = screen.char_size() else {
            return; // Can't render without a font
        };

        // Reduced padding for compact layout (1 char instead of 2)
        let padding = 1; // Padding in characters
        let available_width = if rect.width >= padding * 2 * char_width {
            (rect.width / char_width) - (padding * 2)
        } else {
            0
        };

        // Wrap the text
        let wrapped_lines = Self::wrap_text(&self.content, available_width);

        // Calculate bubble dimensions
        let line_count = wrapped_lines.len();
        let text_height = line_count * char_height;
        let timestamp_height = if self.timestamp.is_some() {
            char_height
        } else {
            0
        };

        // Total height needed: text + timestamp + top/bottom padding (reduced)
        let gap = if self.timestamp.is_some() { char_height / 4 } else { 0 };
        let total_height = text_height + timestamp_height + gap + (padding * 2 * char_height);
        let bubble_rect = Rect::new(
            rect.x,
            rect.y,
            rect.width,
            total_height.min(rect.height),
        );

        // Draw bubble background
        let bubble_color = self.get_bubble_color(theme);
        screen.fill_rect(bubble_rect, bubble_color);

        // Draw border around bubble
        screen.draw_box(bubble_rect, BoxStyle::Single, theme.border);

        // Render text lines
        let text_color = self.get_text_color(theme);
        let text_x = rect.x + (padding * char_width);
        let mut text_y = rect.y + (padding * char_height);

        for line in &wrapped_lines {
            if text_y + char_height > rect.y + rect.height {
                break; // Don't render beyond available space
            }

            screen.draw_text(text_x, text_y, line, text_color);
            text_y += char_height;
        }

        // Render timestamp if available
        if let Some(timestamp) = self.timestamp {
            let timestamp_text = Self::format_timestamp(timestamp);
            let timestamp_color = self.get_timestamp_color(theme);
            let timestamp_x = rect.x + rect.width
                - (timestamp_text.chars().count() * char_width)
                - (padding * char_width);
            let timestamp_y = rect.y + text_height + (padding * char_height) + (gap);

            if timestamp_y + char_height <= rect.y + rect.height {
                screen.draw_text(timestamp_x, timestamp_y, &timestamp_text, timestamp_color);
            }
        }
    }
}

impl Widget for MessageWidget {
    fn render(&self, screen: &mut Screen, rect: Rect) {
        let theme = screen.theme();
        self.render_internal(screen, rect, theme);
    }

    fn handle_input(&mut self, _key: Key) -> WidgetEvent {
        // Message widgets don't handle input
        WidgetEvent::None
    }

    fn size_hint(&self) -> (usize, usize) {
        // Calculate preferred size based on content
        // This is a hint - actual size will be determined by layout
        let content_len = self.content.chars().count();
        let estimated_width = (content_len * 8).min(400); // Rough estimate
        let estimated_height = 50; // Rough estimate for one line + padding

        (estimated_width, estimated_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_wrap_text_simple() {
        let text = "Hello world";
        let lines = MessageWidget::wrap_text(text, 10);
        // "Hello world" (11 chars) with width 10 wraps to 2 lines: "Hello" and "world"
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Hello");
        assert_eq!(lines[1], "world");
    }

    #[test]
    fn test_wrap_text_long() {
        let text = "This is a very long line that should be wrapped";
        let lines = MessageWidget::wrap_text(text, 20);
        assert!(lines.len() > 1);
        assert!(lines[0].chars().count() <= 20);
    }

    #[test]
    fn test_wrap_text_empty() {
        let text = "";
        let lines = MessageWidget::wrap_text(text, 10);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }

    #[test]
    fn test_wrap_text_long_word() {
        let text = "supercalifragilisticexpialidocious";
        let lines = MessageWidget::wrap_text(text, 10);
        // Should break the long word
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_format_timestamp() {
        // Test with seconds
        let timestamp = 3661; // 1 hour, 1 minute, 1 second
        let formatted = MessageWidget::format_timestamp(timestamp);
        assert_eq!(formatted, "01:01:01");

        // Test without seconds
        let timestamp = 3660; // 1 hour, 1 minute
        let formatted = MessageWidget::format_timestamp(timestamp);
        assert_eq!(formatted, "01:01");
    }

    #[test]
    fn test_message_widget_creation() {
        let widget = MessageWidget::new(
            MessageRole::User,
            "Hello".to_string(),
            Some(1234567890),
        );
        assert_eq!(widget.role, MessageRole::User);
        assert_eq!(widget.content, "Hello");
        assert_eq!(widget.timestamp, Some(1234567890));
    }
}
