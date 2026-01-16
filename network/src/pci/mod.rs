// PCI (Peripheral Component Interconnect) device discovery and configuration

use crate::error::NetError;

/// PCI vendor ID for Red Hat (virtio devices)
pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;

/// PCI device ID for virtio-net
pub const VIRTIO_NET_DEVICE_ID: u16 = 0x1000;

/// PCI configuration space address
/// 
/// On x86_64, PCI configuration space is accessed via I/O ports 0xCF8 (address) and 0xCFC (data)
#[cfg(target_arch = "x86_64")]
pub struct PciConfig {
    // Configuration space is accessed via I/O ports
}

/// PCI device information
#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    /// Bus number
    pub bus: u8,
    /// Device number
    pub device: u8,
    /// Function number
    pub function: u8,
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID
    pub device_id: u16,
    /// Class code
    pub class_code: u8,
    /// Subclass
    pub subclass: u8,
    /// Programming interface
    pub prog_if: u8,
    /// Base address registers
    pub bars: [u32; 6],
    /// Interrupt line
    pub interrupt_line: u8,
}

impl PciDevice {
    /// Read a 32-bit value from PCI configuration space
    #[cfg(target_arch = "x86_64")]
    pub fn read_config_dword(&self, offset: u8) -> u32 {
        let address = (1u32 << 31)
            | ((self.bus as u32) << 16)
            | ((self.device as u32) << 11)
            | ((self.function as u32) << 8)
            | (offset as u32 & 0xFC);
        
        unsafe {
            // Write address to 0xCF8
            x86_64::instructions::port::Port::new(0xCF8).write(address);
            // Read data from 0xCFC
            x86_64::instructions::port::Port::new(0xCFC).read()
        }
    }
    
    /// Write a 32-bit value to PCI configuration space
    #[cfg(target_arch = "x86_64")]
    pub fn write_config_dword(&self, offset: u8, value: u32) {
        let address = (1u32 << 31)
            | ((self.bus as u32) << 16)
            | ((self.device as u32) << 11)
            | ((self.function as u32) << 8)
            | (offset as u32 & 0xFC);
        
        unsafe {
            // Write address to 0xCF8
            x86_64::instructions::port::Port::new(0xCF8).write(address);
            // Write data to 0xCFC
            x86_64::instructions::port::Port::new(0xCFC).write(value);
        }
    }
    
    /// Read a 16-bit value from PCI configuration space
    #[cfg(target_arch = "x86_64")]
    pub fn read_config_word(&self, offset: u8) -> u16 {
        let dword = self.read_config_dword(offset);
        if offset & 2 == 0 {
            (dword & 0xFFFF) as u16
        } else {
            ((dword >> 16) & 0xFFFF) as u16
        }
    }
    
    /// Read a 8-bit value from PCI configuration space
    #[cfg(target_arch = "x86_64")]
    pub fn read_config_byte(&self, offset: u8) -> u8 {
        let dword = self.read_config_dword(offset);
        let shift = (offset & 3) * 8;
        ((dword >> shift) & 0xFF) as u8
    }
    
    /// Get the base address register at index `index`
    pub fn get_bar(&self, index: usize) -> u64 {
        if index >= 6 {
            return 0;
        }
        
        let bar_low = self.bars[index];
        
        // Check if it's a 64-bit BAR
        if (bar_low & 0x6) == 0x4 {
            // 64-bit BAR - read next BAR for high bits
            if index < 5 {
                let bar_high = self.bars[index + 1] as u64;
                ((bar_high as u64) << 32) | ((bar_low & 0xFFFFFFF0) as u64)
            } else {
                (bar_low & 0xFFFFFFF0) as u64
            }
        } else {
            // 32-bit BAR
            (bar_low & 0xFFFFFFF0) as u64
        }
    }
}

/// Scan PCI bus for devices
/// 
/// # Returns
/// A vector of all discovered PCI devices
#[cfg(target_arch = "x86_64")]
pub fn scan_pci_bus() -> alloc::vec::Vec<PciDevice> {
    let mut devices = alloc::vec::Vec::new();
    
    // Scan all buses (0-255)
    for bus in 0..=255 {
        // Scan all devices (0-31)
        for device in 0..=31 {
            // Scan all functions (0-7)
            for function in 0..=7 {
                // Read vendor ID (offset 0x00)
                let vendor_id = unsafe {
                    let address = (1u32 << 31)
                        | ((bus as u32) << 16)
                        | ((device as u32) << 11)
                        | ((function as u32) << 8);
                    x86_64::instructions::port::Port::new(0xCF8).write(address);
                    x86_64::instructions::port::Port::new(0xCFC).read() as u16
                };
                
                // If vendor ID is 0xFFFF, device doesn't exist
                if vendor_id == 0xFFFF {
                    continue;
                }
                
                // Read device ID (offset 0x02)
                let device_id = unsafe {
                    let address = (1u32 << 31)
                        | ((bus as u32) << 16)
                        | ((device as u32) << 11)
                        | ((function as u32) << 8)
                        | 0x00;
                    x86_64::instructions::port::Port::new(0xCF8).write(address);
                    let dword = x86_64::instructions::port::Port::new(0xCFC).read();
                    ((dword >> 16) & 0xFFFF) as u16
                };
                
                // Read class code (offset 0x08-0x0B)
                let class_reg = unsafe {
                    let address = (1u32 << 31)
                        | ((bus as u32) << 16)
                        | ((device as u32) << 11)
                        | ((function as u32) << 8)
                        | 0x08;
                    x86_64::instructions::port::Port::new(0xCF8).write(address);
                    x86_64::instructions::port::Port::new(0xCFC).read()
                };
                
                let class_code = ((class_reg >> 24) & 0xFF) as u8;
                let subclass = ((class_reg >> 16) & 0xFF) as u8;
                let prog_if = ((class_reg >> 8) & 0xFF) as u8;
                
                // Read BARs (offset 0x10-0x27)
                let mut bars = [0u32; 6];
                for i in 0..6 {
                    bars[i] = unsafe {
                        let address = (1u32 << 31)
                            | ((bus as u32) << 16)
                            | ((device as u32) << 11)
                            | ((function as u32) << 8)
                            | (0x10 + (i as u8 * 4));
                        x86_64::instructions::port::Port::new(0xCF8).write(address);
                        x86_64::instructions::port::Port::new(0xCFC).read()
                    };
                }
                
                // Read interrupt line (offset 0x3C)
                let interrupt_reg = unsafe {
                    let address = (1u32 << 31)
                        | ((bus as u32) << 16)
                        | ((device as u32) << 11)
                        | ((function as u32) << 8)
                        | 0x3C;
                    x86_64::instructions::port::Port::new(0xCF8).write(address);
                    x86_64::instructions::port::Port::new(0xCFC).read()
                };
                let interrupt_line = (interrupt_reg & 0xFF) as u8;
                
                let pci_device = PciDevice {
                    bus,
                    device,
                    function,
                    vendor_id,
                    device_id,
                    class_code,
                    subclass,
                    prog_if,
                    bars,
                    interrupt_line,
                };
                
                devices.push(pci_device);
            }
        }
    }
    
    devices
}

/// Find a PCI device by vendor and device ID
/// 
/// # Arguments
/// * `vendor_id` - The vendor ID to search for
/// * `device_id` - The device ID to search for
/// 
/// # Returns
/// * `Some(PciDevice)` if found
/// * `None` if not found
pub fn find_pci_device(vendor_id: u16, device_id: u16) -> Option<PciDevice> {
    #[cfg(target_arch = "x86_64")]
    {
        let devices = scan_pci_bus();
        devices.into_iter()
            .find(|d| d.vendor_id == vendor_id && d.device_id == device_id)
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        // ARM64 and other architectures would use different PCI access methods
        None
    }
}
