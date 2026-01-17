//! Color rendering tests for the TUI framework
//!
//! These tests verify that colors are correctly parsed and that the theme
//! constants contain the expected values from the PRD.

use tui::{Color, DARK_THEME, LIGHT_THEME};

#[test]
fn test_dark_theme_background_colors() {
    // Verify all background colors match the PRD specification
    assert_eq!(
        DARK_THEME.background,
        Color::from_hex("#0D1117").unwrap(),
        "Dark theme background color mismatch"
    );
    assert_eq!(
        DARK_THEME.surface,
        Color::from_hex("#161B22").unwrap(),
        "Dark theme surface color mismatch"
    );
    assert_eq!(
        DARK_THEME.border,
        Color::from_hex("#21262D").unwrap(),
        "Dark theme border color mismatch"
    );
}

#[test]
fn test_dark_theme_text_colors() {
    // Verify all text colors match the PRD specification
    assert_eq!(
        DARK_THEME.text_primary,
        Color::from_hex("#F0F6FC").unwrap(),
        "Dark theme primary text color mismatch"
    );
    assert_eq!(
        DARK_THEME.text_secondary,
        Color::from_hex("#C9D1D9").unwrap(),
        "Dark theme secondary text color mismatch"
    );
}

#[test]
fn test_dark_theme_accent_colors() {
    // Verify all accent colors match the PRD specification
    assert_eq!(
        DARK_THEME.accent_primary,
        Color::from_hex("#58A6FF").unwrap(),
        "Dark theme primary accent color mismatch"
    );
    assert_eq!(
        DARK_THEME.accent_success,
        Color::from_hex("#7EE787").unwrap(),
        "Dark theme success accent color mismatch"
    );
    assert_eq!(
        DARK_THEME.accent_warning,
        Color::from_hex("#FFA657").unwrap(),
        "Dark theme warning accent color mismatch"
    );
    assert_eq!(
        DARK_THEME.accent_error,
        Color::from_hex("#FF7B72").unwrap(),
        "Dark theme error accent color mismatch"
    );
    assert_eq!(
        DARK_THEME.accent_assistant,
        Color::from_hex("#A371F7").unwrap(),
        "Dark theme assistant accent color mismatch"
    );
}

#[test]
fn test_dark_theme_provider_colors() {
    // Verify all provider brand colors match the PRD specification
    assert_eq!(
        DARK_THEME.provider_openai,
        Color::from_hex("#10A37F").unwrap(),
        "OpenAI brand color mismatch"
    );
    assert_eq!(
        DARK_THEME.provider_anthropic,
        Color::from_hex("#D4A574").unwrap(),
        "Anthropic brand color mismatch"
    );
    assert_eq!(
        DARK_THEME.provider_groq,
        Color::from_hex("#F55036").unwrap(),
        "Groq brand color mismatch"
    );
    assert_eq!(
        DARK_THEME.provider_xai,
        Color::from_hex("#FFFFFF").unwrap(),
        "xAI brand color mismatch"
    );
    assert_eq!(
        DARK_THEME.provider_local,
        Color::from_hex("#7C3AED").unwrap(),
        "Local brand color mismatch"
    );
}

#[test]
fn test_light_theme_background_colors() {
    // Verify all background colors match the PRD specification
    assert_eq!(
        LIGHT_THEME.background,
        Color::from_hex("#FFFFFF").unwrap(),
        "Light theme background color mismatch"
    );
    assert_eq!(
        LIGHT_THEME.surface,
        Color::from_hex("#F6F8FA").unwrap(),
        "Light theme surface color mismatch"
    );
    assert_eq!(
        LIGHT_THEME.border,
        Color::from_hex("#D0D7DE").unwrap(),
        "Light theme border color mismatch"
    );
}

#[test]
fn test_light_theme_text_colors() {
    // Verify all text colors match the PRD specification
    assert_eq!(
        LIGHT_THEME.text_primary,
        Color::from_hex("#1F2328").unwrap(),
        "Light theme primary text color mismatch"
    );
    assert_eq!(
        LIGHT_THEME.text_secondary,
        Color::from_hex("#424A53").unwrap(),
        "Light theme secondary text color mismatch"
    );
}

#[test]
fn test_light_theme_accent_colors() {
    // Verify all accent colors match the PRD specification
    assert_eq!(
        LIGHT_THEME.accent_primary,
        Color::from_hex("#0969DA").unwrap(),
        "Light theme primary accent color mismatch"
    );
    assert_eq!(
        LIGHT_THEME.accent_success,
        Color::from_hex("#1A7F37").unwrap(),
        "Light theme success accent color mismatch"
    );
    assert_eq!(
        LIGHT_THEME.accent_warning,
        Color::from_hex("#9A6700").unwrap(),
        "Light theme warning accent color mismatch"
    );
    assert_eq!(
        LIGHT_THEME.accent_error,
        Color::from_hex("#CF222E").unwrap(),
        "Light theme error accent color mismatch"
    );
    assert_eq!(
        LIGHT_THEME.accent_assistant,
        Color::from_hex("#8250DF").unwrap(),
        "Light theme assistant accent color mismatch"
    );
}

#[test]
fn test_color_rgb_values() {
    // Test specific RGB values to ensure correct parsing
    let color = DARK_THEME.background;
    assert_eq!(color.r, 0x0D, "Background red component mismatch");
    assert_eq!(color.g, 0x11, "Background green component mismatch");
    assert_eq!(color.b, 0x17, "Background blue component mismatch");

    let accent = DARK_THEME.accent_assistant;
    assert_eq!(accent.r, 0xA3, "Accent red component mismatch");
    assert_eq!(accent.g, 0x71, "Accent green component mismatch");
    assert_eq!(accent.b, 0xF7, "Accent blue component mismatch");
}

#[test]
fn test_color_contrast() {
    // Verify that text colors have good contrast with backgrounds
    // Dark theme: light text on dark background
    assert!(DARK_THEME.text_primary.r > 200, "Dark theme text should be light");
    assert!(DARK_THEME.background.r < 50, "Dark theme background should be dark");

    // Light theme: dark text on light background
    assert!(LIGHT_THEME.text_primary.r < 100, "Light theme text should be dark");
    assert!(LIGHT_THEME.background.r > 200, "Light theme background should be light");
}

#[test]
fn test_color_to_rgb() {
    let color = Color::from_hex("#FF8800").unwrap();
    let (r, g, b) = color.to_rgb();

    assert_eq!(r, 0xFF);
    assert_eq!(g, 0x88);
    assert_eq!(b, 0x00);
}

#[test]
fn test_color_to_rgba() {
    let color = Color::new_rgba(255, 128, 0, 200);
    let (r, g, b, a) = color.to_rgba();

    assert_eq!(r, 255);
    assert_eq!(g, 128);
    assert_eq!(b, 0);
    assert_eq!(a, 200);
}
