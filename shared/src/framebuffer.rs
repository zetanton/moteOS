// Framebuffer interface for moteOS
// Provides safe access to bootloader-provided framebuffer

use crate::{Color, Point, Rect};

/// Pixel format for framebuffer
#[repr(u8)]
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
#[repr(C)]
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
    pub unsafe fn write_pixel(&self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return; // Bounds check - silently ignore out of bounds
        }

        let offset = (y * self.stride) + (x * self.bytes_per_pixel());
        let pixel_ptr = self.base.add(offset);

        match self.pixel_format {
            PixelFormat::Rgb => {
                pixel_ptr.write(color.r);
                pixel_ptr.add(1).write(color.g);
                pixel_ptr.add(2).write(color.b);
            }
            PixelFormat::Bgr => {
                pixel_ptr.write(color.b);
                pixel_ptr.add(1).write(color.g);
                pixel_ptr.add(2).write(color.r);
            }
            PixelFormat::Rgba => {
                pixel_ptr.write(color.r);
                pixel_ptr.add(1).write(color.g);
                pixel_ptr.add(2).write(color.b);
                pixel_ptr.add(3).write(color.a);
            }
            PixelFormat::Bgra => {
                pixel_ptr.write(color.b);
                pixel_ptr.add(1).write(color.g);
                pixel_ptr.add(2).write(color.r);
                pixel_ptr.add(3).write(color.a);
            }
        }
    }

    /// Fill a rectangle with a solid color
    ///
    /// # Safety
    ///
    /// Same safety requirements as `write_pixel`
    pub unsafe fn fill_rectangle(&self, rect: Rect, color: Color) {
        // Clip rectangle to framebuffer bounds
        let bounds = Rect::new(0, 0, self.width, self.height);
        if let Some(clipped) = rect.clip_to(bounds) {
            for py in clipped.y..clipped.bottom() {
                for px in clipped.x..clipped.right() {
                    self.write_pixel(px, py, color);
                }
            }
        }
    }

    /// Draw a line using Bresenham's algorithm
    ///
    /// # Safety
    ///
    /// Same safety requirements as `write_pixel`
    pub unsafe fn draw_line(&self, start: Point, end: Point, color: Color) {
        // Simple Bresenham's line algorithm without clipping
        // Caller must ensure points are within bounds
        let mut x0 = start.x as isize;
        let mut y0 = start.y as isize;
        let x1 = end.x as isize;
        let y1 = end.y as isize;

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if x0 >= 0 && y0 >= 0 {
                self.write_pixel(x0 as usize, y0 as usize, color);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Clear the entire framebuffer (fill with black)
    ///
    /// # Safety
    ///
    /// Same safety requirements as `write_pixel`
    pub unsafe fn clear(&self) {
        let rect = Rect::new(0, 0, self.width, self.height);
        self.fill_rectangle(rect, Color::black());
    }

    // ========== Safe wrapper functions ==========

    /// Safely set a pixel at the given coordinates
    ///
    /// Returns `true` if the pixel was set, `false` if coordinates are out of bounds
    pub fn set_pixel(&self, x: usize, y: usize, color: Color) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        unsafe {
            self.write_pixel(x, y, color);
        }
        true
    }

    /// Safely fill a rectangle with a solid color
    ///
    /// The rectangle is automatically clipped to framebuffer bounds
    pub fn fill_rectangle_safe(&self, rect: Rect, color: Color) {
        unsafe {
            self.fill_rectangle(rect, color);
        }
    }

    /// Safely draw a line with proper clipping
    ///
    /// Uses Cohen-Sutherland line clipping algorithm to clip the line
    /// to framebuffer bounds before drawing
    pub fn draw_line_safe(&self, start: Point, end: Point, color: Color) {
        let bounds = Rect::new(0, 0, self.width, self.height);

        // Clip line to bounds using Cohen-Sutherland algorithm
        if let Some((clipped_start, clipped_end)) =
            self.clip_line_cohen_sutherland(start, end, bounds)
        {
            unsafe {
                self.draw_line(clipped_start, clipped_end, color);
            }
        }
    }

    /// Cohen-Sutherland line clipping algorithm
    ///
    /// Returns `Some((start, end))` if the line (or part of it) is visible,
    /// `None` if the line is completely outside the bounds
    fn clip_line_cohen_sutherland(
        &self,
        mut p0: Point,
        mut p1: Point,
        bounds: Rect,
    ) -> Option<(Point, Point)> {
        // Convert to signed integers for calculations
        let mut x0 = p0.x as i32;
        let mut y0 = p0.y as i32;
        let mut x1 = p1.x as i32;
        let mut y1 = p1.y as i32;

        // Compute region codes for both points
        let mut code0 = compute_region_code_signed(x0, y0, bounds);
        let mut code1 = compute_region_code_signed(x1, y1, bounds);

        const MAX_ITERATIONS: usize = 10; // Prevent infinite loops
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return None; // Safety: prevent infinite loops
            }

            // If both endpoints are inside the bounds, accept the line
            if code0 == 0 && code1 == 0 {
                // Clamp to bounds to ensure valid coordinates
                let start = Point::new(
                    x0.max(0).min(bounds.right() as i32 - 1) as usize,
                    y0.max(0).min(bounds.bottom() as i32 - 1) as usize,
                );
                let end = Point::new(
                    x1.max(0).min(bounds.right() as i32 - 1) as usize,
                    y1.max(0).min(bounds.bottom() as i32 - 1) as usize,
                );
                return Some((start, end));
            }

            // If both endpoints are on the same side of the bounds, reject the line
            if (code0 & code1) != 0 {
                return None;
            }

            // Pick an endpoint that is outside the bounds
            let code_out = if code0 != 0 { code0 } else { code1 };
            let mut x = x0;
            let mut y = y0;

            // Find intersection point
            let dx = x1 - x0;
            let dy = y1 - y0;

            // Clip against each edge
            if (code_out & 0x01) != 0 {
                // Left edge
                if dx != 0 {
                    y = y0 + ((y1 - y0) * (bounds.x as i32 - x0)) / dx;
                }
                x = bounds.x as i32;
            } else if (code_out & 0x02) != 0 {
                // Right edge
                if dx != 0 {
                    y = y0 + ((y1 - y0) * (bounds.right() as i32 - 1 - x0)) / dx;
                }
                x = bounds.right() as i32 - 1;
            } else if (code_out & 0x04) != 0 {
                // Top edge
                if dy != 0 {
                    x = x0 + ((x1 - x0) * (bounds.y as i32 - y0)) / dy;
                }
                y = bounds.y as i32;
            } else if (code_out & 0x08) != 0 {
                // Bottom edge
                if dy != 0 {
                    x = x0 + ((x1 - x0) * (bounds.bottom() as i32 - 1 - y0)) / dy;
                }
                y = bounds.bottom() as i32 - 1;
            }

            // Update the point
            if code_out == code0 {
                x0 = x;
                y0 = y;
                code0 = compute_region_code_signed(x0, y0, bounds);
            } else {
                x1 = x;
                y1 = y;
                code1 = compute_region_code_signed(x1, y1, bounds);
            }
        }
    }
}

/// Compute region code for Cohen-Sutherland clipping (signed version)
///
/// Returns a 4-bit code:
/// - Bit 0: left of bounds
/// - Bit 1: right of bounds
/// - Bit 2: above bounds (top)
/// - Bit 3: below bounds (bottom)
fn compute_region_code_signed(x: i32, y: i32, bounds: Rect) -> u8 {
    let mut code = 0u8;

    if x < bounds.x as i32 {
        code |= 0x01; // Left
    }
    if x >= bounds.right() as i32 {
        code |= 0x02; // Right
    }
    if y < bounds.y as i32 {
        code |= 0x04; // Top
    }
    if y >= bounds.bottom() as i32 {
        code |= 0x08; // Bottom
    }

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a mock framebuffer for testing
    // Note: In real tests, we'd need actual memory, but for unit tests
    // we can test the logic without actual framebuffer access

    #[test]
    fn test_color_creation() {
        let color = Color::new(100, 150, 200, 255);
        assert_eq!(color.r, 100);
        assert_eq!(color.g, 150);
        assert_eq!(color.b, 200);
        assert_eq!(color.a, 255);

        let rgb_color = Color::rgb(50, 100, 150);
        assert_eq!(rgb_color.a, 255);
    }

    #[test]
    fn test_point_creation() {
        let point = Point::new(10, 20);
        assert_eq!(point.x, 10);
        assert_eq!(point.y, 20);
    }

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(10, 20, 100, 200);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 200);
        assert_eq!(rect.right(), 110);
        assert_eq!(rect.bottom(), 220);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 20, 100, 200);

        assert!(rect.contains(Point::new(50, 50)));
        assert!(rect.contains(Point::new(10, 20)));
        assert!(rect.contains(Point::new(109, 219)));

        assert!(!rect.contains(Point::new(9, 50)));
        assert!(!rect.contains(Point::new(110, 50)));
        assert!(!rect.contains(Point::new(50, 19)));
        assert!(!rect.contains(Point::new(50, 220)));
    }

    #[test]
    fn test_rect_clip_to() {
        let bounds = Rect::new(0, 0, 100, 100);

        // Rectangle completely inside bounds
        let rect1 = Rect::new(10, 20, 30, 40);
        let clipped1 = rect1.clip_to(bounds);
        assert_eq!(clipped1, Some(rect1));

        // Rectangle partially outside (right and bottom)
        let rect2 = Rect::new(80, 80, 30, 30);
        let clipped2 = rect2.clip_to(bounds);
        assert_eq!(clipped2, Some(Rect::new(80, 80, 20, 20)));

        // Rectangle completely outside
        let rect3 = Rect::new(150, 150, 50, 50);
        let clipped3 = rect3.clip_to(bounds);
        assert_eq!(clipped3, None);

        // Rectangle partially outside (left and top - use wrapping)
        // Since Rect uses usize, we'll test with a rectangle that starts at 0
        // but extends beyond bounds on the right
        let rect4 = Rect::new(0, 0, 150, 150);
        let clipped4 = rect4.clip_to(bounds);
        assert_eq!(clipped4, Some(Rect::new(0, 0, 100, 100)));
    }

    #[test]
    fn test_region_code_computation() {
        let bounds = Rect::new(10, 20, 100, 200);

        // Point inside bounds
        let code1 = compute_region_code_signed(50, 50, bounds);
        assert_eq!(code1, 0);

        // Point to the left
        let code2 = compute_region_code_signed(5, 50, bounds);
        assert_eq!(code2 & 0x01, 0x01);

        // Point to the right
        let code3 = compute_region_code_signed(150, 50, bounds);
        assert_eq!(code3 & 0x02, 0x02);

        // Point above
        let code4 = compute_region_code_signed(50, 10, bounds);
        assert_eq!(code4 & 0x04, 0x04);

        // Point below
        let code5 = compute_region_code_signed(50, 250, bounds);
        assert_eq!(code5 & 0x08, 0x08);

        // Point in top-left corner (outside)
        let code6 = compute_region_code_signed(5, 10, bounds);
        assert_eq!(code6 & 0x01, 0x01);
        assert_eq!(code6 & 0x04, 0x04);
    }

    #[test]
    fn test_line_clipping_inside() {
        // Create a mock framebuffer info (we won't actually write to it)
        let fb = FramebufferInfo::new(core::ptr::null_mut(), 100, 100, 300, PixelFormat::Rgba);

        let bounds = Rect::new(0, 0, 100, 100);
        let start = Point::new(10, 10);
        let end = Point::new(50, 50);

        // Line completely inside should be accepted
        let result = fb.clip_line_cohen_sutherland(start, end, bounds);
        assert!(result.is_some());
        let (clipped_start, clipped_end) = result.unwrap();
        assert_eq!(clipped_start.x, 10);
        assert_eq!(clipped_start.y, 10);
        assert_eq!(clipped_end.x, 50);
        assert_eq!(clipped_end.y, 50);
    }

    #[test]
    fn test_line_clipping_outside() {
        let fb = FramebufferInfo::new(core::ptr::null_mut(), 100, 100, 300, PixelFormat::Rgba);

        let bounds = Rect::new(0, 0, 100, 100);

        // Line completely outside (both points to the left - use large values that wrap)
        // Since Point uses usize, we'll use values that are clearly outside bounds
        let start = Point::new(200, 50);
        let end = Point::new(250, 60);
        let result = fb.clip_line_cohen_sutherland(start, end, bounds);
        assert!(result.is_none());

        // Line completely outside (both points above - use large values)
        let start = Point::new(50, 200);
        let end = Point::new(60, 250);
        let result = fb.clip_line_cohen_sutherland(start, end, bounds);
        assert!(result.is_none());
    }

    #[test]
    fn test_line_clipping_partial() {
        let fb = FramebufferInfo::new(core::ptr::null_mut(), 100, 100, 300, PixelFormat::Rgba);

        let bounds = Rect::new(0, 0, 100, 100);

        // Line partially inside (starts outside left, ends inside)
        // We can't use negative values with Point, so we'll test with a point
        // that's outside the right edge instead
        let start = Point::new(150, 50);
        let end = Point::new(50, 50);
        let result = fb.clip_line_cohen_sutherland(start, end, bounds);
        assert!(result.is_some());
        let (clipped_start, clipped_end) = result.unwrap();
        // Clipped start should be at right edge
        assert_eq!(clipped_start.x, 99); // bounds.right() - 1
        assert_eq!(clipped_start.y, 50);
        assert_eq!(clipped_end.x, 50);
        assert_eq!(clipped_end.y, 50);
    }

    #[test]
    fn test_set_pixel_bounds_checking() {
        // Create a small buffer for testing
        let mut buffer = [0u8; 400]; // 10x10 RGBA
        let fb = FramebufferInfo::new(
            buffer.as_mut_ptr(),
            10,
            10,
            40, // stride = width * 4
            PixelFormat::Rgba,
        );

        let color = Color::rgb(255, 0, 0);

        // Valid coordinates
        assert!(fb.set_pixel(5, 5, color));

        // Out of bounds X
        assert!(!fb.set_pixel(10, 5, color));
        assert!(!fb.set_pixel(100, 5, color));

        // Out of bounds Y
        assert!(!fb.set_pixel(5, 10, color));
        assert!(!fb.set_pixel(5, 100, color));

        // Both out of bounds
        assert!(!fb.set_pixel(100, 100, color));
    }
}
