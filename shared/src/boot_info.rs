// Boot information passed from bootloader to kernel

use crate::framebuffer::FramebufferInfo;
use crate::memory::MemoryMap;

/// Boot information passed to kernel_main
///
/// This structure contains all information needed by the kernel to initialize
/// and run the operating system.
#[repr(C)]
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
