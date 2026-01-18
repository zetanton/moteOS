//! Screen update module
//!
//! This module handles updating the screen/framebuffer with the current
//! application state, including rendering chat messages, input, and UI elements.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use crate::GLOBAL_STATE;
use config::{ApiKeyProvider, WizardState};
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

    let bounds = kernel_state.screen.bounds();
    let Some((char_width, char_height)) = kernel_state.screen.char_size() else {
        return;
    };

    let theme = kernel_state.screen.theme();

    // Calculate center position
    let center_x = bounds.width / 2;
    let center_y = bounds.height / 2;

    // Helper to draw centered text
    let draw_centered = |screen: &mut tui::Screen, y: usize, text: &str, color| {
        let text_width = text.chars().count() * char_width;
        let x = center_x.saturating_sub(text_width / 2);
        screen.draw_text(x, y, text, color);
    };

    // Draw title
    let title = "moteOS Setup";
    draw_centered(&mut kernel_state.screen, char_height * 2, title, theme.accent_primary);

    // Render based on wizard state
    let wizard_state = kernel_state.wizard.state().clone();
    match wizard_state {
        WizardState::Welcome => {
            draw_centered(&mut kernel_state.screen, center_y - char_height, "Welcome to moteOS!", theme.text_primary);
            draw_centered(&mut kernel_state.screen, center_y + char_height, "Press ENTER to begin setup", theme.text_secondary);
            draw_centered(&mut kernel_state.screen, center_y + char_height * 3, "Press ESC to cancel", theme.text_tertiary);
        }
        WizardState::NetworkTypeSelect => {
            draw_centered(&mut kernel_state.screen, center_y - char_height * 3, "Select Network Type", theme.text_primary);
            draw_centered(&mut kernel_state.screen, center_y - char_height, "[1] Ethernet", theme.text_secondary);
            draw_centered(&mut kernel_state.screen, center_y + char_height, "[2] WiFi", theme.text_secondary);
            draw_centered(&mut kernel_state.screen, center_y + char_height * 4, "Press ESC to go back", theme.text_tertiary);
        }
        WizardState::NetworkScan { .. } => {
            draw_centered(&mut kernel_state.screen, center_y, "Scanning for WiFi networks...", theme.text_primary);
            draw_centered(&mut kernel_state.screen, center_y + char_height * 2, "Press ESC to go back", theme.text_tertiary);
        }
        WizardState::NetworkSelect { selected_index } => {
            draw_centered(&mut kernel_state.screen, center_y - char_height * 5, "Select WiFi Network", theme.text_primary);

            let networks = kernel_state.wizard.available_networks();
            let start_y = center_y - char_height * 2;
            for (i, network) in networks.iter().take(5).enumerate() {
                let prefix = if i == selected_index { "> " } else { "  " };
                let line = format!("{}{}", prefix, network.ssid);
                let color = if i == selected_index { theme.accent_primary } else { theme.text_secondary };
                draw_centered(&mut kernel_state.screen, start_y + i * char_height, &line, color);
            }

            draw_centered(&mut kernel_state.screen, center_y + char_height * 4, "Use UP/DOWN to select, ENTER to confirm", theme.text_tertiary);
        }
        WizardState::NetworkPassword { ref ssid } => {
            let title = format!("Enter password for: {}", ssid);
            draw_centered(&mut kernel_state.screen, center_y - char_height * 2, &title, theme.text_primary);

            // Show password input (masked)
            let input = kernel_state.wizard.input_buffer();
            let masked: String = "*".repeat(input.len());
            draw_centered(&mut kernel_state.screen, center_y, &masked, theme.text_secondary);

            draw_centered(&mut kernel_state.screen, center_y + char_height * 3, "Press ENTER to connect, ESC to go back", theme.text_tertiary);
        }
        WizardState::ApiKeyMenu => {
            draw_centered(&mut kernel_state.screen, center_y - char_height * 4, "Configure LLM Provider", theme.text_primary);
            draw_centered(&mut kernel_state.screen, center_y - char_height * 2, "[1] OpenAI", theme.text_secondary);
            draw_centered(&mut kernel_state.screen, center_y - char_height, "[2] Anthropic", theme.text_secondary);
            draw_centered(&mut kernel_state.screen, center_y, "[3] Groq", theme.text_secondary);
            draw_centered(&mut kernel_state.screen, center_y + char_height, "[4] xAI", theme.text_secondary);
            draw_centered(&mut kernel_state.screen, center_y + char_height * 3, "[S] Skip (use local model only)", theme.text_secondary);
            draw_centered(&mut kernel_state.screen, center_y + char_height * 5, "Press ESC to go back", theme.text_tertiary);
        }
        WizardState::ApiKeyInput { ref provider } => {
            let provider_name = match provider {
                ApiKeyProvider::OpenAI => "OpenAI",
                ApiKeyProvider::Anthropic => "Anthropic",
                ApiKeyProvider::Groq => "Groq",
                ApiKeyProvider::XAI => "xAI",
                ApiKeyProvider::Skip => "Skip",
            };
            let title = format!("Enter {} API Key", provider_name);
            draw_centered(&mut kernel_state.screen, center_y - char_height * 2, &title, theme.text_primary);

            // Show API key input (masked)
            let input = kernel_state.wizard.input_buffer();
            let masked: String = if input.is_empty() {
                String::from("(type your API key)")
            } else {
                "*".repeat(input.len())
            };
            draw_centered(&mut kernel_state.screen, center_y, &masked, theme.text_secondary);

            draw_centered(&mut kernel_state.screen, center_y + char_height * 3, "Press ENTER to save, ESC to go back", theme.text_tertiary);
        }
        WizardState::Ready { .. } => {
            draw_centered(&mut kernel_state.screen, center_y - char_height * 2, "Setup Complete!", theme.accent_success);
            draw_centered(&mut kernel_state.screen, center_y, "Press ENTER to save and start moteOS", theme.text_primary);
            draw_centered(&mut kernel_state.screen, center_y + char_height * 2, "Press ESC to go back and make changes", theme.text_tertiary);
        }
        WizardState::Complete => {
            draw_centered(&mut kernel_state.screen, center_y, "Starting moteOS...", theme.text_primary);
        }
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
