//! Main event loop
//!
//! This module implements the main event loop that drives the operating system.
//! The loop handles input events, network polling, and screen updates.

use crate::input;
use crate::screen;

/// Main event loop
///
/// This is the main loop of the operating system. It continuously:
/// 1. Handles keyboard input
/// 2. Polls the network stack
/// 3. Updates the screen
/// 4. Sleeps briefly to maintain ~60 FPS
///
/// This function never returns.
pub fn main_loop() -> ! {
    loop {
        // Handle keyboard input
        input::handle_input();

        // Poll network stack
        poll_network();

        // Update screen
        screen::update_screen();

        // Sleep for ~16ms to maintain ~60 FPS
        // TODO: Use timer interrupt instead of busy sleep
        sleep_ms(16);
    }
}

/// Poll the network stack
///
/// Calls the network stack's poll function to process incoming/outgoing packets,
/// handle timeouts, and update TCP state machines.
fn poll_network() {
    // TODO: Implement once network stack is complete
    // This will call network::poll_network_stack() or similar
}

/// Sleep for the specified number of milliseconds
///
/// # Arguments
///
/// * `ms` - Number of milliseconds to sleep
///
/// # Note
///
/// This is a busy-wait implementation. A proper implementation would use
/// timer interrupts to avoid wasting CPU cycles.
fn sleep_ms(ms: u64) {
    // TODO: Implement proper sleep using timer
    // For now, just a busy loop
    #[cfg(target_arch = "x86_64")]
    {
        // Rough approximation - this will vary by CPU speed
        // Should be replaced with HPET/APIC timer
        for _ in 0..(ms * 1000000) {
            unsafe {
                core::arch::asm!("pause");
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 busy wait
        // Should be replaced with ARM Generic Timer
        for _ in 0..(ms * 1000000) {
            unsafe {
                core::arch::asm!("yield");
            }
        }
    }
}
