//! Input handling module
//!
//! This module handles keyboard input from PS/2 or USB HID keyboards.
//! It reads keyboard events and dispatches them to the appropriate handlers.

use crate::GLOBAL_STATE;
use config::Key;

/// Handle keyboard input
///
/// Reads keyboard input and processes it based on the current application state.
/// This is called from the main event loop.
pub fn handle_input() {
    // Read keyboard input
    if let Some(key) = read_keyboard() {
        process_key(key);
    }
}

/// Read keyboard input
///
/// Attempts to read a key from the keyboard buffer.
/// Returns None if no key is available.
///
/// # Returns
///
/// * `Some(Key)` - If a key was pressed
/// * `None` - If no key is available
fn read_keyboard() -> Option<Key> {
    // TODO: Implement keyboard driver integration
    // This will interface with PS/2 or USB HID keyboard drivers
    // For now, return None (no input)
    None
}

/// Process a keyboard key
///
/// Handles the key based on the current application state.
///
/// # Arguments
///
/// * `key` - The key that was pressed
fn process_key(key: Key) {
    let mut state = GLOBAL_STATE.lock();
    if let Some(ref mut kernel_state) = *state {
        // If setup is not complete, pass to wizard
        if !kernel_state.setup_complete {
            // TODO: Pass to setup wizard once TUI is implemented
            // wizard.handle_input(key);
            return;
        }

        // Otherwise, handle in chat screen
        match key {
            Key::Char(ch) => {
                // TODO: Add character to input buffer
            }
            Key::Enter => {
                // TODO: Send message to LLM
                // let message = input_buffer.clone();
                // send_message(message);
            }
            Key::Backspace => {
                // TODO: Remove character from input buffer
            }
            Key::F1 => {
                // TODO: Show help screen
            }
            Key::F2 => {
                // TODO: Show provider selection screen
            }
            Key::F3 => {
                // TODO: Show model selection screen
            }
            Key::F4 => {
                // TODO: Show config screen
            }
            Key::F9 => {
                // TODO: Clear conversation (new chat)
                kernel_state.conversation.clear();
            }
            Key::F10 => {
                // TODO: Shutdown
                shutdown();
            }
            Key::Escape => {
                // TODO: Return to chat screen (if in another screen)
            }
            _ => {
                // Ignore other keys for now
            }
        }
    }
}

/// Shutdown the system
///
/// Performs a clean shutdown of the operating system.
fn shutdown() -> ! {
    // TODO: Implement proper shutdown
    // For now, just halt
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe");
        }
    }
}

// TODO: Implement message sending once LLM integration is complete
//
// /// Send a message to the LLM
// ///
// /// Adds the user message to the conversation and requests a completion
// /// from the current LLM provider.
// ///
// /// # Arguments
// ///
// /// * `text` - The message text
// fn send_message(text: String) {
//     let mut state = GLOBAL_STATE.lock();
//     if let Some(ref mut kernel_state) = *state {
//         // Add user message
//         kernel_state.conversation.push(Message {
//             role: Role::User,
//             content: text.clone(),
//         });
//
//         // Generate response (will be done async)
//         // TODO: Implement streaming response handling
//     }
// }
