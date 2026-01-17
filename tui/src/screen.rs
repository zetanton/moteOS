//! High-level screen interface for widget rendering

use crate::colors::Color;
use crate::font::Font;
use crate::framebuffer::{Framebuffer, FramebufferInfo};
use crate::theme::Theme;
use crate::types::Rect;

/// Box drawing style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxStyle {
    /// Single line borders
    Single,
    /// Double line borders
    Double,
    /// Rounded corners
    Rounded,
    /// No borders
    None,
}

/// Main screen structure for rendering
///
/// Provides a safe, high-level interface to the framebuffer for rendering
/// text, boxes, and widgets.
pub struct Screen {
    framebuffer: Framebuffer,
    font: Option<&'static Font>,
    theme: &'static Theme,
    dirty: bool,
}

impl Screen {
    /// Create a new screen from framebuffer info
    ///
    /// # Safety
    ///
    /// The framebuffer info must point to valid video memory.
    pub unsafe fn new(fb_info: FramebufferInfo, theme: &'static Theme) -> Self {
        Self {
            framebuffer: Framebuffer::new(fb_info),
            font: None,
            theme,
            dirty: true,
        }
    }

    /// Set the font to use for text rendering
    pub fn set_font(&mut self, font: &'static Font) {
        self.font = Some(font);
    }

    /// Get the current theme
    pub const fn theme(&self) -> &'static Theme {
        self.theme
    }

    /// Set the theme
    pub fn set_theme(&mut self, theme: &'static Theme) {
        self.theme = theme;
        self.dirty = true;
    }

    /// Get the screen width in pixels
    pub fn width(&self) -> usize {
        self.framebuffer.width()
    }

    /// Get the screen height in pixels
    pub fn height(&self) -> usize {
        self.framebuffer.height()
    }

    /// Get the screen dimensions as a Rect
    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.width(), self.height())
    }

    /// Mark the screen as dirty (needs redraw)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if the screen needs redraw
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the screen with the theme's background color
    pub fn clear(&mut self) {
        unsafe {
            self.framebuffer.clear(self.theme.background);
        }
        self.dirty = false;
    }

    /// Clear a rectangular region with a color
    pub fn clear_rect(&mut self, rect: Rect, color: Color) {
        unsafe {
            self.framebuffer.fill_rect(rect, color);
        }
        self.dirty = true;
    }

    /// Set a pixel at the given coordinates
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        unsafe {
            self.framebuffer.set_pixel(x, y, color);
        }
        self.dirty = true;
    }

    /// Fill a rectangle with a solid color
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        unsafe {
            self.framebuffer.fill_rect(rect, color);
        }
        self.dirty = true;
    }

    /// Draw a horizontal line
    pub fn draw_hline(&mut self, x: usize, y: usize, width: usize, color: Color) {
        unsafe {
            self.framebuffer.draw_hline(x, y, width, color);
        }
        self.dirty = true;
    }

    /// Draw a vertical line
    pub fn draw_vline(&mut self, x: usize, y: usize, height: usize, color: Color) {
        unsafe {
            self.framebuffer.draw_vline(x, y, height, color);
        }
        self.dirty = true;
    }

    /// Draw a box with the specified style
    pub fn draw_box(&mut self, rect: Rect, style: BoxStyle, color: Color) {
        match style {
            BoxStyle::None => {}
            BoxStyle::Single | BoxStyle::Double | BoxStyle::Rounded => {
                // Draw top and bottom borders
                self.draw_hline(rect.x, rect.y, rect.width, color);
                self.draw_hline(rect.x, rect.y + rect.height - 1, rect.width, color);

                // Draw left and right borders
                self.draw_vline(rect.x, rect.y, rect.height, color);
                self.draw_vline(rect.x + rect.width - 1, rect.y, rect.height, color);
            }
        }
    }

    /// Draw text at the given position
    ///
    /// Returns the number of characters successfully rendered.
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, color: Color) -> usize {
        let Some(font) = self.font else {
            return 0;
        };

        let mut chars_rendered = 0;
        let mut current_x = x;

        for ch in text.chars() {
            // Check if we're still within bounds
            if current_x + font.width > self.width() {
                break;
            }

            // Get glyph data
            let Some(glyph_data) = font.glyph_data(ch) else {
                // Skip characters we don't have glyphs for
                continue;
            };

            // Render the glyph
            self.draw_glyph(current_x, y, font, glyph_data, color);

            current_x += font.width;
            chars_rendered += 1;
        }

        self.dirty = true;
        chars_rendered
    }

    /// Draw a single glyph at the given position
    fn draw_glyph(&mut self, x: usize, y: usize, font: &Font, glyph_data: &[u8], color: Color) {
        let bytes_per_row = (font.width + 7) / 8;

        for row in 0..font.height {
            if y + row >= self.height() {
                break;
            }

            let row_offset = row * bytes_per_row;

            for col in 0..font.width {
                if x + col >= self.width() {
                    break;
                }

                let byte_index = row_offset + (col / 8);
                let bit_index = 7 - (col % 8);

                if byte_index < glyph_data.len() {
                    let byte = glyph_data[byte_index];
                    let bit_set = (byte >> bit_index) & 1 == 1;

                    if bit_set {
                        unsafe {
                            self.framebuffer.set_pixel(x + col, y + row, color);
                        }
                    }
                }
            }
        }
    }

    /// Present the screen (flush to display)
    ///
    /// This is a no-op for direct framebuffer rendering but provides
    /// a hook for future double-buffering implementations.
    pub fn present(&mut self) {
        // For direct framebuffer rendering, this is a no-op
        // Future implementations might use this for double buffering
        self.dirty = false;
    }

    /// Get the size of a text string in pixels
    pub fn text_size(&self, text: &str) -> (usize, usize) {
        let Some(font) = self.font else {
            return (0, 0);
        };

        let width = text.chars().count() * font.width;
        let height = font.height;
        (width, height)
    }

    /// Get the character dimensions (width, height) for the current font
    pub fn char_size(&self) -> Option<(usize, usize)> {
        self.font.map(|f| (f.width, f.height))
    }

    /// Get the number of rows and columns that fit on screen with current font
    pub fn text_dimensions(&self) -> Option<(usize, usize)> {
        self.font.map(|f| {
            let cols = self.width() / f.width;
            let rows = self.height() / f.height;
            (cols, rows)
        })
    }
}
