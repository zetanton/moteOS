// Interrupt handling for network drivers

use crate::drivers::virtio::{get_virtio_net, VirtioNet};
use crate::error::NetError;
extern crate alloc;
use alloc::string::ToString;

/// Virtio-net interrupt handler
///
/// This function should be registered with the interrupt system to handle
/// virtio-net device interrupts.
///
/// # Safety
/// This function must only be called from an interrupt handler context.
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn virtio_net_interrupt_handler(
    _stack_frame: x86_64::structures::idt::InterruptStackFrame,
) {
    // Get the virtio-net driver instance
    if let Some(mut driver_guard) = get_virtio_net() {
        if let Some(ref mut driver) = driver_guard.as_mut() {
            // Handle the interrupt
            let _ = driver.handle_interrupt();
        }
    }

    // Acknowledge the interrupt
    // For PCI devices, we may need to send EOI to the interrupt controller
    // This depends on whether MSI-X or legacy interrupts are used
    x86_64::instructions::port::Port::new(0x20).write(0x20u8);
}

/// Register virtio-net interrupt handler
///
/// This function registers the virtio-net interrupt handler with the interrupt system.
/// It should be called after the virtio-net driver is initialized.
///
/// # Arguments
/// * `interrupt_line` - The interrupt line (IRQ) for the virtio-net device
///
/// # Safety
/// This function modifies the interrupt descriptor table and must be called
/// during system initialization.
///
/// # Note
/// In a real implementation, this would need to be integrated with the boot/interrupts
/// module to actually register the handler in the IDT. This is a placeholder interface.
#[cfg(target_arch = "x86_64")]
pub unsafe fn register_virtio_net_interrupt(interrupt_line: u8) -> Result<(), NetError> {
    // Map interrupt line to interrupt vector
    // For PCI devices, interrupt_line is typically in the range 0-15
    // We map it to interrupt vector 32 + interrupt_line
    let interrupt_vector = 32 + interrupt_line as usize;

    if interrupt_vector >= 256 {
        return Err(NetError::PciError("Invalid interrupt line".to_string()));
    }

    // Note: In a real implementation, we would need access to the IDT
    // The actual registration would be done by the boot/interrupts module
    // This function serves as a placeholder to show the intended interface

    Ok(())
}
