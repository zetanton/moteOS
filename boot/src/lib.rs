#![no_std]

// Boot crate for moteOS
// Handles UEFI/BIOS boot, memory management, interrupts, framebuffer, and timer setup

pub mod bios;
#[cfg(not(target_env = "uefi"))]
pub mod interrupts;
pub mod memory;
pub mod uefi;

// Re-export commonly used types from shared
pub use shared::{
    BootInfo, Color, FramebufferInfo, MemoryKind, MemoryMap, MemoryRegion, PixelFormat, Point,
    Rect,
};
