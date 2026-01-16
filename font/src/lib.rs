// fon/src/lib.rs

#![no_std]

pub mod psf;

use psf::Font;

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

    pub fn draw_char(&mut self, c: char, x: usize, y: usize, font: &Font) {
        if let Some(glyph) = font.glyph(c) {
            for (row, byte) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if (byte >> (7 - col)) & 1 == 1 {
                        let px = x + col;
                        let py = y + row;
                        if px < self.width && py < self.height {
                            self.buffer[py * self.width + px] = 0xFFFFFFFF; // White
                        }
                    }
                }
            }
        }
    }

    pub fn draw_text(&mut self, text: &str, x: usize, y: usize, font: &Font) {
        let mut current_x = x;
        for c in text.chars() {
            self.draw_char(c, current_x, y, font);
            let width = match font.header {
                psf::Version::V1(_) => 8,
                psf::Version::V2(header) => header.width as usize,
            };
            current_x += width;
        }
    }
}

static FONT_BYTES: &[u8] = include_bytes!("../../assets/ter-u16n.psf");

/// Get the default font.
///
/// # Safety
///
/// The font is loaded from a static byte slice, which is safe.
pub fn get_default_font() -> psf::Result<Font> {
    unsafe { Font::load(FONT_BYTES) }
}
