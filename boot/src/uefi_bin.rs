// UEFI binary entry point
// This file is compiled as a binary for UEFI boot

#![no_std]
#![no_main]

use uefi::prelude::*;

// Import the appropriate UEFI implementation based on architecture
#[cfg(target_arch = "x86_64")]
use boot::uefi::x86_64;

#[cfg(target_arch = "aarch64")]
use boot::uefi::aarch64;

/// UEFI entry point - delegates to architecture-specific implementation
#[no_mangle]
pub extern "efiapi" fn efi_main(
    image_handle: Handle,
    system_table: *mut uefi::table::SystemTable<uefi::table::Runtime>,
) -> uefi::Status {
    #[cfg(target_arch = "x86_64")]
    return x86_64::efi_main(image_handle, system_table);
    
    #[cfg(target_arch = "aarch64")]
    return aarch64::efi_main(image_handle, system_table);
}
