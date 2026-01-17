// UEFI boot implementation for x86_64 architecture
// Handles UEFI boot services, Graphics Output Protocol (GOP), and memory map

use crate::{BootInfo, FramebufferInfo, MemoryKind, MemoryMap, MemoryRegion, PixelFormat};
use kernel::kernel_main;
use core::fmt::Write;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::data_types::Identify;
use uefi::table::boot::{MemoryMapKey, MemoryType, SearchType};
use uefi::table::Boot;

 

/// UEFI entry point for x86_64
///
/// This is the main entry point called by UEFI firmware.
/// It initializes UEFI services, acquires the framebuffer, gets the memory map,
/// exits boot services, and then calls kernel_main().
pub extern "efiapi" fn efi_main(
    _image_handle: Handle,
    system_table: *mut uefi::table::SystemTable<uefi::table::Runtime>,
) -> uefi::Status {
    // Safety: system_table is provided by UEFI firmware and is valid
    // Convert from Runtime view to Boot view to access boot services
    let st_boot_ref = unsafe {
        // Cast the raw pointer to Boot view - this is safe because at entry time
        // the system table is in boot services mode
        &mut *(system_table as *mut uefi::table::SystemTable<Boot>)
    };
    
    // Clone the SystemTable so we can move it into exit_boot_services
    // This is safe because the system table pointer is valid and stable
    let st_boot = unsafe { st_boot_ref.unsafe_clone() };
    let _ = st_boot_ref.stdout().clear();
    let _ = writeln!(st_boot_ref.stdout(), "moteOS: booting kernel...");
    let bs = st_boot_ref.boot_services();

    // Acquire framebuffer via Graphics Output Protocol
    let framebuffer_info = match acquire_framebuffer(bs) {
        Ok(info) => info,
        Err(_) => {
            // If we can't get framebuffer, create a dummy one
            // This should not happen in normal operation
            FramebufferInfo::new(core::ptr::null_mut(), 0, 0, 0, PixelFormat::Bgra)
        }
    };

    // Get memory map (key is not needed in uefi 0.27 - exit_boot_services handles it)
    let (memory_map, _memory_map_key) = match get_memory_map(bs) {
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
    // In uefi 0.27, exit_boot_services is a method on SystemTable<Boot>
    // It takes MemoryType and returns (SystemTable<Runtime>, MemoryMap<'static>)
    // It consumes st_boot and returns a Runtime view
    // We need to move st_boot here, so we can't use it after this point
    let (_st_runtime, _final_memory_map) = st_boot.exit_boot_services(
        MemoryType::LOADER_DATA
    );

    // Convert memory map storage to our MemoryMap format
    // Note: This is a simplified conversion - in a real implementation,
    // we'd need to properly parse the UEFI memory map
    // Create empty memory map for now
    static EMPTY_REGIONS: [MemoryRegion; 0] = [];
    let memory_regions: &'static [MemoryRegion] = &EMPTY_REGIONS;
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

    // Call kernel_main - this never returns
    kernel_main(boot_info);
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
    let mut buffer = [0u8; 32768];

    let mmap = bs
        .memory_map(&mut buffer)
        .map_err(|_| uefi::Status::ABORTED)?;
    
    let key = mmap.key();

    // Parse memory descriptors
    // Note: This is a simplified implementation
    // In a real implementation, we'd properly parse all descriptors
    // Create empty memory map for now
    static EMPTY_REGIONS: [MemoryRegion; 0] = [];
    let memory_regions: &'static [MemoryRegion] = &EMPTY_REGIONS;

    let memory_map = MemoryMap::new(memory_regions);

    Ok((memory_map, key))
}
