// UEFI boot implementation for AArch64 architecture
// Handles UEFI boot services, Graphics Output Protocol (GOP), and memory map
// Targets Raspberry Pi and other ARM64 UEFI systems

use crate::{BootInfo, FramebufferInfo, MemoryKind, MemoryMap, MemoryRegion, PixelFormat};
use kernel::kernel_main;
use core::fmt::Write;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::data_types::Identify;
use uefi::table::boot::{MemoryMapKey, MemoryType, SearchType};
use uefi::table::Boot;

/// UEFI entry point for AArch64
///
/// This is the main entry point called by UEFI firmware on ARM64 systems.
/// It initializes UEFI services, acquires the framebuffer, gets the memory map,
/// exits boot services, and then calls kernel_main().
pub fn efi_main(
    _image_handle: Handle,
    st_boot_ref: &mut uefi::table::SystemTable<Boot>,
) -> uefi::Status {
    // Clone the SystemTable so we can move it into exit_boot_services
    let st_boot = unsafe { st_boot_ref.unsafe_clone() };
    let _ = st_boot_ref.stdout().reset(false);
    let _ = st_boot_ref.stdout().clear();
    let _ = writeln!(st_boot_ref.stdout(), "moteOS: bootloader started");
    {
        let bs = st_boot_ref.boot_services();
        let _ = bs.stall(2_000_000);
    }
    let bs = st_boot_ref.boot_services();

    // Acquire framebuffer via Graphics Output Protocol
    let mut framebuffer_failed = false;
    let framebuffer_info = match acquire_framebuffer(bs) {
        Ok(info) => info,
        Err(_) => {
            framebuffer_failed = true;
            // If we can't get framebuffer, create a dummy one
            // This should not happen in normal operation
            FramebufferInfo::new(core::ptr::null_mut(), 0, 0, 0, PixelFormat::Bgra)
        }
    };
    if framebuffer_failed {
        let _ = writeln!(st_boot_ref.stdout(), "moteOS: framebuffer not found");
    }

    // Get memory map (key is not needed in uefi 0.27 - exit_boot_services handles it)
    let (memory_map, _memory_map_key) = match get_memory_map(bs) {
        Ok(map) => map,
        Err(_) => {
            let _ = writeln!(st_boot_ref.stdout(), "moteOS: memory map failed");
            return uefi::Status::ABORTED;
        }
    };

    // Find largest usable memory region for heap
    let (heap_start, heap_size) = if let Some(heap_region) = memory_map
        .regions
        .iter()
        .filter(|r| r.kind == MemoryKind::Usable)
        .max_by_key(|r| r.len)
    {
        // Reserve at least 64MB for heap
        (heap_region.start, (64 * 1024 * 1024).min(heap_region.len))
    } else {
        let _ = writeln!(st_boot_ref.stdout(), "moteOS: no usable memory region");
        let bs = st_boot_ref.boot_services();
        let _ = bs.stall(1_000_000);
        (0, 0)
    };

    // Exit boot services (required before using memory allocator)
    // This invalidates the boot services pointer, so we must do this last
    // In uefi 0.27, exit_boot_services is a method on SystemTable<Boot>
    // It takes MemoryType and returns (SystemTable<Runtime>, MemoryMap<'static>)
    // It consumes st_boot and returns a Runtime view
    // We need to move st_boot here, so we can't use it after this point
    let (_st_runtime, _final_memory_map) = st_boot.exit_boot_services(
        MemoryType::LOADER_DATA
    );

    // Get ACPI RSDP address (if available)
    // Note: ARM64 systems may use Device Tree instead of ACPI
    let rsdp_addr = None; // TODO: Locate ACPI RSDP or Device Tree

    // Create BootInfo
    let boot_info = BootInfo::new(
        framebuffer_info,
        memory_map,
        rsdp_addr,
        heap_start,
        heap_size,
    );

    // Configure MMU for ARM64
    // Note: UEFI may have already set up the MMU, but we should ensure
    // proper page table configuration for our kernel
    // This is a placeholder - actual MMU setup would be more complex
    unsafe {
        configure_mmu();
    }

    let _ = writeln!(st_boot_ref.stdout(), "moteOS: entering kernel_main");
    {
        let bs = st_boot_ref.boot_services();
        let _ = bs.stall(2_000_000);
    }

    // Call kernel_main - this never returns
    kernel_main(boot_info);

    // If kernel_main returns, report and halt
    let _ = writeln!(st_boot_ref.stdout(), "moteOS: kernel_main returned");
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

/// Acquire framebuffer via Graphics Output Protocol
fn acquire_framebuffer(bs: &BootServices) -> Result<FramebufferInfo, uefi::Status> {
    // Locate Graphics Output Protocol using the Identify trait
    let gop_handle = bs
        .locate_handle_buffer(SearchType::ByProtocol(&GraphicsOutput::GUID))
        .map_err(|_| uefi::Status::NOT_FOUND)?;

    if gop_handle.is_empty() {
        return Err(uefi::Status::NOT_FOUND);
    }

    // Open the protocol
    let mut gop = bs
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle[0])
        .map_err(|_| uefi::Status::NOT_FOUND)?;

    // Query available modes and use the first one
    // TODO: Implement better mode selection (find highest resolution >= 1024x768)
    let mut modes = gop.modes(bs);
    let selected_mode = modes
        .next()
        .ok_or(uefi::Status::NOT_FOUND)?;

    // Set the mode
    gop.set_mode(&selected_mode)
        .map_err(|_| uefi::Status::ABORTED)?;

    // Get current mode info
    let mode_info = gop.current_mode_info();
    let (width, height) = mode_info.resolution();
    let stride_pixels = mode_info.stride();

    // Determine pixel format from mode info
    // Force 32bpp BGRA for QEMU/OVMF; 24bpp modes often appear as grayscale.
    let pixel_format = match mode_info.pixel_format() {
        uefi::proto::console::gop::PixelFormat::Rgb => PixelFormat::Bgra,
        uefi::proto::console::gop::PixelFormat::Bgr => PixelFormat::Bgra,
        uefi::proto::console::gop::PixelFormat::Bitmask => PixelFormat::Bgra,
        uefi::proto::console::gop::PixelFormat::BltOnly => PixelFormat::Bgra,
    };

    // Get framebuffer base address
    let framebuffer_base = gop.frame_buffer().as_mut_ptr() as *mut u8;

    let stride = stride_pixels * 4;

    Ok(FramebufferInfo::new(
        framebuffer_base,
        width,
        height,
        stride,
        pixel_format,
    ))
}

/// Get memory map from UEFI
fn get_memory_map(bs: &BootServices) -> Result<(MemoryMap, MemoryMapKey), uefi::Status> {
    const MAX_MEMORY_REGIONS: usize = 256;
    const PAGE_SIZE: usize = 4096;

    // Static storage for memory regions
    static mut REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] = [MemoryRegion {
        start: 0,
        len: 0,
        kind: MemoryKind::Reserved,
    }; MAX_MEMORY_REGIONS];
    static mut REGION_COUNT: usize = 0;

    // Allocate buffer for memory map
    // UEFI requires a buffer that's large enough - we'll use a reasonable size
    let mut buffer = [0u8; 32768];

    let mmap = bs
        .memory_map(&mut buffer)
        .map_err(|_| uefi::Status::ABORTED)?;
    
    let key = mmap.key();

    // Parse memory descriptors into our MemoryRegion array
    let mut count = 0usize;
    for desc in mmap.entries() {
        if count >= MAX_MEMORY_REGIONS {
            break;
        }
        let kind = match desc.ty {
            MemoryType::CONVENTIONAL
            | MemoryType::LOADER_CODE
            | MemoryType::LOADER_DATA
            | MemoryType::BOOT_SERVICES_CODE
            | MemoryType::BOOT_SERVICES_DATA => MemoryKind::Usable,
            MemoryType::ACPI_RECLAIM => MemoryKind::AcpiReclaimable,
            MemoryType::ACPI_NON_VOLATILE => MemoryKind::AcpiNvs,
            _ => MemoryKind::Reserved,
        };

        unsafe {
            REGIONS[count] = MemoryRegion {
                start: desc.phys_start as usize,
                len: (desc.page_count as usize) * PAGE_SIZE,
                kind,
            };
        }
        count += 1;
    }

    unsafe {
        REGION_COUNT = count;
    }

    let memory_regions = unsafe { &REGIONS[..REGION_COUNT] };
    let memory_map = MemoryMap::new(memory_regions);

    Ok((memory_map, key))
}

/// Configure MMU for ARM64
///
/// # Safety
///
/// This function is unsafe because it directly manipulates system registers.
/// It should only be called after exiting boot services.
unsafe fn configure_mmu() {
    // ARM64 MMU configuration
    // UEFI typically sets up the MMU, but we may need to:
    // 1. Ensure proper page table configuration
    // 2. Set up memory attributes (Normal, Device, etc.)
    // 3. Configure cacheability and shareability
    
    // For now, this is a placeholder
    // In a full implementation, we would:
    // - Read current page table base (TTBR0_EL1)
    // - Verify memory attributes
    // - Set up any additional mappings needed
    
    // Note: Raspberry Pi 4 uses 4KB pages and 48-bit virtual addressing
    // The MMU is typically already configured by UEFI firmware
}
