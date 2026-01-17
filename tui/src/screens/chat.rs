//! Chat screen implementation
//!
//! Provides a full-screen chat interface with:
//! - Message list with scrolling
//! - Input area
//! - Status bar
//! - Hotkey bar

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::screen::Screen;
use crate::theme::Theme;
use crate::types::{Key, Rect, WidgetEvent};
use crate::widgets::{InputWidget, MessageRole, MessageWidget};

/// Connection status for the chat screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Connected to the LLM provider
    Connected,
    /// Disconnected from the LLM provider
    Disconnected,
    /// Error state with message
    Error(String),
}

/// Events emitted by the chat screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatEvent {
    /// No event
    None,
    /// User submitted a message (content is in the input widget)
    MessageSubmitted,
    /// User wants to scroll up
    ScrollUp,
    /// User wants to scroll down
    ScrollDown,
    /// User wants to go to top
    ScrollToTop,
    /// User wants to go to bottom
    ScrollToBottom,
    /// Custom event
    Custom(&'static str),
}

/// Chat screen with message list, input, status bar, and hotkeys
///
/// Layout:
/// - Header bar: 1 line (title, provider, status)
/// - Chat area: Remaining height - input height - footer height
/// - Input area: 3 lines
/// - Footer/hotkeys: 1 line
pub struct ChatScreen {
    /// List of messages in the conversation
    messages: Vec<MessageWidget>,
    /// Input widget for typing messages
    input: InputWidget,
    /// Scroll offset (number of lines scrolled up from bottom)
    scroll_offset: usize,
    /// Connection status
    status: ConnectionStatus,
    /// Current provider name
    provider: String,
    /// Current model name
    model: String,
    /// Title to display in header
    title: String,
}

impl ChatScreen {
    /// Create a new chat screen
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider name (e.g., "OpenAI")
    /// * `model` - Model name (e.g., "gpt-4o")
    pub fn new(provider: String, model: String) -> Self {
        Self {
            messages: Vec::new(),
            input: InputWidget::new("Type your message...".into()),
            scroll_offset: 0,
            status: ConnectionStatus::Disconnected,
            provider,
            model,
            title: "moteOS Chat".to_string(),
        }
    }

    /// Add a message to the conversation
    ///
    /// # Arguments
    ///
    /// * `role` - The role of the message sender
    /// * `content` - The message content
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        let timestamp = None; // TODO: Get actual timestamp when timer is available
        let message = MessageWidget::new(role, content, timestamp);
        self.messages.push(message);
        // Auto-scroll to bottom when new message is added
        self.scroll_offset = 0;
    }

    /// Update the last message (for streaming responses)
    ///
    /// # Arguments
    ///
    /// * `content` - The updated message content
    pub fn update_last_message(&mut self, content: &str) {
        if let Some(last_msg) = self.messages.last_mut() {
            if last_msg.role == MessageRole::Assistant {
                last_msg.set_content(content.to_string());
            }
        }
    }

    /// Scroll up by one page
    ///
    /// Scrolls up by approximately one screen height worth of messages
    pub fn scroll_up(&mut self) {
        // Scroll by a reasonable amount (e.g., 10 lines)
        self.scroll_offset = self.scroll_offset.saturating_add(10);
    }

    /// Scroll down by one page
    ///
    /// Scrolls down by approximately one screen height worth of messages
    pub fn scroll_down(&mut self) {
        // Scroll by a reasonable amount (e.g., 10 lines)
        let scroll_amount = 10.min(self.scroll_offset);
        self.scroll_offset = self.scroll_offset.saturating_sub(scroll_amount);
    }

    /// Scroll to the top of the message list
    pub fn scroll_to_top(&mut self) {
        // Set to a large value, will be clamped during rendering
        self.scroll_offset = usize::MAX;
    }

    /// Scroll to the bottom of the message list
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// Set the connection status
    ///
    /// # Arguments
    ///
    /// * `status` - The new connection status
    pub fn set_status(&mut self, status: ConnectionStatus) {
        self.status = status;
    }

    /// Get the current connection status
    pub fn status(&self) -> &ConnectionStatus {
        &self.status
    }

    /// Set the provider name
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider name
    pub fn set_provider(&mut self, provider: String) {
        self.provider = provider;
    }

    /// Get the current provider name
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Set the model name
    ///
    /// # Arguments
    ///
    /// * `model` - The model name
    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    /// Get the current model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the input widget (mutable)
    pub fn input_mut(&mut self) -> &mut InputWidget {
        &mut self.input
    }

    /// Get the input widget
    pub fn input(&self) -> &InputWidget {
        &self.input
    }

    /// Handle keyboard input
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was pressed
    ///
    /// # Returns
    ///
    /// A ChatEvent indicating what action should be taken
    pub fn handle_input(&mut self, key: Key) -> ChatEvent {
        // Focus the input widget
        self.input.set_focused(true);

        // Handle input in the input widget
        match self.input.handle_input(key) {
            WidgetEvent::Submit => {
                let text = self.input.get_text().to_string();
                if !text.trim().is_empty() {
                    self.input.clear();
                    return ChatEvent::MessageSubmitted;
                }
                ChatEvent::None
            }
            WidgetEvent::Close => {
                self.input.clear();
                ChatEvent::None
            }
            WidgetEvent::Changed => ChatEvent::None,
            WidgetEvent::None => {
                // Handle scrolling keys when input doesn't consume them
                match key {
                    Key::PageUp => {
                        self.scroll_up();
                        ChatEvent::ScrollUp
                    }
                    Key::PageDown => {
                        self.scroll_down();
                        ChatEvent::ScrollDown
                    }
                    Key::Home => {
                        // Home - scroll to top
                        self.scroll_to_top();
                        ChatEvent::ScrollToTop
                    }
                    Key::End => {
                        // Ctrl+End or just End - scroll to bottom
                        self.scroll_to_bottom();
                        ChatEvent::ScrollToBottom
                    }
                    _ => ChatEvent::None,
                }
            }
            WidgetEvent::Custom(_) => ChatEvent::None,
        }
    }

    /// Render the chat screen to the given screen
    ///
    /// # Arguments
    ///
    /// * `screen` - The screen to render to
    pub fn render(&mut self, screen: &mut Screen) {
        let theme = screen.theme();
        let bounds = screen.bounds();

        // Get character dimensions for layout calculations
        let Some((char_width, char_height)) = screen.char_size() else {
            return; // Can't render without a font
        };

        // Layout constants (from spec: header=1, input=3, footer=1)
        let header_height = char_height;
        let input_height = 3 * char_height;
        let footer_height = char_height;

        // Calculate available height for chat area
        let total_used = header_height + input_height + footer_height;
        let chat_height = bounds.height.saturating_sub(total_used);

        // Layout rectangles
        let header_rect = Rect::new(0, 0, bounds.width, header_height);
        let chat_rect = Rect::new(0, header_height, bounds.width, chat_height);
        let input_rect = Rect::new(0, header_height + chat_height, bounds.width, input_height);
        let footer_rect = Rect::new(
            0,
            header_height + chat_height + input_height,
            bounds.width,
            footer_height,
        );

        // Render header bar
        self.render_header(screen, header_rect, theme);

        // Render message list
        self.render_messages(screen, chat_rect, theme, char_width, char_height);

        // Render input area
        self.input.render(screen, input_rect);

        // Render footer/hotkeys
        self.render_footer(screen, footer_rect, theme, char_width);
    }

    /// Render the header bar with title, provider, and status
    fn render_header(&self, screen: &mut Screen, rect: Rect, theme: &Theme) {
        // Fill header background
        screen.fill_rect(rect, theme.surface);

        // Draw bottom border
        screen.draw_hline(rect.x, rect.y + rect.height - 1, rect.width, theme.border);

        // Get character dimensions
        let Some((char_width, char_height)) = screen.char_size() else {
            return;
        };

        let text_y = rect.y + (char_height / 2);

        // Render title on the left
        let title_x = rect.x + char_width;
        screen.draw_text(title_x, text_y, &self.title, theme.text_primary);

        // Render provider and model in the middle
        let provider_text = format!("{} / {}", self.provider, self.model);
        let provider_text_width = provider_text.chars().count() * char_width;
        let provider_x = rect.x + (rect.width / 2) - (provider_text_width / 2);
        screen.draw_text(provider_x, text_y, &provider_text, theme.text_secondary);

        // Render status on the right
        let status_text = self.format_status();
        let status_color = self.get_status_color(theme);
        let status_text_width = status_text.chars().count() * char_width;
        let status_x = rect.x + rect.width.saturating_sub(status_text_width + char_width);
        screen.draw_text(status_x, text_y, &status_text, status_color);
    }

    /// Render the message list with scrolling
    fn render_messages(
        &self,
        screen: &mut Screen,
        rect: Rect,
        theme: &Theme,
        char_width: usize,
        char_height: usize,
    ) {
        // Clear chat area
        screen.fill_rect(rect, theme.background);

        if self.messages.is_empty() {
            // Show empty state
            let empty_text = "No messages yet. Start a conversation!";
            let empty_text_width = empty_text.chars().count() * char_width;
            let empty_x = rect.x + (rect.width / 2) - (empty_text_width / 2);
            let empty_y = rect.y + (rect.height / 2);
            screen.draw_text(empty_x, empty_y, empty_text, theme.text_tertiary);
            return;
        }

        // Calculate message area width (with padding)
        let message_rect_width = rect.width.saturating_sub(2 * char_width);
        let padding = char_height; // Padding between messages

        // Calculate heights for all messages
        let message_heights: Vec<usize> = self.messages
            .iter()
            .map(|msg| self.estimate_message_height(msg, message_rect_width, char_width, char_height))
            .collect();

        // Calculate total height needed
        let total_height: usize = message_heights.iter().sum::<usize>() + (self.messages.len().saturating_sub(1) * padding);

        // Clamp scroll offset based on available space
        // Scroll offset represents how many "lines" we've scrolled up
        // We'll use a simple approach: scroll by message count
        let max_scroll = if total_height > rect.height {
            // Estimate max scroll in terms of message count
            // This is approximate but works for basic scrolling
            self.messages.len().saturating_sub(1)
        } else {
            0
        };
        self.scroll_offset = self.scroll_offset.min(max_scroll);

        // Render messages from bottom to top
        let mut current_y = rect.y + rect.height;
        let mut messages_skipped = 0;

        // Start from the last message and work backwards
        for (message, &height) in self.messages.iter().zip(message_heights.iter()).rev() {
            // Skip messages based on scroll offset
            if messages_skipped < self.scroll_offset {
                messages_skipped += 1;
                continue;
            }

            // Check if we have space to render this message
            if current_y < rect.y || current_y < rect.y + height {
                break;
            }

            // Position message from bottom
            current_y = current_y.saturating_sub(height + padding);
            let message_rect = Rect::new(
                rect.x + char_width,
                current_y,
                message_rect_width,
                height,
            );

            // Render message
            message.render(screen, message_rect);
        }

        // Show scroll indicator if scrolled
        if self.scroll_offset > 0 {
            let indicator = format!("↑ {} more", self.scroll_offset);
            screen.draw_text(
                rect.x + char_width,
                rect.y + char_height / 2,
                &indicator,
                theme.text_tertiary,
            );
        }
    }

    /// Estimate the height needed for a message
    fn estimate_message_height(
        &self,
        message: &MessageWidget,
        available_width: usize,
        char_width: usize,
        char_height: usize,
    ) -> usize {
        let available_chars = available_width / char_width.max(1);
        let wrapped_lines = MessageWidget::wrap_text(&message.content, available_chars);
        let line_count = wrapped_lines.len().max(1);

        // Add padding: top + bottom + gap for timestamp if present
        let padding = char_height * 2;
        let timestamp_height = if message.timestamp.is_some() {
            char_height
        } else {
            0
        };

        (line_count * char_height) + padding + timestamp_height
    }

    /// Render the footer with hotkeys
    fn render_footer(
        &self,
        screen: &mut Screen,
        rect: Rect,
        theme: &Theme,
        char_width: usize,
    ) {
        // Fill footer background
        screen.fill_rect(rect, theme.surface);

        // Draw top border
        screen.draw_hline(rect.x, rect.y, rect.width, theme.border);

        // Get character dimensions
        let Some((_, char_height)) = screen.char_size() else {
            return;
        };

        let text_y = rect.y + (char_height / 2);

        // Hotkeys to display
        let hotkeys = [
            ("F1", "Help"),
            ("F2", "Provider"),
            ("F3", "Model"),
            ("F4", "Config"),
            ("F9", "New Chat"),
            ("F10", "Quit"),
        ];

        // Render hotkeys
        let mut x = rect.x + char_width;
        for (key, label) in hotkeys.iter() {
            let hotkey_text = format!("{}:{}", key, label);
            screen.draw_text(x, text_y, key, theme.accent_primary);
            x += key.chars().count() * char_width;
            screen.draw_text(x, text_y, ":", theme.text_secondary);
            x += char_width;
            screen.draw_text(x, text_y, label, theme.text_secondary);
            x += label.chars().count() * char_width + char_width * 2; // Spacing
        }
    }

    /// Format the connection status as a string
    fn format_status(&self) -> String {
        match &self.status {
            ConnectionStatus::Connected => "● Connected".to_string(),
            ConnectionStatus::Disconnected => "○ Disconnected".to_string(),
            ConnectionStatus::Error(msg) => {
                // Truncate error message if too long
                if msg.len() > 20 {
                    format!("● Error: {}", &msg[..20])
                } else {
                    format!("● Error: {}", msg)
                }
            }
        }
    }

    /// Get the color for the status indicator
    fn get_status_color(&self, theme: &Theme) -> crate::colors::Color {
        match &self.status {
            ConnectionStatus::Connected => theme.accent_success,
            ConnectionStatus::Disconnected => theme.text_tertiary,
            ConnectionStatus::Error(_) => theme.accent_error,
        }
    }
}

