//! Minimal serial output (COM1) for headless testing

use core::fmt::{self, Write};

#[cfg(target_arch = "x86_64")]
pub struct SerialPort {
    base: u16,
}

#[cfg(target_arch = "x86_64")]
impl SerialPort {
    pub const fn new(base: u16) -> Self {
        Self { base }
    }

    pub fn init(&self) {
        unsafe {
            outb(self.base + 1, 0x00); // Disable interrupts
            outb(self.base + 3, 0x80); // Enable DLAB
            outb(self.base + 0, 0x03); // Divisor low (38400 baud)
            outb(self.base + 1, 0x00); // Divisor high
            outb(self.base + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(self.base + 2, 0xC7); // Enable FIFO
            outb(self.base + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    fn transmit_empty(&self) -> bool {
        unsafe { inb(self.base + 5) & 0x20 != 0 }
    }

    pub fn write_byte(&self, byte: u8) {
        while !self.transmit_empty() {}
        unsafe {
            outb(self.base, byte);
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}

#[cfg(target_arch = "x86_64")]
unsafe fn inb(port: u16) -> u8 {
    let mut val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port);
    val
}

#[cfg(target_arch = "aarch64")]
pub struct SerialPort {
    base: usize,
}

#[cfg(target_arch = "aarch64")]
impl SerialPort {
    // QEMU virt PL011 base address
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    pub fn init(&self) {
        // UEFI/QEMU usually initializes PL011; nothing required for basic TX.
    }

    fn tx_ready(&self) -> bool {
        // UARTFR register at offset 0x18, TXFF bit 5
        unsafe {
            let fr = core::ptr::read_volatile((self.base + 0x18) as *const u32);
            (fr & (1 << 5)) == 0
        }
    }

    pub fn write_byte(&self, byte: u8) {
        while !self.tx_ready() {}
        unsafe {
            core::ptr::write_volatile(self.base as *mut u32, byte as u32);
        }
    }
}

#[cfg(target_arch = "aarch64")]
impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub fn println(message: &str) {
    #[cfg(target_arch = "x86_64")]
    {
        let mut port = SerialPort::new(0x3F8);
        port.init();
        let _ = writeln!(port, "{}", message);
    }

    #[cfg(target_arch = "aarch64")]
    {
        let mut port = SerialPort::new(0x0900_0000);
        port.init();
        let _ = writeln!(port, "{}", message);
    }
}
