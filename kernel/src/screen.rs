//! Screen update module
//!
//! This module handles updating the screen/framebuffer with the current
//! application state, including rendering chat messages, input, and UI elements.

use crate::GLOBAL_STATE;

/// Update the screen
///
/// Renders the current application state to the framebuffer.
/// This is called from the main event loop.
pub fn update_screen() {
    let state = GLOBAL_STATE.lock();
    if let Some(ref kernel_state) = *state {
        // Determine what to render based on state
        if !kernel_state.setup_complete {
            // Render setup wizard
            render_setup_wizard();
        } else {
            // Render chat screen
            render_chat_screen();
        }
    }
}

/// Render the setup wizard screen
///
/// Displays the setup wizard UI for initial configuration.
fn render_setup_wizard() {
    // TODO: Implement once TUI framework is complete
    // This will:
    // 1. Clear the screen
    // 2. Render wizard UI elements
    // 3. Render current wizard state
    // 4. Present the framebuffer
}

/// Render the chat screen
///
/// Displays the main chat interface with conversation history and input.
fn render_chat_screen() {
    // TODO: Implement once TUI framework is complete
    // This will:
    // 1. Clear the screen
    // 2. Render header (provider, model, status)
    // 3. Render conversation messages
    // 4. Render input area
    // 5. Render footer (hotkeys)
    // 6. Present the framebuffer
}

// TODO: Add more screen rendering functions as screens are added
//
// /// Render the help screen
// fn render_help_screen() { ... }
//
// /// Render the provider selection screen
// fn render_provider_select() { ... }
//
// /// Render the model selection screen
// fn render_model_select() { ... }
//
// /// Render the configuration screen
// fn render_config_screen() { ... }
