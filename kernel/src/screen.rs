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
fn render_setup_wizard(kernel_state: &mut crate::KernelState) {
    // Clear the screen
    kernel_state.screen.clear();
    
    // TODO: Implement full wizard UI once wizard screen is integrated
    // For now, display a simple message indicating setup is needed
    let bounds = kernel_state.screen.bounds();
    let Some((char_width, char_height)) = kernel_state.screen.char_size() else {
        return;
    };
    
    let theme = kernel_state.screen.theme();
    let welcome_text = "Welcome to moteOS Setup";
    let text_width = welcome_text.chars().count() * char_width;
    let text_x = bounds.x + (bounds.width / 2) - (text_width / 2);
    let text_y = bounds.y + (bounds.height / 2);
    
    kernel_state.screen.draw_text(text_x, text_y, welcome_text, theme.text_primary);
    
    let instruction_text = "Press any key to continue...";
    let inst_width = instruction_text.chars().count() * char_width;
    let inst_x = bounds.x + (bounds.width / 2) - (inst_width / 2);
    let inst_y = text_y + char_height * 2;
    
    kernel_state.screen.draw_text(inst_x, inst_y, instruction_text, theme.text_secondary);
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
