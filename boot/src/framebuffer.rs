// Framebuffer interface for moteOS
// Provides safe access to bootloader-provided framebuffer


/// Pixel format for framebuffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 24-bit RGB (red, green, blue)
    Rgb,
    /// 24-bit BGR (blue, green, red)
    Bgr,
    /// 32-bit RGBA (red, green, blue, alpha)
    Rgba,
    /// 32-bit BGRA (blue, green, red, alpha)
    Bgra,
}

impl PixelFormat {
    /// Get bytes per pixel for this format
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Rgb | PixelFormat::Bgr => 3,
            PixelFormat::Rgba | PixelFormat::Bgra => 4,
        }
    }
}

/// Framebuffer information structure
/// 
/// This struct contains all information needed to access and write to the framebuffer.
/// The `base` pointer is unsafe to dereference and must be used with caution.
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Base address of the framebuffer (unsafe to dereference)
    pub base: *mut u8,
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
    /// Stride (bytes per row, may be larger than width * bytes_per_pixel)
    pub stride: usize,
    /// Pixel format
    pub pixel_format: PixelFormat,
}

impl FramebufferInfo {
    /// Create a new FramebufferInfo
    pub fn new(
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

    /// Get the total size of the framebuffer in bytes
    pub fn size_bytes(&self) -> usize {
        self.stride * self.height
    }

    /// Get bytes per pixel for this framebuffer
    pub fn bytes_per_pixel(&self) -> usize {
        self.pixel_format.bytes_per_pixel()
    }

    /// Write a pixel at the given coordinates
    /// 
    /// # Safety
    /// 
    /// This function is unsafe because:
    /// - It dereferences a raw pointer
    /// - It does not check bounds (caller must ensure x < width, y < height)
    /// - The framebuffer memory must be valid and writable
    pub unsafe fn write_pixel(&self, x: usize, y: usize, r: u8, g: u8, b: u8, a: u8) {
        if x >= self.width || y >= self.height {
            return; // Bounds check - silently ignore out of bounds
        }

        let offset = (y * self.stride) + (x * self.bytes_per_pixel());
        let pixel_ptr = self.base.add(offset);

        match self.pixel_format {
            PixelFormat::Rgb => {
                pixel_ptr.write(r);
                pixel_ptr.add(1).write(g);
                pixel_ptr.add(2).write(b);
            }
            PixelFormat::Bgr => {
                pixel_ptr.write(b);
                pixel_ptr.add(1).write(g);
                pixel_ptr.add(2).write(r);
            }
            PixelFormat::Rgba => {
                pixel_ptr.write(r);
                pixel_ptr.add(1).write(g);
                pixel_ptr.add(2).write(b);
                pixel_ptr.add(3).write(a);
            }
            PixelFormat::Bgra => {
                pixel_ptr.write(b);
                pixel_ptr.add(1).write(g);
                pixel_ptr.add(2).write(r);
                pixel_ptr.add(3).write(a);
            }
        }
    }

    /// Fill a rectangle with a solid color
    /// 
    /// # Safety
    /// 
    /// Same safety requirements as `write_pixel`
    pub unsafe fn fill_rect(&self, x: usize, y: usize, width: usize, height: usize, r: u8, g: u8, b: u8, a: u8) {
        let end_x = (x + width).min(self.width);
        let end_y = (y + height).min(self.height);

        for py in y..end_y {
            for px in x..end_x {
                self.write_pixel(px, py, r, g, b, a);
            }
        }
    }

    /// Clear the entire framebuffer (fill with black)
    /// 
    /// # Safety
    /// 
    /// Same safety requirements as `write_pixel`
    pub unsafe fn clear(&self) {
        self.fill_rect(0, 0, self.width, self.height, 0, 0, 0, 255);
    }
}
