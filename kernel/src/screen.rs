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
    let mut state = GLOBAL_STATE.lock();
    if let Some(ref mut kernel_state) = *state {
        // Determine what to render based on state
        if !kernel_state.setup_complete {
            // Render setup wizard
            render_setup_wizard(kernel_state);
        } else {
            // Render chat screen
            render_chat_screen(kernel_state);
        }
    }
}

/// Render the setup wizard screen
///
/// Displays the setup wizard UI for initial configuration.
fn render_setup_wizard(_kernel_state: &mut crate::KernelState) {
    // TODO: Implement once TUI framework is complete
    // This will:
    // 1. Clear the screen
    // 2. Render wizard UI elements
    // 3. Render current wizard state
    // 4. Present the framebuffer
    
    // For now, just clear the screen
    // kernel_state.screen.clear();
}

/// Render the chat screen
///
/// Displays the main chat interface with conversation history and input.
fn render_chat_screen(kernel_state: &mut crate::KernelState) {
    // Clear the screen
    kernel_state.screen.clear();
    
    // Update connection status based on network state
    let status = if kernel_state.network.is_some() {
        tui::screens::ConnectionStatus::Connected
    } else {
        tui::screens::ConnectionStatus::Disconnected
    };
    kernel_state.chat_screen.set_status(status);
    
    // Render the chat screen
    kernel_state.chat_screen.render(&mut kernel_state.screen);
    
    // Note: Screen presentation is handled by the TUI framework
}
