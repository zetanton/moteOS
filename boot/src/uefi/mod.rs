// UEFI boot support for moteOS
// Provides UEFI boot services, framebuffer acquisition, and memory map handling

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
