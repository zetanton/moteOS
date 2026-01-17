//! Theme Showcase Example
//!
//! This example demonstrates how to use the dark and light themes
//! in a moteOS application.

#![no_std]
#![no_main]

use tui::{Color, DARK_THEME, LIGHT_THEME, Theme};

/// Example function showing how to render text with theme colors
fn render_chat_message(theme: &Theme, is_assistant: bool) {
    let text_color = if is_assistant {
        theme.accent_assistant
    } else {
        theme.text_primary
    };

    let background = theme.surface;

    // In actual implementation, you would:
    // 1. Fill a rectangle with the background color
    // 2. Render text with the text_color
    // 3. Add borders using theme.border

    // Example RGB values for dark theme assistant message:
    // text_color = #A371F7 (163, 113, 247)
    // background = #161B22 (22, 27, 34)
}

/// Example function showing provider color usage
fn render_provider_badge(theme: &Theme, provider: &str) {
    let badge_color = match provider {
        "openai" => theme.provider_openai,     // #10A37F
        "anthropic" => theme.provider_anthropic, // #D4A574
        "groq" => theme.provider_groq,         // #F55036
        "xai" => theme.provider_xai,           // #FFFFFF (dark) / #000000 (light)
        "local" => theme.provider_local,       // #7C3AED
        _ => theme.accent_primary,
    };

    // Render a small colored badge next to the provider name
}

/// Example function showing status colors
fn render_status_indicator(theme: &Theme, status: &str) {
    let status_color = match status {
        "success" | "connected" => theme.accent_success,  // #7EE787
        "warning" | "reconnecting" => theme.accent_warning, // #FFA657
        "error" | "disconnected" => theme.accent_error,   // #FF7B72
        _ => theme.text_secondary,
    };

    // Render a status dot or icon with the appropriate color
}

/// Example function showing how to render a chat interface
fn render_chat_interface() {
    let theme = &DARK_THEME;

    // Background (full screen)
    let bg_color = theme.background; // #0D1117

    // Header bar
    let header_bg = theme.surface;   // #161B22
    let header_text = theme.text_primary; // #F0F6FC
    let header_border = theme.border; // #21262D

    // Message area
    let message_bg = theme.background;
    let user_text = theme.text_primary;
    let assistant_text = theme.accent_assistant; // #A371F7
    let timestamp_text = theme.text_secondary; // #C9D1D9

    // Input area
    let input_bg = theme.surface;
    let input_text = theme.text_primary;
    let input_border = theme.accent_primary; // #58A6FF (when focused)

    // In a real implementation, you would use these colors to:
    // 1. Clear screen with bg_color
    // 2. Draw header rectangle with header_bg
    // 3. Draw messages with appropriate colors
    // 4. Draw input box with input_bg
    // 5. Draw borders and separators
}

/// Example showing color blending for transparency effects
fn render_modal_overlay(theme: &Theme) {
    let overlay = Color::new_rgba(0, 0, 0, 128); // Semi-transparent black
    let modal_bg = theme.surface;

    // Blend overlay with background for dimming effect
    let background = theme.background;
    let dimmed = background.blend(overlay, 0.5);

    // Modal would use:
    // - Dimmed background behind it
    // - theme.surface for the modal background
    // - theme.border for modal borders
    // - theme.text_primary for modal text
}

/// Example showing syntax highlighting colors
fn render_code_block(theme: &Theme, code: &str) {
    let code_bg = theme.surface;
    let code_border = theme.border;
    let code_text = theme.accent_code; // #79C0FF

    // In a real implementation, you would:
    // 1. Parse the code into tokens
    // 2. Use different accent colors for different token types:
    //    - Keywords: theme.accent_primary
    //    - Strings: theme.accent_success
    //    - Numbers: theme.accent_warning
    //    - Comments: theme.text_tertiary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_usage() {
        // Verify we can access theme colors
        let dark = &DARK_THEME;
        let light = &LIGHT_THEME;

        // Themes should be different
        assert_ne!(dark.background, light.background);
        assert_ne!(dark.text_primary, light.text_primary);

        // Provider colors should be consistent (except xAI)
        assert_eq!(dark.provider_openai, light.provider_openai);
        assert_eq!(dark.provider_anthropic, light.provider_anthropic);
        assert_eq!(dark.provider_groq, light.provider_groq);
        assert_eq!(dark.provider_local, light.provider_local);

        // xAI should be different (white on dark, black on light)
        assert_ne!(dark.provider_xai, light.provider_xai);
    }

    #[test]
    fn test_color_creation() {
        // Test creating colors directly
        let red = Color::new(255, 0, 0);
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);

        // Test creating colors from hex
        let blue = Color::from_hex("#0000FF").unwrap();
        assert_eq!(blue.r, 0);
        assert_eq!(blue.g, 0);
        assert_eq!(blue.b, 255);
    }
}
