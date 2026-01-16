#![no_std]

// Boot crate for moteOS
// Handles UEFI/BIOS boot, memory management, interrupts, framebuffer, and timer setup

pub mod uefi;
pub mod bios;
pub mod memory;
pub mod interrupts;
pub mod framebuffer;
pub mod timer;
