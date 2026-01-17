// tui/src/lib.rs

#![no_std]
extern crate alloc;

pub mod font;

use font::Font;
pub use shared::Color;

pub struct Framebuffer<'a> {
    pub width: usize,
    pub height: usize,
    pub buffer: &'a mut [u32],
}

impl<'a> Framebuffer<'a> {
    pub fn new(width: usize, height: usize, buffer: &'a mut [u32]) -> Self {
        Self {
            width,
            height,
            buffer,
        }
    }

    pub fn draw_char(&mut self, c: char, x: usize, y: usize, font: &Font, color: Color) {
        if let Some(glyph) = font.glyph_data(c) {
            for (row, byte) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if (byte >> (7 - col)) & 1 == 1 {
                        let px = x + col;
                        let py = y + row;
                        if px < self.width && py < self.height {
                            // Convert RGBA Color to u32 (assuming ARGB or similar for the dummy fb)
                            let color_u32 = ((color.a as u32) << 24) |
                                            ((color.r as u32) << 16) |
                                            ((color.g as u32) << 8) |
                                            (color.b as u32);
                            self.buffer[py * self.width + px] = color_u32;
                        }
                    }
                }
            }
        }
    }
}

pub fn draw_text(
    fb: &mut Framebuffer,
    font: &Font,
    x: usize,
    y: usize,
    text: &str,
    color: Color,
) {
    let mut current_x = x;
    for c in text.chars() {
        fb.draw_char(c, current_x, y, font, color);
        current_x += font.width;
    }
}

static FONT_BYTES: &[u8] = include_bytes!("../../assets/ter-u16n.psf");

/// Get the default font.
///
/// # Safety
///
/// The font is loaded from a static byte slice, which is safe.
pub fn get_default_font() -> font::Result<Font> {
    unsafe { Font::load_psf(FONT_BYTES) }
}