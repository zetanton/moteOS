// UEFI binary entry point
// This file is compiled as a binary for UEFI boot

#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::table::Boot;
use uefi::entry;

// Import the appropriate UEFI implementation based on architecture
#[cfg(target_arch = "x86_64")]
use boot::uefi::x86_64;

#[cfg(target_arch = "aarch64")]
use boot::uefi::aarch64;

/// UEFI entry point - delegates to architecture-specific implementation
#[entry]
fn efi_main(
    image_handle: Handle,
    mut system_table: SystemTable<Boot>,
) -> Status {
    #[cfg(target_arch = "x86_64")]
    return x86_64::efi_main(image_handle, &mut system_table);
    
    #[cfg(target_arch = "aarch64")]
    return aarch64::efi_main(image_handle, &mut system_table);
}

/// Panic handler for UEFI boot
#[cfg(all(not(test), not(feature = "kernel-linked")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In a real implementation, we'd log the panic info via UEFI console
    // For now, just loop forever
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("hlt");
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfi");
        }
    }
}
