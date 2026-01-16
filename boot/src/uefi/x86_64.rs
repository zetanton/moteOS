// UEFI boot implementation for x86_64 architecture
// Handles UEFI boot services, Graphics Output Protocol (GOP), and memory map

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use crate::framebuffer::{FramebufferInfo, PixelFormat};
use crate::memory::{MemoryKind, MemoryMap, MemoryRegion};
use crate::BootInfo;

/// UEFI entry point for x86_64
/// 
/// This is the main entry point called by UEFI firmware.
/// It initializes UEFI services, acquires the framebuffer, gets the memory map,
/// exits boot services, and then calls kernel_main().
#[no_mangle]
pub extern "efiapi" fn efi_main(
    image_handle: Handle,
    system_table: *mut uefi::table::SystemTable<uefi::table::Runtime>,
) -> uefi::Status {
    // Safety: system_table is provided by UEFI firmware and is valid
    let uefi::Result::Ok(bs) = unsafe { uefi::table::SystemTable::as_mut(system_table) }
        .map(|st| unsafe { st.boot_services() }) else {
        return uefi::Status::ABORTED;
    };

    // Acquire framebuffer via Graphics Output Protocol
    let framebuffer_info = match acquire_framebuffer(bs) {
        Ok(info) => info,
        Err(_) => {
            // If we can't get framebuffer, create a dummy one
            // This should not happen in normal operation
            FramebufferInfo::new(
                core::ptr::null_mut(),
                0,
                0,
                0,
                PixelFormat::Bgra,
            )
        }
    };

    // Get memory map
    let (memory_map, memory_map_key) = match get_memory_map(bs) {
        Ok(map) => map,
        Err(_) => {
            return uefi::Status::ABORTED;
        }
    };

    // Find largest usable memory region for heap
    let heap_region = memory_map
        .regions
        .iter()
        .filter(|r| r.kind == MemoryKind::Usable)
        .max_by_key(|r| r.len)
        .expect("No usable memory region found");

    // Reserve at least 64MB for heap
    let heap_size = (64 * 1024 * 1024).min(heap_region.len);
    let heap_start = heap_region.start;

    // Exit boot services (required before using memory allocator)
    // This invalidates the boot services pointer, so we must do this last
    let (memory_map_storage, _) = match bs.exit_boot_services(image_handle, memory_map_key) {
        Ok(map) => map,
        Err(_) => {
            return uefi::Status::ABORTED;
        }
    };

    // Convert memory map storage to our MemoryMap format
    // Note: This is a simplified conversion - in a real implementation,
    // we'd need to properly parse the UEFI memory map
    let memory_regions: &'static [MemoryRegion] = unsafe {
        core::slice::from_raw_parts(
            core::ptr::null(),
            0,
        )
    };
    let memory_map = MemoryMap::new(memory_regions);

    // Get ACPI RSDP address (if available)
    let rsdp_addr = None; // TODO: Locate ACPI RSDP

    // Create BootInfo
    let boot_info = BootInfo::new(
        framebuffer_info,
        memory_map,
        rsdp_addr,
        heap_start,
        heap_size,
    );

    // Call kernel_main
    // Note: kernel_main should be defined in the kernel crate
    // For now, we'll just halt
    unsafe {
        x86_64::instructions::hlt();
    }

    uefi::Status::SUCCESS
}

/// Acquire framebuffer via Graphics Output Protocol
fn acquire_framebuffer(bs: &BootServices) -> Result<FramebufferInfo, uefi::Status> {
    // Locate Graphics Output Protocol
    let gop_handle = bs
        .locate_handle_buffer::<GraphicsOutput>()
        .map_err(|_| uefi::Status::NOT_FOUND)?;

    if gop_handle.is_empty() {
        return Err(uefi::Status::NOT_FOUND);
    }

    // Open the protocol
    let gop = bs
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle[0])
        .map_err(|_| uefi::Status::NOT_FOUND)?;

    // Query available modes
    let modes = gop.modes(bs);
    
    // Find the best mode (highest resolution, or at least 1024x768)
    let mut best_mode = None;
    let mut best_resolution = 0;

    for mode in modes {
        let info = mode.info();
        let resolution = info.resolution().0 * info.resolution().1;
        
        if resolution >= 1024 * 768 && resolution > best_resolution {
            best_resolution = resolution;
            best_mode = Some(mode);
        }
    }

    // If no suitable mode found, use the first available mode
    let selected_mode = best_mode.unwrap_or_else(|| {
        modes.into_iter().next().expect("No graphics modes available")
    });

    // Set the mode
    gop.set_mode(&selected_mode)
        .map_err(|_| uefi::Status::ABORTED)?;

    // Get current mode info
    let mode_info = gop.current_mode_info();
    let (width, height) = mode_info.resolution();
    let stride = mode_info.stride();

    // Determine pixel format from mode info
    let pixel_format = match mode_info.pixel_format() {
        uefi::proto::console::gop::PixelFormat::Rgb => PixelFormat::Rgb,
        uefi::proto::console::gop::PixelFormat::Bgr => PixelFormat::Bgr,
        uefi::proto::console::gop::PixelFormat::Bitmask => PixelFormat::Bgra, // Default
        uefi::proto::console::gop::PixelFormat::BltOnly => PixelFormat::Bgra, // Default
    };

    // Get framebuffer base address
    let framebuffer_base = gop.frame_buffer().as_mut_ptr() as *mut u8;

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
    // Allocate buffer for memory map
    // UEFI requires a buffer that's large enough - we'll use a reasonable size
    let buffer_size = 4096 * 8; // 32KB should be enough for most systems
    let mut buffer = [0u8; 32768];
    
    let (key, desc_size) = bs
        .memory_map(&mut buffer)
        .map_err(|_| uefi::Status::ABORTED)?;

    // Parse memory descriptors
    // Note: This is a simplified implementation
    // In a real implementation, we'd properly parse all descriptors
    let memory_regions: &'static [MemoryRegion] = unsafe {
        core::slice::from_raw_parts(
            core::ptr::null(),
            0,
        )
    };

    let memory_map = MemoryMap::new(memory_regions);

    Ok((memory_map, key))
}
