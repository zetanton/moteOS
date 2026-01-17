//! PS/2 Keyboard Driver
//!
//! This module provides a PS/2 keyboard driver for x86_64 systems.
//! It handles scancode reading, make/break code processing, and
//! conversion to the config::Key format.

#![cfg(target_arch = "x86_64")]

extern crate alloc;

use config::Key;
use spin::Mutex;
use alloc::collections::VecDeque;

/// PS/2 controller ports
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

/// PS/2 status register bits
const STATUS_OUTPUT_FULL: u8 = 0x01;  // Data available in output buffer
const STATUS_INPUT_FULL: u8 = 0x02;   // Input buffer full
const STATUS_SYSTEM_FLAG: u8 = 0x04;  // System flag
const STATUS_COMMAND_DATA: u8 = 0x08; // Command (1) or data (0)
const STATUS_TIMEOUT: u8 = 0x40;      // Timeout error
const STATUS_PARITY_ERROR: u8 = 0x80; // Parity error

/// PS/2 scancode constants
const SCANCODE_BREAK_PREFIX: u8 = 0xF0;
const SCANCODE_EXTENDED_PREFIX: u8 = 0xE0;

/// Global keyboard buffer
static KEY_BUFFER: Mutex<VecDeque<Key>> = Mutex::new(VecDeque::new());

/// Initialize the PS/2 keyboard
///
/// This function should be called during kernel initialization.
/// It enables the keyboard and clears any pending scancodes.
pub fn init() {
    // Clear the key buffer
    let mut buffer = KEY_BUFFER.lock();
    buffer.clear();
    
    // Enable keyboard interrupts (done via PIC configuration)
    // The keyboard should already be enabled by the bootloader
}

/// Check if a scancode is available
///
/// Returns true if the PS/2 controller has data ready to read.
pub fn has_scancode() -> bool {
    unsafe {
        let status = Port::<u8>::new(PS2_STATUS_PORT).read();
        (status & STATUS_OUTPUT_FULL) != 0
    }
}

/// Read a scancode from the PS/2 controller
///
/// Returns Some(scancode) if data is available, None otherwise.
/// This function does not block.
pub fn read_scancode() -> Option<u8> {
    if !has_scancode() {
        return None;
    }
    
    unsafe {
        let scancode = Port::<u8>::new(PS2_DATA_PORT).read();
        Some(scancode)
    }
}

/// Process a scancode and convert it to a Key
///
/// Handles make/break codes and extended scancodes.
/// Only processes make codes (key presses), ignoring break codes (key releases).
///
/// # Arguments
///
/// * `scancode` - The raw scancode from the keyboard
/// * `extended` - Whether this is an extended scancode (0xE0 prefix)
///
/// # Returns
///
/// Some(Key) if the scancode represents a key press, None otherwise.
fn process_scancode(scancode: u8, extended: bool) -> Option<Key> {
    // Handle break codes (key release) - we ignore these
    if scancode == SCANCODE_BREAK_PREFIX {
        return None;
    }
    
    // Handle extended prefix
    if scancode == SCANCODE_EXTENDED_PREFIX {
        return None; // Will be handled by caller
    }
    
    // Convert scancode to Key
    match (extended, scancode) {
        // Regular keys (US QWERTY layout)
        (false, 0x1C) => Some(Key::Enter),
        (false, 0x0E) => Some(Key::Backspace),
        (false, 0x01) => Some(Key::Esc),
        (false, 0x0F) => Some(Key::Tab),
        (false, 0x48) => Some(Key::Up),
        (false, 0x50) => Some(Key::Down),
        (false, 0x4B) => Some(Key::Left),
        (false, 0x4D) => Some(Key::Right),
        (false, 0x53) => Some(Key::Delete),
        
        // Function keys
        (false, 0x3B) => Some(Key::F(1)),
        (false, 0x3C) => Some(Key::F(2)),
        (false, 0x3D) => Some(Key::F(3)),
        (false, 0x3E) => Some(Key::F(4)),
        (false, 0x3F) => Some(Key::F(5)),
        (false, 0x40) => Some(Key::F(6)),
        (false, 0x41) => Some(Key::F(7)),
        (false, 0x42) => Some(Key::F(8)),
        (false, 0x43) => Some(Key::F(9)),
        (false, 0x44) => Some(Key::F(10)),
        (false, 0x57) => Some(Key::F(11)),
        (false, 0x58) => Some(Key::F(12)),
        
        // Extended keys
        (true, 0x53) => Some(Key::Delete), // Delete (extended)
        (true, 0x48) => Some(Key::Up),     // Up arrow (extended)
        (true, 0x50) => Some(Key::Down),    // Down arrow (extended)
        (true, 0x4B) => Some(Key::Left),    // Left arrow (extended)
        (true, 0x4D) => Some(Key::Right),   // Right arrow (extended)
        
        // Character keys (US QWERTY layout)
        (false, 0x02) => Some(Key::Char('1')),
        (false, 0x03) => Some(Key::Char('2')),
        (false, 0x04) => Some(Key::Char('3')),
        (false, 0x05) => Some(Key::Char('4')),
        (false, 0x06) => Some(Key::Char('5')),
        (false, 0x07) => Some(Key::Char('6')),
        (false, 0x08) => Some(Key::Char('7')),
        (false, 0x09) => Some(Key::Char('8')),
        (false, 0x0A) => Some(Key::Char('9')),
        (false, 0x0B) => Some(Key::Char('0')),
        (false, 0x0C) => Some(Key::Char('-')),
        (false, 0x0D) => Some(Key::Char('=')),
        
        (false, 0x10) => Some(Key::Char('q')),
        (false, 0x11) => Some(Key::Char('w')),
        (false, 0x12) => Some(Key::Char('e')),
        (false, 0x13) => Some(Key::Char('r')),
        (false, 0x14) => Some(Key::Char('t')),
        (false, 0x15) => Some(Key::Char('y')),
        (false, 0x16) => Some(Key::Char('u')),
        (false, 0x17) => Some(Key::Char('i')),
        (false, 0x18) => Some(Key::Char('o')),
        (false, 0x19) => Some(Key::Char('p')),
        (false, 0x1A) => Some(Key::Char('[')),
        (false, 0x1B) => Some(Key::Char(']')),
        
        (false, 0x1E) => Some(Key::Char('a')),
        (false, 0x1F) => Some(Key::Char('s')),
        (false, 0x20) => Some(Key::Char('d')),
        (false, 0x21) => Some(Key::Char('f')),
        (false, 0x22) => Some(Key::Char('g')),
        (false, 0x23) => Some(Key::Char('h')),
        (false, 0x24) => Some(Key::Char('j')),
        (false, 0x25) => Some(Key::Char('k')),
        (false, 0x26) => Some(Key::Char('l')),
        (false, 0x27) => Some(Key::Char(';')),
        (false, 0x28) => Some(Key::Char('\'')),
        (false, 0x29) => Some(Key::Char('`')),
        
        (false, 0x2B) => Some(Key::Char('\\')),
        (false, 0x2C) => Some(Key::Char('z')),
        (false, 0x2D) => Some(Key::Char('x')),
        (false, 0x2E) => Some(Key::Char('c')),
        (false, 0x2F) => Some(Key::Char('v')),
        (false, 0x30) => Some(Key::Char('b')),
        (false, 0x31) => Some(Key::Char('n')),
        (false, 0x32) => Some(Key::Char('m')),
        (false, 0x33) => Some(Key::Char(',')),
        (false, 0x34) => Some(Key::Char('.')),
        (false, 0x35) => Some(Key::Char('/')),
        
        (false, 0x39) => Some(Key::Char(' ')), // Space
        
        // Shifted characters (we'll handle shift in the interrupt handler if needed)
        // For now, we'll just return the base character
        _ => None,
    }
}

/// Handle a keyboard scancode (called from interrupt handler)
///
/// This function processes scancodes and adds keys to the buffer.
/// It handles make/break codes and extended scancodes.
///
/// # Arguments
///
/// * `scancode` - The raw scancode from the keyboard
pub fn handle_scancode(scancode: u8) {
    static mut STATE: ScancodeState = ScancodeState::Normal;
    static mut EXTENDED: bool = false;
    
    match unsafe { *STATE } {
        ScancodeState::Normal => {
            if scancode == SCANCODE_EXTENDED_PREFIX {
                unsafe { EXTENDED = true; }
                return;
            } else if scancode == SCANCODE_BREAK_PREFIX {
                // Check if we're in extended mode
                if unsafe { EXTENDED } {
                    unsafe { *STATE = ScancodeState::ExtendedBreak; }
                } else {
                    unsafe { *STATE = ScancodeState::Break; }
                }
                return;
            } else {
                // Regular make code
                if let Some(key) = process_scancode(scancode, unsafe { EXTENDED }) {
                    let mut buffer = KEY_BUFFER.lock();
                    buffer.push_back(key);
                }
                unsafe { EXTENDED = false; }
            }
        }
        ScancodeState::Break => {
            // This is the key that was released - we ignore it
            unsafe {
                *STATE = ScancodeState::Normal;
                EXTENDED = false;
            }
        }
        ScancodeState::ExtendedBreak => {
            // Extended break code: E0 F0 <scancode>
            // This is the actual scancode being released - ignore it
            unsafe {
                *STATE = ScancodeState::Normal;
                EXTENDED = false;
            }
        }
    }
}

/// Internal state for scancode processing
#[derive(Clone, Copy)]
enum ScancodeState {
    Normal,
    Break,
    ExtendedBreak,
}

/// Read a key from the keyboard buffer
///
/// Returns Some(Key) if a key is available, None otherwise.
/// This function does not block.
pub fn read_key() -> Option<Key> {
    let mut buffer = KEY_BUFFER.lock();
    buffer.pop_front()
}

/// Poll the keyboard for input
///
/// This function should be called periodically to check for keyboard input.
/// It reads scancodes and processes them into keys.
pub fn poll() {
    while let Some(scancode) = read_scancode() {
        handle_scancode(scancode);
    }
}

/// Port I/O wrapper for x86_64
struct Port<T> {
    port: u16,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> Port<T> {
    const fn new(port: u16) -> Self {
        Self {
            port,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl Port<u8> {
    unsafe fn read(&self) -> u8 {
        let value: u8;
        core::arch::asm!("in al, dx", out("al") value, in("dx") self.port);
        value
    }
    
    unsafe fn write(&self, value: u8) {
        core::arch::asm!("out dx, al", in("dx") self.port, in("al") value);
    }
}
