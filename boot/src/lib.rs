#![no_std]

// Boot crate for moteOS
// Handles UEFI/BIOS boot, memory management, interrupts, framebuffer, and timer setup

pub mod bios;
pub mod framebuffer;
#[cfg(not(target_env = "uefi"))]

#[cfg(not(target_env = "uefi"))]
pub mod interrupts;
pub mod memory;
pub mod timer;
pub mod uefi;

// Re-export commonly used types
pub use framebuffer::FramebufferInfo;
pub use framebuffer::PixelFormat;
pub use memory::{MemoryKind, MemoryMap, MemoryRegion};

/// Boot information passed to kernel_main
///
/// This structure contains all information needed by the kernel to initialize
/// and run the operating system.
#[derive(Debug)]
pub struct BootInfo {
    /// Framebuffer information
    pub framebuffer: FramebufferInfo,
    /// Memory map from bootloader
    pub memory_map: MemoryMap,
    /// ACPI RSDP address (for power management)
    pub rsdp_addr: Option<usize>,
    /// Heap start address (physical)
    pub heap_start: usize,
    /// Heap size in bytes
    pub heap_size: usize,
}

impl BootInfo {
    /// Create a new BootInfo structure
    pub fn new(
        framebuffer: FramebufferInfo,
        memory_map: MemoryMap,
        rsdp_addr: Option<usize>,
        heap_start: usize,
        heap_size: usize,
    ) -> Self {
        Self {
            framebuffer,
            memory_map,
            rsdp_addr,
            heap_start,
            heap_size,
        }
    }
}
