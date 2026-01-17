//! Main event loop
//!
//! This module implements the main event loop that drives the operating system.
//! The loop handles input events, network polling, and screen updates.

use crate::GLOBAL_STATE;
use crate::init;
use boot::timer;
use network::poll_network_stack;

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
        crate::input::handle_input();

        // Poll network stack
        poll_network();

        // Update screen
        crate::screen::update_screen();

        // Sleep for ~16ms to maintain ~60 FPS
        sleep_ms(16);
    }
}

/// Poll the network stack
///
/// Calls the network stack's poll function to process incoming/outgoing packets,
/// handle timeouts, and update TCP state machines.
fn poll_network() {
    let timestamp_ms = init::get_time_ms();
    let _ = poll_network_stack(timestamp_ms);
}

/// Sleep for the specified number of milliseconds
///
/// # Arguments
///
/// * `ms` - Number of milliseconds to sleep
fn sleep_ms(ms: u64) {
    timer::sleep_ms(ms);
}
