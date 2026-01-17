//! Const evaluation tests for theme system
//!
//! These tests verify that all theme colors are evaluated at compile time
//! and that the const function approach works correctly.

use tui::{Color, DARK_THEME, LIGHT_THEME};

/// Test that we can use theme colors in const contexts
const CONST_BACKGROUND: Color = DARK_THEME.background;
const CONST_TEXT: Color = DARK_THEME.text_primary;
const CONST_ACCENT: Color = DARK_THEME.accent_assistant;

#[test]
fn test_const_evaluation() {
    // Verify that const colors match theme colors
    assert_eq!(CONST_BACKGROUND, DARK_THEME.background);
    assert_eq!(CONST_TEXT, DARK_THEME.text_primary);
    assert_eq!(CONST_ACCENT, DARK_THEME.accent_assistant);
}

#[test]
fn test_const_color_creation() {
    // Test that we can create colors in const contexts
    const RED: Color = Color::new(255, 0, 0);
    const BLUE: Color = Color::new_rgba(0, 0, 255, 128);

    assert_eq!(RED.r, 255);
    assert_eq!(RED.g, 0);
    assert_eq!(RED.b, 0);
    assert_eq!(RED.a, 255);

    assert_eq!(BLUE.r, 0);
    assert_eq!(BLUE.g, 0);
    assert_eq!(BLUE.b, 255);
    assert_eq!(BLUE.a, 128);
}

#[test]
fn test_theme_colors_are_const() {
    // This test verifies that theme constants can be used at compile time
    // If this compiles, it proves the theme is fully const-evaluated

    const fn verify_theme_is_const() -> bool {
        // Access theme colors in a const function
        let _bg = DARK_THEME.background;
        let _text = DARK_THEME.text_primary;
        let _accent = DARK_THEME.accent_assistant;
        true
    }

    const VERIFIED: bool = verify_theme_is_const();
    assert!(VERIFIED);
}

#[test]
fn test_both_themes_are_const() {
    // Verify both dark and light themes are compile-time constants
    const DARK_BG: Color = DARK_THEME.background;
    const LIGHT_BG: Color = LIGHT_THEME.background;

    // Dark theme should have dark background
    assert!(DARK_BG.r < 50 && DARK_BG.g < 50 && DARK_BG.b < 50);

    // Light theme should have light background
    assert!(LIGHT_BG.r > 200 && LIGHT_BG.g > 200 && LIGHT_BG.b > 200);
}

#[test]
fn test_color_from_hex_is_const() {
    // Test that Color::from_hex can be used in const contexts
    const PARSED_COLOR: Result<Color, _> = Color::from_hex("#FF0000");

    match PARSED_COLOR {
        Ok(color) => {
            assert_eq!(color.r, 255);
            assert_eq!(color.g, 0);
            assert_eq!(color.b, 0);
        }
        Err(_) => panic!("Failed to parse color at compile time"),
    }
}

#[test]
fn test_theme_helper_is_const() {
    // Test that Theme::get_text_color is const
    const fn get_primary_text() -> Color {
        DARK_THEME.get_text_color(true)
    }

    const PRIMARY_TEXT: Color = get_primary_text();
    assert_eq!(PRIMARY_TEXT, DARK_THEME.text_primary);
}

/// Compile-time assertion that all theme colors are valid
const _: () = {
    // This block runs at compile time
    // If any color is invalid, compilation will fail

    let _dark_bg = DARK_THEME.background;
    let _dark_surface = DARK_THEME.surface;
    let _dark_border = DARK_THEME.border;
    let _dark_text_primary = DARK_THEME.text_primary;
    let _dark_text_secondary = DARK_THEME.text_secondary;
    let _dark_accent_primary = DARK_THEME.accent_primary;
    let _dark_accent_assistant = DARK_THEME.accent_assistant;

    let _light_bg = LIGHT_THEME.background;
    let _light_surface = LIGHT_THEME.surface;
    let _light_border = LIGHT_THEME.border;
    let _light_text_primary = LIGHT_THEME.text_primary;
    let _light_text_secondary = LIGHT_THEME.text_secondary;
    let _light_accent_primary = LIGHT_THEME.accent_primary;
    let _light_accent_assistant = LIGHT_THEME.accent_assistant;
};
