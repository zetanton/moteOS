//! Input handling module
//!
//! This module handles keyboard input from PS/2 or USB HID keyboards.
//! It reads keyboard events and dispatches them to the appropriate handlers.

use crate::GLOBAL_STATE;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use config::Key;
#[cfg(target_arch = "x86_64")]
use crate::ps2;
use llm::{GenerationConfig, Message, Role};
use tui::types::Key as TuiKey;

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
    #[cfg(target_arch = "x86_64")]
    {
        // Poll PS/2 keyboard for any pending scancodes
        // (in case interrupts aren't working or we're in polling mode)
        ps2::poll();
        
        // Read a key from the PS/2 keyboard buffer
        ps2::read_key()
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Other architectures would use different keyboard drivers
        None
    }
}

/// Convert config::Key to tui::types::Key
fn convert_key(key: Key) -> TuiKey {
    match key {
        Key::Char(ch) => TuiKey::Char(ch),
        Key::Enter => TuiKey::Enter,
        Key::Backspace => TuiKey::Backspace,
        Key::Esc => TuiKey::Escape,
        Key::Tab => TuiKey::Tab,
        Key::Up => TuiKey::Up,
        Key::Down => TuiKey::Down,
        Key::Left => TuiKey::Left,
        Key::Right => TuiKey::Right,
        Key::Delete => TuiKey::Delete,
        Key::F(n) => match n {
            1 => TuiKey::F1,
            2 => TuiKey::F2,
            3 => TuiKey::F3,
            4 => TuiKey::F4,
            5 => TuiKey::F5,
            6 => TuiKey::F6,
            7 => TuiKey::F7,
            8 => TuiKey::F8,
            9 => TuiKey::F9,
            10 => TuiKey::F10,
            11 => TuiKey::F11,
            12 => TuiKey::F12,
            _ => TuiKey::Escape,
        },
    }
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
        // If setup is not complete, handle setup wizard input
        if !kernel_state.setup_complete {
            // For now, any key completes setup (full wizard implementation pending)
            // In a full implementation, this would pass to SetupWizard::handle_input()
            kernel_state.setup_complete = true;
            return;
        }

        // Convert key to TUI key format
        let tui_key = convert_key(key);

        // Handle special function keys
        match tui_key {
            TuiKey::F1 => {
                // TODO: Show help screen
            }
            TuiKey::F2 => {
                // Switch provider
                switch_provider(kernel_state);
            }
            TuiKey::F3 => {
                // TODO: Show model selection screen
            }
            TuiKey::F4 => {
                // TODO: Show config screen
            }
            TuiKey::F9 => {
                // Clear conversation (new chat)
                kernel_state.conversation.clear();
                kernel_state.chat_screen = tui::screens::ChatScreen::new(
                    kernel_state.current_provider_name.clone(),
                    kernel_state.current_model.clone(),
                );
            }
            TuiKey::F10 => {
                // Shutdown
                shutdown();
            }
            TuiKey::Enter => {
                // Handle message submission through chat screen
                let event = kernel_state.chat_screen.handle_input(tui_key);
                if let tui::screens::ChatEvent::MessageSubmitted = event {
                    let message_text = kernel_state.chat_screen.input().get_text().to_string();
                    if !message_text.trim().is_empty() {
                        send_message(kernel_state, message_text);
                    }
                }
            }
            _ => {
                // Pass other keys to chat screen
                let event = kernel_state.chat_screen.handle_input(tui_key);
                match event {
                    tui::screens::ChatEvent::MessageSubmitted => {
                        let message_text = kernel_state.chat_screen.input().get_text().to_string();
                        if !message_text.trim().is_empty() {
                            send_message(kernel_state, message_text);
                        }
                    }
                    _ => {
                        // Other events are handled by the chat screen itself
                    }
                }
            }
        }
    }
}

/// Switch to a different LLM provider
///
/// Cycles through available providers or allows selection.
fn switch_provider(kernel_state: &mut crate::KernelState) {
    // TODO: Implement provider switching UI
    // For now, just cycle through available providers
    let providers = ["openai", "anthropic", "groq", "xai"];
    let current_idx = providers
        .iter()
        .position(|p| *p == kernel_state.current_provider_name.to_lowercase())
        .unwrap_or(0);
    let next_idx = (current_idx + 1) % providers.len();
    let next_provider = providers[next_idx];

    // Temporarily update config to use the next provider
    let mut temp_config = kernel_state.config.clone();
    temp_config.preferences.default_provider = next_provider.to_string();

    // Try to initialize the next provider
    match crate::init::init_provider(&temp_config, kernel_state.network.as_mut()) {
        Ok((provider, name, model)) => {
            kernel_state.current_provider = provider;
            kernel_state.current_provider_name = name;
            kernel_state.current_model = model;
            kernel_state.chat_screen.set_provider(kernel_state.current_provider_name.clone());
            kernel_state.chat_screen.set_model(kernel_state.current_model.clone());
            // Update config to persist the change
            kernel_state.config.preferences.default_provider = next_provider.to_string();
        }
        Err(_) => {
            // Failed to switch, keep current provider
        }
    }
}

/// Send a message to the LLM
///
/// Adds the user message to the conversation and requests a completion
/// from the current LLM provider with streaming support.
///
/// # Arguments
///
/// * `kernel_state` - Mutable reference to kernel state
/// * `text` - The message text
fn send_message(kernel_state: &mut crate::KernelState, text: String) {
    // Don't send if already generating
    if kernel_state.is_generating {
        return;
    }

    // Add user message to conversation
    let user_message = Message::new(Role::User, text.clone());
    kernel_state.conversation.push(user_message.clone());

    // Add message to chat screen
    kernel_state
        .chat_screen
        .add_message(tui::widgets::MessageRole::User, text.clone());

    // Mark as generating
    kernel_state.is_generating = true;
    kernel_state
        .chat_screen
        .set_status(tui::screens::ConnectionStatus::Connected);

    // Create assistant message placeholder
    kernel_state.chat_screen.add_message(
        tui::widgets::MessageRole::Assistant,
        String::new(),
    );

    // Generate response with streaming
    let mut response_text = String::new();
    let config = GenerationConfig {
        temperature: kernel_state.config.preferences.temperature,
        max_tokens: None,
        stop_sequences: Vec::new(),
        top_p: None,
        top_k: None,
    };

    let mut on_token = |token: &str| {
        // Stream token to chat screen
        response_text.push_str(token);
        kernel_state
            .chat_screen
            .update_last_message(&response_text);
    };
    let result = kernel_state.current_provider.complete(
        &kernel_state.conversation,
        &kernel_state.current_model,
        &config,
        &mut on_token,
    );

    // Mark as no longer generating
    kernel_state.is_generating = false;

    // Handle result
    match result {
        Ok(completion_result) => {
            // Add assistant message to conversation
            kernel_state.conversation.push(Message::new(
                Role::Assistant,
                completion_result.text.clone(),
            ));

            // Update status
            kernel_state
                .chat_screen
                .set_status(tui::screens::ConnectionStatus::Connected);
        }
        Err(e) => {
            // Show error
            let error_msg = format!("Error: {:?}", e);
            kernel_state
                .chat_screen
                .set_status(tui::screens::ConnectionStatus::Error(error_msg));
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
