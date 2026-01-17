//! Theme system for moteOS TUI
//!
//! Provides dark and light themes with color definitions from the PRD.

#![no_std]

use crate::colors::Color;

/// Theme structure containing all color definitions
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // Background colors
    pub background: Color,
    pub surface: Color,
    pub border: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_disabled: Color,

    // Accent colors
    pub accent_primary: Color,
    pub accent_success: Color,
    pub accent_warning: Color,
    pub accent_error: Color,
    pub accent_assistant: Color,
    pub accent_code: Color,

    // Provider brand colors
    pub provider_openai: Color,
    pub provider_anthropic: Color,
    pub provider_groq: Color,
    pub provider_xai: Color,
    pub provider_local: Color,
}

/// Helper macro to unwrap Color::from_hex at compile time
macro_rules! hex_color {
    ($hex:expr) => {{
        match Color::from_hex($hex) {
            Ok(color) => color,
            Err(_) => panic!("Invalid hex color"),
        }
    }};
}

/// Dark theme color palette (GitHub Dark inspired)
///
/// Background: #0D1117
/// Surface: #161B22
/// Border: #21262D
/// Text Primary: #F0F6FC
/// Text Secondary: #C9D1D9
/// Accent Primary: #58A6FF
/// Accent Success: #7EE787
/// Accent Warning: #FFA657
/// Accent Error: #FF7B72
/// Accent Assistant: #A371F7
pub const DARK_THEME: Theme = Theme {
    // Background colors
    background: hex_color!("#0D1117"),
    surface: hex_color!("#161B22"),
    border: hex_color!("#21262D"),

    // Text colors
    text_primary: hex_color!("#F0F6FC"),
    text_secondary: hex_color!("#C9D1D9"),
    text_tertiary: hex_color!("#8B949E"),
    text_disabled: hex_color!("#484F58"),

    // Accent colors
    accent_primary: hex_color!("#58A6FF"),
    accent_success: hex_color!("#7EE787"),
    accent_warning: hex_color!("#FFA657"),
    accent_error: hex_color!("#FF7B72"),
    accent_assistant: hex_color!("#A371F7"),
    accent_code: hex_color!("#79C0FF"),

    // Provider brand colors
    provider_openai: hex_color!("#10A37F"),
    provider_anthropic: hex_color!("#D4A574"),
    provider_groq: hex_color!("#F55036"),
    provider_xai: hex_color!("#FFFFFF"),
    provider_local: hex_color!("#7C3AED"),
};

/// Light theme color palette (GitHub Light inspired)
///
/// Background: #FFFFFF
/// Surface: #F6F8FA
/// Border: #D0D7DE
/// Text Primary: #1F2328
/// Text Secondary: #424A53
/// Accent Primary: #0969DA
/// Accent Success: #1A7F37
/// Accent Warning: #9A6700
/// Accent Error: #CF222E
/// Accent Assistant: #8250DF
pub const LIGHT_THEME: Theme = Theme {
    // Background colors
    background: hex_color!("#FFFFFF"),
    surface: hex_color!("#F6F8FA"),
    border: hex_color!("#D0D7DE"),

    // Text colors
    text_primary: hex_color!("#1F2328"),
    text_secondary: hex_color!("#424A53"),
    text_tertiary: hex_color!("#656D76"),
    text_disabled: hex_color!("#8C959F"),

    // Accent colors
    accent_primary: hex_color!("#0969DA"),
    accent_success: hex_color!("#1A7F37"),
    accent_warning: hex_color!("#9A6700"),
    accent_error: hex_color!("#CF222E"),
    accent_assistant: hex_color!("#8250DF"),
    accent_code: hex_color!("#0550AE"),

    // Provider brand colors
    provider_openai: hex_color!("#10A37F"),
    provider_anthropic: hex_color!("#D4A574"),
    provider_groq: hex_color!("#F55036"),
    provider_xai: hex_color!("#000000"),
    provider_local: hex_color!("#7C3AED"),
};

impl Theme {
    /// Get the appropriate text color based on the background
    pub const fn get_text_color(&self, use_primary: bool) -> Color {
        if use_primary {
            self.text_primary
        } else {
            self.text_secondary
        }
    }

    /// Get provider color by name
    pub const fn get_provider_color(&self, provider: &str) -> Option<Color> {
        match provider {
            "openai" => Some(self.provider_openai),
            "anthropic" => Some(self.provider_anthropic),
            "groq" => Some(self.provider_groq),
            "xai" => Some(self.provider_xai),
            "local" => Some(self.provider_local),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme_colors() {
        // Test background colors
        assert_eq!(DARK_THEME.background, Color::from_hex("#0D1117").unwrap());
        assert_eq!(DARK_THEME.surface, Color::from_hex("#161B22").unwrap());
        assert_eq!(DARK_THEME.border, Color::from_hex("#21262D").unwrap());

        // Test text colors
        assert_eq!(DARK_THEME.text_primary, Color::from_hex("#F0F6FC").unwrap());
        assert_eq!(DARK_THEME.text_secondary, Color::from_hex("#C9D1D9").unwrap());

        // Test accent colors
        assert_eq!(DARK_THEME.accent_primary, Color::from_hex("#58A6FF").unwrap());
        assert_eq!(DARK_THEME.accent_success, Color::from_hex("#7EE787").unwrap());
        assert_eq!(DARK_THEME.accent_warning, Color::from_hex("#FFA657").unwrap());
        assert_eq!(DARK_THEME.accent_error, Color::from_hex("#FF7B72").unwrap());
        assert_eq!(DARK_THEME.accent_assistant, Color::from_hex("#A371F7").unwrap());
    }

    #[test]
    fn test_light_theme_colors() {
        // Test background colors
        assert_eq!(LIGHT_THEME.background, Color::from_hex("#FFFFFF").unwrap());
        assert_eq!(LIGHT_THEME.surface, Color::from_hex("#F6F8FA").unwrap());
        assert_eq!(LIGHT_THEME.border, Color::from_hex("#D0D7DE").unwrap());

        // Test text colors
        assert_eq!(LIGHT_THEME.text_primary, Color::from_hex("#1F2328").unwrap());
        assert_eq!(LIGHT_THEME.text_secondary, Color::from_hex("#424A53").unwrap());

        // Test accent colors
        assert_eq!(LIGHT_THEME.accent_primary, Color::from_hex("#0969DA").unwrap());
        assert_eq!(LIGHT_THEME.accent_success, Color::from_hex("#1A7F37").unwrap());
        assert_eq!(LIGHT_THEME.accent_warning, Color::from_hex("#9A6700").unwrap());
        assert_eq!(LIGHT_THEME.accent_error, Color::from_hex("#CF222E").unwrap());
        assert_eq!(LIGHT_THEME.accent_assistant, Color::from_hex("#8250DF").unwrap());
    }

    #[test]
    fn test_provider_colors() {
        // Test dark theme provider colors
        assert_eq!(DARK_THEME.provider_openai, Color::from_hex("#10A37F").unwrap());
        assert_eq!(DARK_THEME.provider_anthropic, Color::from_hex("#D4A574").unwrap());
        assert_eq!(DARK_THEME.provider_groq, Color::from_hex("#F55036").unwrap());
        assert_eq!(DARK_THEME.provider_xai, Color::from_hex("#FFFFFF").unwrap());
        assert_eq!(DARK_THEME.provider_local, Color::from_hex("#7C3AED").unwrap());

        // Test light theme provider colors (same as dark except xAI)
        assert_eq!(LIGHT_THEME.provider_xai, Color::from_hex("#000000").unwrap());
    }

    #[test]
    fn test_get_text_color() {
        let primary = DARK_THEME.get_text_color(true);
        let secondary = DARK_THEME.get_text_color(false);

        assert_eq!(primary, DARK_THEME.text_primary);
        assert_eq!(secondary, DARK_THEME.text_secondary);
    }

    #[test]
    fn test_get_provider_color() {
        assert_eq!(
            DARK_THEME.get_provider_color("openai"),
            Some(DARK_THEME.provider_openai)
        );
        assert_eq!(
            DARK_THEME.get_provider_color("anthropic"),
            Some(DARK_THEME.provider_anthropic)
        );
        assert_eq!(DARK_THEME.get_provider_color("invalid"), None);
    }
}
