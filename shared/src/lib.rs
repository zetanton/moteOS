#![no_std]

// Shared crate for moteOS
// Common types, utilities, and data structures shared across crates

pub mod boot_info;
pub mod framebuffer;
pub mod memory;
pub mod timer;

/// Color structure for pixel rendering
///
/// Per Section 3.4 of technical specifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new Color from RGBA components
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new Color from RGB (alpha defaults to 255)
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a black color
    pub const fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    /// Create a white color
    pub const fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

/// Point in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    /// Create a new Point
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// Rectangle structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    /// Create a new Rectangle
    pub const fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get the right edge (x + width)
    pub const fn right(&self) -> usize {
        self.x + self.width
    }

    /// Get the bottom edge (y + height)
    pub const fn bottom(&self) -> usize {
        self.y + self.height
    }

    /// Check if a point is inside this rectangle
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x < self.right() && point.y >= self.y && point.y < self.bottom()
    }

    /// Clip this rectangle to fit within the given bounds
    pub fn clip_to(&self, bounds: Rect) -> Option<Rect> {
        let x = self.x.max(bounds.x);
        let y = self.y.max(bounds.y);
        let right = self.right().min(bounds.right());
        let bottom = self.bottom().min(bounds.bottom());

        if x < right && y < bottom {
            Some(Rect::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum FontError {
    NotAPsfFont,
    InvalidMagic,
    BufferTooSmall,
}

#[derive(Debug)]
pub enum ColorError {
    InvalidHex,
}

// Re-export shared boot types
pub use boot_info::BootInfo;
pub use framebuffer::{FramebufferInfo, PixelFormat};
pub use memory::{MemoryKind, MemoryMap, MemoryRegion};
