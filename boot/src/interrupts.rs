// Interrupt handling for moteOS
// Configures IDT (x86_64) or GIC (ARM64) and interrupt handlers

#[cfg(target_arch = "x86_64")]
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// Interrupt handler function type
#[cfg(target_arch = "x86_64")]
pub type InterruptHandler = extern "x86-interrupt" fn(InterruptStackFrame);

/// Global Interrupt Descriptor Table
#[cfg(target_arch = "x86_64")]
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// Initialize the Interrupt Descriptor Table
/// 
/// Sets up handlers for:
/// - Breakpoint exceptions
/// - Double fault exceptions
/// - Timer interrupts (IRQ 0)
/// - Keyboard interrupts (IRQ 1)
/// 
/// # Safety
/// 
/// This function must only be called once during boot initialization.
#[cfg(target_arch = "x86_64")]
pub unsafe fn init_idt() {
    IDT.breakpoint.set_handler_fn(breakpoint_handler);
    IDT.double_fault.set_handler_fn(double_fault_handler);
    
    // Timer interrupt (IRQ 0, mapped to interrupt 32)
    IDT[32].set_handler_fn(timer_interrupt_handler);
    
    // Keyboard interrupt (IRQ 1, mapped to interrupt 33)
    IDT[33].set_handler_fn(keyboard_interrupt_handler);
    
    // Load the IDT
    IDT.load();
}

/// Breakpoint exception handler
#[cfg(target_arch = "x86_64")]
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    // For now, just halt - in a real implementation, we'd log this
    unsafe {
        x86_64::instructions::hlt();
    }
}

/// Double fault exception handler
/// 
/// Double faults are unrecoverable errors. This handler should never be called
/// in normal operation.
#[cfg(target_arch = "x86_64")]
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // Double fault is unrecoverable - halt the system
    loop {
        unsafe {
            x86_64::instructions::hlt();
        }
    }
}

/// Timer interrupt handler
/// 
/// Called periodically by the timer hardware (HPET/APIC).
/// This handler should be fast and not perform heavy operations.
#[cfg(target_arch = "x86_64")]
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Increment tick counter
    crate::timer::increment_ticks();
    
    // Acknowledge the interrupt
    // For APIC: write to EOI register
    // For HPET: handled by hardware
}

/// Keyboard interrupt handler
/// 
/// Called when a key is pressed or released on the keyboard.
#[cfg(target_arch = "x86_64")]
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Read keyboard scan code from port 0x60
    // In a real implementation, we'd queue this for processing by the input system
    
    // Acknowledge the interrupt
    unsafe {
        // Send EOI to PIC
        x86_64::instructions::port::Port::new(0x20).write(0x20u8);
    }
}

/// Panic handler for interrupts
/// 
/// This is called when a panic occurs. It halts the CPU.
#[cfg(target_arch = "x86_64")]
pub fn panic_handler() -> ! {
    loop {
        unsafe {
            x86_64::instructions::hlt();
        }
    }
}

/// Enable interrupts
#[cfg(target_arch = "x86_64")]
pub unsafe fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

/// Disable interrupts
#[cfg(target_arch = "x86_64")]
pub unsafe fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}

// ARM64 implementation would go here
#[cfg(target_arch = "aarch64")]
pub unsafe fn init_idt() {
    // ARM64 uses GIC (Generic Interrupt Controller) instead of IDT
    // This would be implemented separately
}

#[cfg(target_arch = "aarch64")]
pub unsafe fn enable_interrupts() {
    // ARM64 interrupt enable
}

#[cfg(target_arch = "aarch64")]
pub unsafe fn disable_interrupts() {
    // ARM64 interrupt disable
}
