#![no_std]
#![no_main]

//! moteOS Kernel - Main entry point and event loop
//!
//! This module implements the kernel_main() entry point and the main event loop
//! that drives the operating system, handling input, network, and screen updates.

#[cfg(not(feature = "uefi-minimal"))]
extern crate alloc;

#[cfg(feature = "uefi-minimal")]
use shared::{BootInfo, Color, Rect};

#[cfg(not(feature = "uefi-minimal"))]
use alloc::boxed::Box;
#[cfg(not(feature = "uefi-minimal"))]
use alloc::string::String;
#[cfg(not(feature = "uefi-minimal"))]
use alloc::vec::Vec;
#[cfg(not(feature = "uefi-minimal"))]
use config::{decrypt_api_key, ConfigStorage, EfiConfigStorage, MoteConfig};
use core::panic::PanicInfo;
#[cfg(not(feature = "uefi-minimal"))]
use llm::{GenerationConfig, LlmProvider, Message, Role};
#[cfg(not(feature = "uefi-minimal"))]
use network::{poll_network_stack, NetworkStack};
#[cfg(not(feature = "uefi-minimal"))]
use shared::BootInfo;
#[cfg(not(feature = "uefi-minimal"))]
use spin::Mutex;
#[cfg(not(feature = "uefi-minimal"))]
use tui::{screens::ChatScreen, Screen, Theme, DARK_THEME, LIGHT_THEME};

#[cfg(not(feature = "uefi-minimal"))]
pub mod event_loop;
#[cfg(not(feature = "uefi-minimal"))]
pub mod init;
#[cfg(not(feature = "uefi-minimal"))]
pub mod input;
#[cfg(not(feature = "uefi-minimal"))]
pub mod screen;
pub mod serial;

// Global kernel state
#[cfg(not(feature = "uefi-minimal"))]
static GLOBAL_STATE: Mutex<Option<KernelState>> = Mutex::new(None);

/// Kernel state structure
///
/// Holds all the state needed to run the operating system, including
/// network, configuration, screen, and conversation state.
#[cfg(not(feature = "uefi-minimal"))]
pub struct KernelState {
    /// Screen for rendering
    pub screen: Screen,
    /// Network stack (optional, None if not configured or failed to initialize)
    pub network: Option<NetworkStack>,
    /// Configuration
    pub config: MoteConfig,
    /// Current LLM provider
    pub current_provider: Box<dyn LlmProvider>,
    /// Current provider name
    pub current_provider_name: String,
    /// Current model name
    pub current_model: String,
    /// Chat screen state
    pub chat_screen: ChatScreen,
    /// Current conversation messages
    pub conversation: Vec<Message>,
    /// Whether setup has been completed
    pub setup_complete: bool,
    /// Whether we're currently generating a response
    pub is_generating: bool,
}

#[cfg(not(feature = "uefi-minimal"))]
impl KernelState {
    /// Create a new kernel state
    pub fn new(
        screen: Screen,
        network: Option<NetworkStack>,
        config: MoteConfig,
        provider: Box<dyn LlmProvider>,
        provider_name: String,
        model: String,
        setup_complete: bool,
    ) -> Self {
        let chat_screen = ChatScreen::new(provider_name.clone(), model.clone());
        Self {
            screen,
            network,
            config,
            current_provider: provider,
            current_provider_name: provider_name,
            current_model: model,
            chat_screen,
            conversation: Vec::new(),
            setup_complete,
            is_generating: false,
        }
    }
}

/// Kernel main entry point
///
/// This is called by the bootloader after setting up memory, interrupts,
/// framebuffer, and other basic hardware. It initializes the kernel and
/// enters the main event loop.
///
/// # Arguments
///
/// * `boot_info` - Boot information from the bootloader
///
/// # Panics
///
/// This function never returns normally. It will panic if initialization fails.
#[cfg(feature = "uefi-minimal")]
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: BootInfo) -> ! {
    serial::println("moteOS: kernel_main reached (minimal)");

    let fb = boot_info.framebuffer;
    let bounds = Rect::new(0, 0, fb.width, fb.height);
    fb.fill_rectangle_safe(bounds, Color::rgb(16, 16, 16));

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

#[cfg(not(feature = "uefi-minimal"))]
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: BootInfo) -> ! {
    // Initialize heap allocator
    init::init_heap(boot_info.heap_start, boot_info.heap_size);

    // Load configuration
    let config_storage = EfiConfigStorage;
    let setup_complete = config_storage.exists();
    let config = match config_storage.load() {
        Ok(Some(cfg)) => cfg,
        Ok(None) | Err(_) => MoteConfig::default(),
    };

    // Initialize framebuffer and screen
    let theme = match config.preferences.theme {
        config::ThemeChoice::Dark => &DARK_THEME,
        config::ThemeChoice::Light => &LIGHT_THEME,
    };
    let screen = unsafe { Screen::new(boot_info.framebuffer.into(), theme) };

    // Initialize network (if configured)
    let network = init::init_network(&config).ok();

    // Initialize LLM provider
    let (provider, provider_name, model) = match init::init_provider(&config, network.as_ref()) {
        Ok((p, name, m)) => (p, name, m),
        Err(_) => {
            // Fallback to a dummy provider or panic
            // For now, we'll create a minimal provider that will fail gracefully
            panic!("Failed to initialize LLM provider");
        }
    };

    // Set up global state
    {
        let mut state = GLOBAL_STATE.lock();
        *state = Some(KernelState::new(
            screen,
            network,
            config,
            provider,
            provider_name,
            model,
            setup_complete,
        ));
    }

    // Enter main event loop
    event_loop::main_loop();
}

/// Panic handler
///
/// Called when the kernel panics. Prints panic information to the
/// framebuffer and halts the CPU.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO: Print panic message to framebuffer
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
