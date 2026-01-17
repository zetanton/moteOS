#![no_std]
#![no_main]

//! moteOS Kernel - Main entry point and event loop
//!
//! This module implements the kernel_main() entry point and the main event loop
//! that drives the operating system, handling input, network, and screen updates.

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use boot::BootInfo;
use config::{ConfigStorage, EfiConfigStorage, MoteConfig};
use core::panic::PanicInfo;
use llm::{CompletionResult, GenerationConfig, LlmProvider, Message, Role};
use spin::Mutex;

pub mod event_loop;
pub mod init;
pub mod input;
pub mod screen;

// Global kernel state
static GLOBAL_STATE: Mutex<Option<KernelState>> = Mutex::new(None);

/// Kernel state structure
///
/// Holds all the state needed to run the operating system, including
/// network, configuration, screen, and conversation state.
pub struct KernelState {
    /// Configuration
    pub config: MoteConfig,
    /// Current conversation messages
    pub conversation: Vec<Message>,
    /// Whether setup has been completed
    pub setup_complete: bool,
}

impl KernelState {
    /// Create a new kernel state
    pub fn new(config: MoteConfig, setup_complete: bool) -> Self {
        Self {
            config,
            conversation: Vec::new(),
            setup_complete,
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
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: BootInfo) -> ! {
    // Initialize heap allocator
    init::init_heap(boot_info.heap_start, boot_info.heap_size);

    // Initialize framebuffer and screen
    // Note: This is a stub until TUI is implemented
    // let screen = Screen::new(boot_info.framebuffer, &DARK_THEME);

    // Load configuration
    let config_storage = EfiConfigStorage;
    let setup_complete = config_storage.exists();
    let config = match config_storage.load() {
        Ok(Some(cfg)) => cfg,
        Ok(None) | Err(_) => MoteConfig::default(),
    };

    // Initialize network (if configured)
    // Note: This will be called once network stack is complete
    // let network = init::init_network(&config).ok();

    // Initialize LLM provider (if configured)
    // Note: This will be called once LLM providers are implemented
    // let provider = init::init_provider(&config);

    // Set up global state
    {
        let mut state = GLOBAL_STATE.lock();
        *state = Some(KernelState::new(config, setup_complete));
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
