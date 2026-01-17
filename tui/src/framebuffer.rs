//! Framebuffer interface and rendering primitives

use crate::colors::Color;
use crate::types::Rect;

/// Pixel format for the framebuffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 24-bit RGB
    Rgb,
    /// 24-bit BGR
    Bgr,
    /// 32-bit RGBA
    Rgba,
    /// 32-bit BGRA
    Bgra,
}

impl PixelFormat {
    /// Returns the number of bytes per pixel
    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Rgb | PixelFormat::Bgr => 3,
            PixelFormat::Rgba | PixelFormat::Bgra => 4,
        }
    }

    /// Write a color to a pixel buffer based on the pixel format
    pub fn write_color(&self, buffer: &mut [u8], color: Color) {
        match self {
            PixelFormat::Rgb => {
                buffer[0] = color.r;
                buffer[1] = color.g;
                buffer[2] = color.b;
            }
            PixelFormat::Bgr => {
                buffer[0] = color.b;
                buffer[1] = color.g;
                buffer[2] = color.r;
            }
            PixelFormat::Rgba => {
                buffer[0] = color.r;
                buffer[1] = color.g;
                buffer[2] = color.b;
                buffer[3] = color.a;
            }
            PixelFormat::Bgra => {
                buffer[0] = color.b;
                buffer[1] = color.g;
                buffer[2] = color.r;
                buffer[3] = color.a;
            }
        }
    }
}

/// Framebuffer information structure
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub base: *mut u8,
    pub width: usize,
    pub height: usize,
    pub stride: usize, // Bytes per row
    pub pixel_format: PixelFormat,
}

impl FramebufferInfo {
    /// Create a new FramebufferInfo
    pub const fn new(
        base: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        pixel_format: PixelFormat,
    ) -> Self {
        Self {
            base,
            width,
            height,
            stride,
            pixel_format,
        }
    }
}

/// Framebuffer for low-level pixel operations
///
/// All operations are unsafe as they directly write to video memory.
/// Use the Screen struct for a safer high-level interface.
pub struct Framebuffer {
    base: *mut u8,
    width: usize,
    height: usize,
    stride: usize,
    pixel_format: PixelFormat,
}

impl Framebuffer {
    /// Create a new framebuffer from info
    ///
    /// # Safety
    ///
    /// The caller must ensure that the base pointer is valid and points to
    /// accessible video memory for the duration of the framebuffer's lifetime.
    pub unsafe fn new(info: FramebufferInfo) -> Self {
        Self {
            base: info.base,
            width: info.width,
            height: info.height,
            stride: info.stride,
            pixel_format: info.pixel_format,
        }
    }

    /// Get the width in pixels
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Get the height in pixels
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Get the pixel format
    pub const fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    /// Set a pixel at the given coordinates
    ///
    /// # Safety
    ///
    /// This function performs bounds checking and will not write if the
    /// coordinates are out of bounds.
    pub unsafe fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let bpp = self.pixel_format.bytes_per_pixel();
        let offset = y * self.stride + x * bpp;
        let pixel_ptr = self.base.add(offset);
        let pixel_slice = core::slice::from_raw_parts_mut(pixel_ptr, bpp);
        self.pixel_format.write_color(pixel_slice, color);
    }

    /// Fill a rectangular region with a solid color
    ///
    /// # Safety
    ///
    /// This function performs bounds checking and will clip the rectangle
    /// to the framebuffer dimensions.
    pub unsafe fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x_end = (rect.x + rect.width).min(self.width);
        let y_end = (rect.y + rect.height).min(self.height);

        for y in rect.y..y_end {
            for x in rect.x..x_end {
                self.set_pixel(x, y, color);
            }
        }
    }

    /// Draw a horizontal line
    ///
    /// # Safety
    ///
    /// This function performs bounds checking.
    pub unsafe fn draw_hline(&mut self, x: usize, y: usize, width: usize, color: Color) {
        if y >= self.height {
            return;
        }

        let x_end = (x + width).min(self.width);
        for px in x..x_end {
            self.set_pixel(px, y, color);
        }
    }

    /// Draw a vertical line
    ///
    /// # Safety
    ///
    /// This function performs bounds checking.
    pub unsafe fn draw_vline(&mut self, x: usize, y: usize, height: usize, color: Color) {
        if x >= self.width {
            return;
        }

        let y_end = (y + height).min(self.height);
        for py in y..y_end {
            self.set_pixel(x, py, color);
        }
    }

    /// Clear the entire framebuffer with a color
    ///
    /// # Safety
    ///
    /// Writes to the entire framebuffer.
    pub unsafe fn clear(&mut self, color: Color) {
        self.fill_rect(
            Rect::new(0, 0, self.width, self.height),
            color,
        );
    }
}
