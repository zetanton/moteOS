//! Color system for moteOS TUI
//!
//! Provides 24-bit RGB color support with hex color parsing.

#![no_std]

/// Represents a 24-bit RGB color with alpha channel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color from RGB components (alpha defaults to 255)
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color from RGBA components
    pub const fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a hex color string (e.g., "#RRGGBB" or "#RGB")
    ///
    /// # Examples
    /// ```
    /// let color = Color::from_hex("#FF0000").unwrap(); // Red
    /// let color = Color::from_hex("#F00").unwrap();    // Red (short form)
    /// ```
    pub const fn from_hex(hex: &str) -> Result<Self, ColorError> {
        let bytes = hex.as_bytes();

        // Check for '#' prefix
        let start = if bytes.len() > 0 && bytes[0] == b'#' {
            1
        } else {
            0
        };

        let len = bytes.len() - start;

        match len {
            // Short form: #RGB
            3 => {
                let r = match Self::parse_hex_digit(bytes[start]) {
                    Ok(v) => v * 17, // 0xF -> 0xFF
                    Err(e) => return Err(e),
                };
                let g = match Self::parse_hex_digit(bytes[start + 1]) {
                    Ok(v) => v * 17,
                    Err(e) => return Err(e),
                };
                let b = match Self::parse_hex_digit(bytes[start + 2]) {
                    Ok(v) => v * 17,
                    Err(e) => return Err(e),
                };
                Ok(Self::new(r, g, b))
            }
            // Long form: #RRGGBB
            6 => {
                let r = match Self::parse_hex_byte(bytes[start], bytes[start + 1]) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                let g = match Self::parse_hex_byte(bytes[start + 2], bytes[start + 3]) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                let b = match Self::parse_hex_byte(bytes[start + 4], bytes[start + 5]) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                Ok(Self::new(r, g, b))
            }
            _ => Err(ColorError::InvalidLength),
        }
    }

    /// Parse a single hex digit (0-9, A-F)
    const fn parse_hex_digit(byte: u8) -> Result<u8, ColorError> {
        match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'A'..=b'F' => Ok(byte - b'A' + 10),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            _ => Err(ColorError::InvalidHexChar),
        }
    }

    /// Parse two hex digits into a byte
    const fn parse_hex_byte(high: u8, low: u8) -> Result<u8, ColorError> {
        let h = match Self::parse_hex_digit(high) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let l = match Self::parse_hex_digit(low) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        Ok((h << 4) | l)
    }

    /// Convert color to RGB tuple
    pub const fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Convert color to RGBA tuple
    pub const fn to_rgba(&self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    /// Blend this color with another using alpha blending
    pub fn blend(&self, other: Color, alpha: f32) -> Color {
        let alpha = alpha.clamp(0.0, 1.0);
        let inv_alpha = 1.0 - alpha;

        Color {
            r: (self.r as f32 * inv_alpha + other.r as f32 * alpha) as u8,
            g: (self.g as f32 * inv_alpha + other.g as f32 * alpha) as u8,
            b: (self.b as f32 * inv_alpha + other.b as f32 * alpha) as u8,
            a: (self.a as f32 * inv_alpha + other.a as f32 * alpha) as u8,
        }
    }
}

/// Error type for color parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorError {
    /// Invalid hex character
    InvalidHexChar,
    /// Invalid hex string length
    InvalidLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex_long_form() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_from_hex_short_form() {
        let color = Color::from_hex("#F00").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_from_hex_no_prefix() {
        let color = Color::from_hex("00FF00").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_from_hex_lowercase() {
        let color = Color::from_hex("#a371f7").unwrap();
        assert_eq!(color.r, 0xA3);
        assert_eq!(color.g, 0x71);
        assert_eq!(color.b, 0xF7);
    }

    #[test]
    fn test_blend() {
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        let gray = black.blend(white, 0.5);

        assert!(gray.r > 120 && gray.r < 135);
        assert!(gray.g > 120 && gray.g < 135);
        assert!(gray.b > 120 && gray.b < 135);
    }
}
