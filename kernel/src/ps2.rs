//! PS/2 Keyboard Driver
//!
//! This module provides a PS/2 keyboard driver for x86_64 systems.
//! It handles scancode reading, make/break code processing, and
//! conversion to the config::Key format.

#![no_std]
#![cfg(target_arch = "x86_64")]

extern crate alloc;

use alloc::collections::VecDeque;
use config::Key;
use spin::Mutex;

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

/// Scancode processor state guarded by a mutex so it is safe in interrupt context.
static SCANCODE_PROCESSOR: Mutex<ScancodeProcessor> = Mutex::new(ScancodeProcessor::new());
/// Last raw scancode seen (for debug overlay)
static LAST_SCANCODE: Mutex<Option<u8>> = Mutex::new(None);

/// Initialize the PS/2 keyboard
///
/// This function should be called during kernel initialization.
/// It performs a full controller reset to ensure clean state after UEFI,
/// then enables the keyboard and clears any pending scancodes.
pub fn init() {
    crate::serial::println("PS/2: init starting");

    // Clear the key buffer
    let mut buffer = KEY_BUFFER.lock();
    buffer.clear();
    drop(buffer);

    // Aggressive PS/2 controller reset for post-UEFI state.
    // UEFI firmware may have left the controller in an undefined state.
    unsafe {
        let status = Port::<u8>::new(PS2_STATUS_PORT);
        let command = Port::<u8>::new(PS2_COMMAND_PORT);
        let data = Port::<u8>::new(PS2_DATA_PORT);

        // Step 1: Disable both PS/2 ports to prevent interference
        wait_input_empty();
        command.write(0xAD); // Disable first PS/2 port
        wait_input_empty();
        command.write(0xA7); // Disable second PS/2 port (if exists)

        // Step 2: Flush the output buffer completely
        for _ in 0..64 {
            if status.read() & STATUS_OUTPUT_FULL != 0 {
                let _ = data.read();
            }
            // Small delay between reads
            for _ in 0..100 {
                core::hint::spin_loop();
            }
        }

        // Step 3: Controller self-test
        wait_input_empty();
        command.write(0xAA); // Self-test command
        if wait_output_full() {
            let result = data.read();
            // 0x55 = test passed, 0xFC = test failed
            // Continue regardless - some controllers don't respond properly
            let _ = result;
        }

        // Step 4: Test first PS/2 port
        wait_input_empty();
        command.write(0xAB); // Test first port
        if wait_output_full() {
            let result = data.read();
            // 0x00 = test passed
            let _ = result;
        }

        // Step 5: Enable first PS/2 port
        wait_input_empty();
        command.write(0xAE);

        // Step 6: Read and configure controller config byte
        wait_input_empty();
        command.write(0x20); // Read config byte
        let mut config_byte = 0u8;
        if wait_output_full() {
            config_byte = data.read();
        }

        // Enable first port interrupt (bit 0)
        // Disable translation (bit 6) - we handle both Set 1 and Set 2
        config_byte |= 0x01;      // Enable port 1 interrupt
        config_byte &= !(1 << 6); // Disable translation

        // Write controller config byte back
        wait_input_empty();
        command.write(0x60); // Write config byte
        wait_input_empty();
        data.write(config_byte);

        // Step 7: Reset keyboard device
        wait_input_empty();
        data.write(0xFF); // Reset command
        // Wait for ACK (0xFA) then self-test result (0xAA)
        if wait_output_full() {
            let _ = data.read(); // ACK or error
        }
        if wait_output_full() {
            let _ = data.read(); // Self-test result
        }

        // Step 8: Set scancode set 2 (native PS/2)
        wait_input_empty();
        data.write(0xF0); // Set scancode set command
        let _ = read_ack();
        wait_input_empty();
        data.write(0x02); // Scancode set 2
        let _ = read_ack();

        // Step 9: Enable keyboard scanning
        wait_input_empty();
        data.write(0xF4); // Enable scanning
        let _ = read_ack();

        // Final flush of any remaining data
        for _ in 0..16 {
            if status.read() & STATUS_OUTPUT_FULL != 0 {
                let _ = data.read();
            }
        }
    }

    crate::serial::println("PS/2: init complete - keyboard should be ready");
}

fn wait_input_empty() {
    let status = Port::<u8>::new(PS2_STATUS_PORT);
    for _ in 0..10000 {
        unsafe {
            if status.read() & STATUS_INPUT_FULL == 0 {
                break;
            }
        }
    }
}

fn wait_output_full() -> bool {
    let status = Port::<u8>::new(PS2_STATUS_PORT);
    for _ in 0..10000 {
        unsafe {
            if status.read() & STATUS_OUTPUT_FULL != 0 {
                return true;
            }
        }
    }
    false
}

fn read_ack() -> bool {
    let data = Port::<u8>::new(PS2_DATA_PORT);
    if wait_output_full() {
        let ack = unsafe { data.read() };
        ack == 0xFA
    } else {
        false
    }
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
    if scancode == 0xFA {
        // ACK from keyboard
        return None;
    }
    if scancode == SCANCODE_BREAK_PREFIX {
        return None;
    }
    
    // Handle extended prefix
    if scancode == SCANCODE_EXTENDED_PREFIX {
        return None; // Will be handled by caller
    }
    
    // Try Set 2 scancodes first (typical PS/2 mode)
    if let Some(key) = map_set2(scancode, extended) {
        return Some(key);
    }

    // Handle Set 1 break codes (high bit set) after failing Set 2
    if scancode & 0x80 != 0 {
        let make = scancode & 0x7F;
        if map_set1(make, extended).is_some() {
            return None;
        }
    }

    // Fallback to Set 1 mapping (translated mode)
    map_set1(scancode, extended)
}

fn map_set2(scancode: u8, extended: bool) -> Option<Key> {
    match (extended, scancode) {
        // Regular keys (Set 2 scancodes, US QWERTY)
        (false, 0x5A) => Some(Key::Enter),
        (false, 0x66) => Some(Key::Backspace),
        (false, 0x76) => Some(Key::Esc),
        (false, 0x0D) => Some(Key::Tab),

        // Function keys (Set 2)
        (false, 0x05) => Some(Key::F(1)),
        (false, 0x06) => Some(Key::F(2)),
        (false, 0x04) => Some(Key::F(3)),
        (false, 0x0C) => Some(Key::F(4)),
        (false, 0x03) => Some(Key::F(5)),
        (false, 0x0B) => Some(Key::F(6)),
        (false, 0x83) => Some(Key::F(7)),
        (false, 0x0A) => Some(Key::F(8)),
        (false, 0x01) => Some(Key::F(9)),
        (false, 0x09) => Some(Key::F(10)),
        (false, 0x78) => Some(Key::F(11)),
        (false, 0x07) => Some(Key::F(12)),

        // Extended keys (Set 2)
        (true, 0x71) => Some(Key::Delete),
        (true, 0x75) => Some(Key::Up),
        (true, 0x72) => Some(Key::Down),
        (true, 0x6B) => Some(Key::Left),
        (true, 0x74) => Some(Key::Right),

        // Character keys (Set 2)
        (false, 0x16) => Some(Key::Char('1')),
        (false, 0x1E) => Some(Key::Char('2')),
        (false, 0x26) => Some(Key::Char('3')),
        (false, 0x25) => Some(Key::Char('4')),
        (false, 0x2E) => Some(Key::Char('5')),
        (false, 0x36) => Some(Key::Char('6')),
        (false, 0x3D) => Some(Key::Char('7')),
        (false, 0x3E) => Some(Key::Char('8')),
        (false, 0x46) => Some(Key::Char('9')),
        (false, 0x45) => Some(Key::Char('0')),
        (false, 0x4E) => Some(Key::Char('-')),
        (false, 0x55) => Some(Key::Char('=')),

        (false, 0x15) => Some(Key::Char('q')),
        (false, 0x1D) => Some(Key::Char('w')),
        (false, 0x24) => Some(Key::Char('e')),
        (false, 0x2D) => Some(Key::Char('r')),
        (false, 0x2C) => Some(Key::Char('t')),
        (false, 0x35) => Some(Key::Char('y')),
        (false, 0x3C) => Some(Key::Char('u')),
        (false, 0x43) => Some(Key::Char('i')),
        (false, 0x44) => Some(Key::Char('o')),
        (false, 0x4D) => Some(Key::Char('p')),
        (false, 0x54) => Some(Key::Char('[')),
        (false, 0x5B) => Some(Key::Char(']')),

        (false, 0x1C) => Some(Key::Char('a')),
        (false, 0x1B) => Some(Key::Char('s')),
        (false, 0x23) => Some(Key::Char('d')),
        (false, 0x2B) => Some(Key::Char('f')),
        (false, 0x34) => Some(Key::Char('g')),
        (false, 0x33) => Some(Key::Char('h')),
        (false, 0x3B) => Some(Key::Char('j')),
        (false, 0x42) => Some(Key::Char('k')),
        (false, 0x4B) => Some(Key::Char('l')),
        (false, 0x4C) => Some(Key::Char(';')),
        (false, 0x52) => Some(Key::Char('\'')),
        (false, 0x0E) => Some(Key::Char('`')),

        (false, 0x5D) => Some(Key::Char('\\')),
        (false, 0x1A) => Some(Key::Char('z')),
        (false, 0x22) => Some(Key::Char('x')),
        (false, 0x21) => Some(Key::Char('c')),
        (false, 0x2A) => Some(Key::Char('v')),
        (false, 0x32) => Some(Key::Char('b')),
        (false, 0x31) => Some(Key::Char('n')),
        (false, 0x3A) => Some(Key::Char('m')),
        (false, 0x41) => Some(Key::Char(',')),
        (false, 0x49) => Some(Key::Char('.')),
        (false, 0x4A) => Some(Key::Char('/')),

        (false, 0x29) => Some(Key::Char(' ')),
        _ => None,
    }
}

fn map_set1(scancode: u8, extended: bool) -> Option<Key> {
    match (extended, scancode) {
        (false, 0x1C) => Some(Key::Enter),
        (false, 0x0E) => Some(Key::Backspace),
        (false, 0x01) => Some(Key::Esc),
        (false, 0x0F) => Some(Key::Tab),

        (true, 0x48) => Some(Key::Up),
        (true, 0x50) => Some(Key::Down),
        (true, 0x4B) => Some(Key::Left),
        (true, 0x4D) => Some(Key::Right),
        (true, 0x53) => Some(Key::Delete),
        
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
        
        (false, 0x39) => Some(Key::Char(' ')),
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
    {
        let mut last = LAST_SCANCODE.lock();
        *last = Some(scancode);
    }
    let mut processor = SCANCODE_PROCESSOR.lock();
    processor.handle(scancode, |key| {
        let mut buffer = KEY_BUFFER.lock();
        buffer.push_back(key);
    });
}

/// Internal state for scancode processing
#[derive(Clone, Copy)]
enum ScancodeState {
    Normal,
    Break,
    ExtendedBreak,
}

/// Processor encapsulating scancode state to avoid unsafe statics.
/// Uses a short-held mutex so it is safe to call from the interrupt
/// handler and from polling code without relying on `static mut`.
struct ScancodeProcessor {
    state: ScancodeState,
    extended: bool,
}

impl ScancodeProcessor {
    const fn new() -> Self {
        Self {
            state: ScancodeState::Normal,
            extended: false,
        }
    }

    fn handle<F: FnMut(Key)>(&mut self, scancode: u8, mut on_key: F) {
        match self.state {
            ScancodeState::Normal => {
                if scancode == SCANCODE_EXTENDED_PREFIX {
                    self.extended = true;
                    return;
                } else if scancode == SCANCODE_BREAK_PREFIX {
                    // Break code follows; track whether it was extended.
                    self.state = if self.extended {
                        ScancodeState::ExtendedBreak
                    } else {
                        ScancodeState::Break
                    };
                    return;
                } else {
                    // Regular make code
                    if let Some(key) = process_scancode(scancode, self.extended) {
                        on_key(key);
                    }
                    self.extended = false;
                }
            }
            ScancodeState::Break | ScancodeState::ExtendedBreak => {
                // Key release, ignore but reset state.
                self.state = ScancodeState::Normal;
                self.extended = false;
            }
        }
    }
}

/// Read a key from the keyboard buffer
///
/// Returns Some(Key) if a key is available, None otherwise.
/// This function does not block.
pub fn read_key() -> Option<Key> {
    let mut buffer = KEY_BUFFER.lock();
    buffer.pop_front()
}

/// Debug snapshot for on-screen overlay
pub fn debug_snapshot() -> (Option<u8>, usize, bool) {
    let last = *LAST_SCANCODE.lock();
    let len = KEY_BUFFER.lock().len();
    let pending = has_scancode();
    (last, len, pending)
}

/// Poll the keyboard for input
///
/// This function should be called periodically to check for keyboard input.
/// It reads scancodes and processes them into keys.
pub fn poll() {
    while let Some(scancode) = read_scancode() {
        // Log every scancode we receive for debugging
        use alloc::format;
        crate::serial::println(&format!("PS/2 scancode: 0x{:02X}", scancode));
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
