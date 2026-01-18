// UEFI boot implementation for x86_64 architecture
// Handles UEFI boot services, Graphics Output Protocol (GOP), and memory map

use crate::{BootInfo, Color, FramebufferInfo, MemoryKind, MemoryMap, MemoryRegion, PixelFormat, Rect};
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
pub fn efi_main(
    _image_handle: Handle,
    st_boot_ref: &mut uefi::table::SystemTable<Boot>,
) -> uefi::Status {
    // Avoid direct serial I/O in UEFI (can fault on some firmware/QEMU setups)
    // Clone the SystemTable so we can move it into exit_boot_services
    let st_boot = unsafe { st_boot_ref.unsafe_clone() };
    let _ = st_boot_ref.stdout().reset(false);
    let _ = st_boot_ref.stdout().clear();
    let _ = writeln!(st_boot_ref.stdout(), "moteOS: bootloader started");
    {
        let bs = st_boot_ref.boot_services();
        let _ = bs.stall(2_000_000);
    }
    // Acquire framebuffer via Graphics Output Protocol
    let mut framebuffer_failed = false;
    let framebuffer_info = {
        let bs = st_boot_ref.boot_services();
        match acquire_framebuffer(bs) {
            Ok(info) => info,
            Err(_) => {
                framebuffer_failed = true;
                // If we can't get framebuffer, create a dummy one
                // This should not happen in normal operation
                FramebufferInfo::new(core::ptr::null_mut(), 0, 0, 0, PixelFormat::Bgra)
            }
        }
    };
    if framebuffer_failed {
        let _ = writeln!(st_boot_ref.stdout(), "moteOS: framebuffer not found");
    } else {
        let _ = writeln!(st_boot_ref.stdout(), "moteOS: framebuffer ok");
    }
    {
        let bs = st_boot_ref.boot_services();
        let _ = bs.stall(1_000_000);
    }

    // Quick visual confirmation that the bootloader is running
    if framebuffer_info.width > 0 && framebuffer_info.height > 0 {
        let bounds = Rect::new(0, 0, framebuffer_info.width, framebuffer_info.height);
        framebuffer_info.fill_rectangle_safe(bounds, Color::rgb(0, 0, 255));
        let bs = st_boot_ref.boot_services();
        let _ = bs.stall(1_000_000);
    }

    // Get memory map (key is not needed in uefi 0.27 - exit_boot_services handles it)
    let (memory_map, _memory_map_key) = {
        let bs = st_boot_ref.boot_services();
        match get_memory_map(bs) {
            Ok(map) => map,
            Err(_) => {
                let _ = writeln!(st_boot_ref.stdout(), "moteOS: memory map failed");
                return uefi::Status::ABORTED;
            }
        }
    };
    let _ = writeln!(st_boot_ref.stdout(), "moteOS: memory map ok");
    {
        let bs = st_boot_ref.boot_services();
        let _ = bs.stall(1_000_000);
    }

    // Find largest usable memory region for heap
    // Debug: count usable regions and total usable memory
    let usable_count = memory_map.regions.iter().filter(|r| r.kind == MemoryKind::Usable).count();
    let usable_total: usize = memory_map.regions.iter().filter(|r| r.kind == MemoryKind::Usable).map(|r| r.len).sum();
    let _ = writeln!(st_boot_ref.stdout(), "moteOS: {} usable regions, {} bytes total", usable_count, usable_total);

    let (heap_start, heap_size) = if let Some(heap_region) = memory_map
        .regions
        .iter()
        .filter(|r| r.kind == MemoryKind::Usable)
        .max_by_key(|r| r.len)
    {
        // Reserve at least 64MB for heap
        let size = (64 * 1024 * 1024).min(heap_region.len);
        let _ = writeln!(st_boot_ref.stdout(), "moteOS: heap at 0x{:x}, size {} bytes", heap_region.start, size);
        (heap_region.start, size)
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
    let _ = writeln!(st_boot_ref.stdout(), "moteOS: exiting boot services");
    {
        let bs = st_boot_ref.boot_services();
        let _ = bs.stall(1_000_000);
    }
    let (_st_runtime, _final_memory_map) = st_boot.exit_boot_services(
        MemoryType::LOADER_DATA
    );

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

    // Boot services are invalid past this point; jump straight to the kernel.

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

    // Smart mode selection for real hardware compatibility
    // Prefer standard resolutions with 32-bit color, avoid BltOnly modes
    let modes = gop.modes(bs);
    let mut best_mode = None;
    let mut best_score = 0u32;

    for mode in modes {
        let info = mode.info();
        let (w, h) = info.resolution();
        let format = info.pixel_format();

        // Skip BltOnly modes - no direct framebuffer access
        if matches!(format, uefi::proto::console::gop::PixelFormat::BltOnly) {
            continue;
        }

        // Only accept 32-bit color modes (Rgb or Bgr)
        let is_32bpp = matches!(
            format,
            uefi::proto::console::gop::PixelFormat::Rgb
                | uefi::proto::console::gop::PixelFormat::Bgr
        );
        if !is_32bpp {
            continue;
        }

        // Score based on resolution - prefer moderate sizes that fit most screens
        // Prioritize 1280x720 or 1024x768 for better compatibility
        let score = match (w, h) {
            (1280, 720) => 100,  // Preferred - fits most screens well
            (1024, 768) => 95,   // Good fallback
            (1280, 800) => 90,
            (1280, 1024) => 85,
            (800, 600) => 80,    // Small but usable
            (1440, 900) => 70,
            (1600, 900) => 65,
            (1680, 1050) => 60,
            (1920, 1080) => 50,  // May be too large for some screens
            _ if w >= 1024 && w <= 1440 && h >= 720 && h <= 900 => 75,
            _ if w >= 800 && h >= 600 => 40,
            _ => 10,
        };

        if score > best_score {
            best_score = score;
            best_mode = Some(mode);
        }
    }

    let selected_mode = best_mode.ok_or(uefi::Status::NOT_FOUND)?;

    // Set the mode
    gop.set_mode(&selected_mode)
        .map_err(|_| uefi::Status::ABORTED)?;

    // Get current mode info
    let mode_info = gop.current_mode_info();
    let (width, height) = mode_info.resolution();
    let stride_pixels = mode_info.stride();

    // Correct pixel format handling for real hardware
    let pixel_format = match mode_info.pixel_format() {
        // RGB means red byte first (R, G, B, A order in memory)
        uefi::proto::console::gop::PixelFormat::Rgb => PixelFormat::Rgba,
        // BGR means blue byte first (B, G, R, A order in memory) - most common
        uefi::proto::console::gop::PixelFormat::Bgr => PixelFormat::Bgra,
        // Bitmask requires inspecting pixel mask - default to BGRA (most common)
        uefi::proto::console::gop::PixelFormat::Bitmask => PixelFormat::Bgra,
        // BltOnly shouldn't reach here (filtered above), but handle it
        uefi::proto::console::gop::PixelFormat::BltOnly => {
            return Err(uefi::Status::UNSUPPORTED);
        }
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
    // Increased for real hardware with complex memory maps (200+ regions possible)
    const MAX_MEMORY_REGIONS: usize = 512;
    const PAGE_SIZE: usize = 4096;

    // Static storage for memory regions
    static mut REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] = [MemoryRegion {
        start: 0,
        len: 0,
        kind: MemoryKind::Reserved,
    }; MAX_MEMORY_REGIONS];
    static mut REGION_COUNT: usize = 0;

    // Allocate buffer for memory map
    // 64KB buffer for complex memory maps on real hardware (Lenovo, etc.)
    let mut buffer = [0u8; 65536];

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
        // CRITICAL: Correct memory type classification for real hardware.
        // BOOT_SERVICES regions become undefined after ExitBootServices per UEFI spec.
        // Real hardware (Lenovo, etc.) enforces this - using them causes corruption.
        let kind = match desc.ty {
            // Only CONVENTIONAL memory is truly free after ExitBootServices
            MemoryType::CONVENTIONAL => MemoryKind::Usable,
            // LOADER regions contain our bootloader - can be reclaimed after kernel init
            MemoryType::LOADER_CODE | MemoryType::LOADER_DATA => {
                MemoryKind::BootloaderReclaimable
            }
            // BOOT_SERVICES regions are INVALID after ExitBootServices - MUST NOT use
            MemoryType::BOOT_SERVICES_CODE | MemoryType::BOOT_SERVICES_DATA => {
                MemoryKind::Reserved
            }
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
