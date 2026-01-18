//! Screen update module
//!
//! This module handles updating the screen/framebuffer with the current
//! application state, including rendering chat messages, input, and UI elements.

use crate::GLOBAL_STATE;
#[cfg(target_arch = "x86_64")]
use crate::ps2;

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

/// Track if we need a FULL redraw (clear + redraw everything)
static NEEDS_FULL_REDRAW: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);

/// Track if we need to update (redraw without clear - for input changes)
static NEEDS_UPDATE: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Mark screen as needing full redraw (clear + redraw)
pub fn mark_dirty() {
    NEEDS_FULL_REDRAW.store(true, core::sync::atomic::Ordering::Relaxed);
}

/// Mark screen as needing update (redraw without clear - faster for input)
pub fn mark_needs_update() {
    NEEDS_UPDATE.store(true, core::sync::atomic::Ordering::Relaxed);
}

/// Render the setup wizard screen
///
/// Displays the setup wizard UI for initial configuration.
fn render_setup_wizard(kernel_state: &mut crate::KernelState) {
    // Check if we need any redraw
    let needs_full = NEEDS_FULL_REDRAW.swap(false, core::sync::atomic::Ordering::Relaxed);
    let needs_update = NEEDS_UPDATE.swap(false, core::sync::atomic::Ordering::Relaxed);

    if !needs_full && !needs_update {
        return;
    }

    // Only clear on full redraw
    if needs_full {
        kernel_state.screen.clear();
    }
    
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

    // Show PS/2 debug info to confirm input is arriving
    let debug_y = inst_y + char_height * 2;
    let debug_x = bounds.x + char_width;
    #[cfg(target_arch = "x86_64")]
    {
        let (last, len, pending) = ps2::debug_snapshot();
        let last_text = match last {
            Some(code) => {
                let mut text = alloc::string::String::from("PS/2: last=0x");
                use alloc::string::ToString;
                let hi = (code >> 4) & 0x0F;
                let lo = code & 0x0F;
                text.push(
                    core::char::from_digit(hi as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
                text.push(
                    core::char::from_digit(lo as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
                text.push_str(" buf=");
                text.push_str(&len.to_string());
                text.push_str(" pending=");
                text.push_str(if pending { "1" } else { "0" });
                text
            }
            None => alloc::string::String::from("PS/2: last=none buf=0 pending=0"),
        };
        kernel_state
            .screen
            .draw_text(debug_x, debug_y, &last_text, theme.text_tertiary);
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        kernel_state
            .screen
            .draw_text(debug_x, debug_y, "PS/2: n/a", theme.text_tertiary);
    }
}

/// Render the chat screen
///
/// Displays the main chat interface with conversation history and input.
fn render_chat_screen(kernel_state: &mut crate::KernelState) {
    // Check if we need any redraw
    let needs_full = NEEDS_FULL_REDRAW.swap(false, core::sync::atomic::Ordering::Relaxed);
    let needs_update = NEEDS_UPDATE.swap(false, core::sync::atomic::Ordering::Relaxed);

    if !needs_full && !needs_update {
        return;
    }

    // For partial updates (input changes), only redraw the input area
    if needs_update && !needs_full {
        kernel_state.chat_screen.render_input_only(&mut kernel_state.screen);
        return;
    }

    // Full redraw: clear and render everything
    kernel_state.screen.clear();

    // Update connection status based on network state
    let status = if kernel_state.network.is_some() {
        tui::screens::ConnectionStatus::Connected
    } else {
        tui::screens::ConnectionStatus::Disconnected
    };
    kernel_state.chat_screen.set_status(status);

    // Render the full chat screen
    kernel_state.chat_screen.render(&mut kernel_state.screen);
}
