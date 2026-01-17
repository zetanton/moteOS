// virtio-net driver implementation
// Implements the virtio-net network device driver for QEMU/KVM VMs

use crate::drivers::NetworkDriver;
use crate::error::NetError;
use crate::pci::{find_pci_device, PciDevice, VIRTIO_NET_DEVICE_ID, VIRTIO_VENDOR_ID};
use core::ptr;
use spin::Mutex;
extern crate alloc;

/// Virtio device status register values
const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;
const VIRTIO_STATUS_DRIVER: u8 = 2;
const VIRTIO_STATUS_DRIVER_OK: u8 = 4;
const VIRTIO_STATUS_FEATURES_OK: u8 = 8;
const VIRTIO_STATUS_DEVICE_NEEDS_RESET: u8 = 64;
const VIRTIO_STATUS_FAILED: u8 = 128;

/// Virtio configuration space offsets
const VIRTIO_PCI_CONFIG_OFFSET: u16 = 0x100;

/// Virtio queue selector register offset
const VIRTIO_PCI_QUEUE_SEL: u16 = 0x0E;
/// Virtio queue size register offset
const VIRTIO_PCI_QUEUE_NUM: u16 = 0x10;
/// Virtio queue address register offset (64-bit, split into low/high)
const VIRTIO_PCI_QUEUE_PFN: u16 = 0x0C;
/// Virtio queue notify register offset
const VIRTIO_PCI_QUEUE_NOTIFY: u16 = 0x10;
/// Virtio device status register offset
const VIRTIO_PCI_STATUS: u16 = 0x12;
/// Virtio device features register offset
const VIRTIO_PCI_DEVICE_FEATURES: u16 = 0x00;
/// Virtio driver features register offset
const VIRTIO_PCI_DRIVER_FEATURES: u16 = 0x04;
/// Virtio MSI-X configuration register offset
const VIRTIO_PCI_MSIX_CONFIG: u16 = 0x11;

/// Virtio-net specific configuration offsets (from config space base)
const VIRTIO_NET_CONFIG_MAC: u16 = 0x00; // 6 bytes
const VIRTIO_NET_CONFIG_STATUS: u16 = 0x06; // 2 bytes
const VIRTIO_NET_CONFIG_MAX_QUEUE_PAIRS: u16 = 0x08; // 2 bytes

/// Virtio queue indices
const VIRTIO_NET_RX_QUEUE: u16 = 0;
const VIRTIO_NET_TX_QUEUE: u16 = 1;

/// Virtio-net feature bits
const VIRTIO_NET_F_MAC: u32 = 1 << 5;
const VIRTIO_NET_F_STATUS: u32 = 1 << 16;
const VIRTIO_NET_F_CTRL_VQ: u32 = 1 << 17;
const VIRTIO_NET_F_CTRL_RX: u32 = 1 << 18;
const VIRTIO_NET_F_CTRL_VLAN: u32 = 1 << 19;
const VIRTIO_NET_F_MRG_RXBUF: u32 = 1 << 15;
const VIRTIO_F_VERSION_1: u32 = 1 << 32;

/// Virtqueue descriptor flags
const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;
const VIRTQ_DESC_F_INDIRECT: u16 = 4;

/// Virtqueue available flags
const VIRTQ_AVAIL_F_NO_INTERRUPT: u16 = 1;

/// Virtqueue used flags
const VIRTQ_USED_F_NO_NOTIFY: u16 = 1;

/// Size of virtqueue (must be power of 2)
const VIRTQUEUE_SIZE: u16 = 256;

/// Virtqueue descriptor structure
#[repr(C, packed)]
struct VirtqDesc {
    addr: u64,  // Address (guest physical)
    len: u32,   // Length
    flags: u16, // Flags
    next: u16,  // Next descriptor index
}

/// Virtqueue available ring structure
#[repr(C, packed)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; VIRTQUEUE_SIZE as usize],
    used_event: u16, // Only if VIRTIO_F_EVENT_IDX
}

/// Virtqueue used ring entry
#[repr(C, packed)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

/// Virtqueue used ring structure
#[repr(C, packed)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; VIRTQUEUE_SIZE as usize],
    avail_event: u16, // Only if VIRTIO_F_EVENT_IDX
}

/// Virtqueue structure
struct Virtqueue {
    /// Descriptor table
    desc: *mut VirtqDesc,
    /// Available ring
    avail: *mut VirtqAvail,
    /// Used ring
    used: *mut VirtqUsed,
    /// Queue size
    size: u16,
    /// Next free descriptor index
    next_free: u16,
    /// Last used index we've processed
    last_used_idx: u16,
    /// Descriptor indices for pending packets
    pending: alloc::vec::Vec<u16>,
}

impl Virtqueue {
    /// Create a new virtqueue
    ///
    /// # Safety
    /// The memory region must be properly allocated and aligned
    ///
    /// # Arguments
    /// * `memory_base` - Base address of pre-allocated memory (must be page-aligned)
    unsafe fn new(size: u16, memory_base: *mut u8) -> Result<Self, NetError> {
        // Calculate sizes
        let desc_size = core::mem::size_of::<VirtqDesc>() * size as usize;
        let avail_size = core::mem::size_of::<VirtqAvail>();
        let used_size = core::mem::size_of::<VirtqUsed>();

        // Calculate offsets
        let desc = memory_base as *mut VirtqDesc;
        let avail = memory_base.add(desc_size) as *mut VirtqAvail;
        let used = memory_base.add(desc_size + avail_size) as *mut VirtqUsed;

        // Zero out memory
        ptr::write_bytes(memory_base, 0, desc_size + avail_size + used_size);

        // Initialize available ring
        (*avail).flags = 0;
        (*avail).idx = 0;

        // Initialize used ring
        (*used).flags = 0;
        (*used).idx = 0;

        Ok(Virtqueue {
            desc,
            avail,
            used,
            size,
            next_free: 0,
            last_used_idx: 0,
            pending: alloc::vec::Vec::new(),
        })
    }

    /// Add a buffer to the queue
    ///
    /// # Arguments
    /// * `addr` - Physical address of the buffer
    /// * `len` - Length of the buffer
    /// * `flags` - Descriptor flags
    ///
    /// # Returns
    /// The descriptor index on success
    ///
    /// # Errors
    /// Returns `NetError::QueueError` if the queue is full
    unsafe fn add_buffer(&mut self, addr: u64, len: u32, flags: u16) -> Result<u16, NetError> {
        if self.next_free >= self.size {
            return Err(NetError::QueueError(format!(
                "Queue full: next_free={}, size={}",
                self.next_free, self.size
            )));
        }

        if addr == 0 {
            return Err(NetError::QueueError(
                "Invalid buffer address (null)".to_string(),
            ));
        }

        if len == 0 {
            return Err(NetError::QueueError(
                "Invalid buffer length (zero)".to_string(),
            ));
        }

        let idx = self.next_free;
        let desc = &mut *self.desc.add(idx as usize);

        desc.addr = addr;
        desc.len = len;
        desc.flags = flags;
        desc.next = 0;

        self.next_free = (self.next_free + 1) % self.size;
        Ok(idx)
    }

    /// Notify the device about new buffers
    unsafe fn notify(&mut self, queue_index: u16, io_base: usize) {
        // Add descriptor to available ring
        let avail = &mut *self.avail;
        let ring_idx = (avail.idx % self.size) as usize;
        avail.ring[ring_idx] = self.next_free.wrapping_sub(1);
        avail.idx = avail.idx.wrapping_add(1);

        // Memory barrier to ensure writes are visible
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

        // Notify device
        let notify_addr = io_base + (VIRTIO_PCI_QUEUE_NOTIFY as usize);
        ptr::write_volatile(notify_addr as *mut u16, queue_index);
    }

    /// Check for used buffers
    unsafe fn get_used(&mut self) -> Option<(u32, u32)> {
        let used = &*self.used;

        // Memory barrier
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

        if used.idx == self.last_used_idx {
            return None;
        }

        let used_idx = (self.last_used_idx % self.size) as usize;
        let elem = &used.ring[used_idx];
        self.last_used_idx = self.last_used_idx.wrapping_add(1);

        Some((elem.id, elem.len))
    }
}

/// RX buffer information
struct RxBuffer {
    /// Physical address
    phys: u64,
    /// Virtual address
    ptr: *mut u8,
    /// Buffer size
    size: usize,
    /// Descriptor index in the queue
    desc_idx: u16,
}

/// TX buffer information
struct TxBuffer {
    /// Physical address
    phys: u64,
    /// Virtual address
    ptr: *mut u8,
    /// Buffer size
    size: usize,
    /// Descriptor index in the queue
    desc_idx: u16,
}

/// virtio-net driver
pub struct VirtioNet {
    /// PCI device information
    pci_device: PciDevice,
    /// I/O base address (BAR0)
    io_base: usize,
    /// Configuration space base address
    config_base: usize,
    /// MAC address
    mac_address: [u8; 6],
    /// Receive queue
    rx_queue: Option<Virtqueue>,
    /// Transmit queue
    tx_queue: Option<Virtqueue>,
    /// RX buffer pool with descriptor mapping
    rx_buffers: alloc::vec::Vec<RxBuffer>,
    /// TX buffer pool with descriptor mapping
    tx_buffers: alloc::vec::Vec<TxBuffer>,
    /// Initialized flag
    initialized: bool,
}

impl VirtioNet {
    /// Create a new virtio-net driver instance
    ///
    /// This will scan for a virtio-net PCI device and initialize it.
    pub fn new() -> Result<Self, NetError> {
        // Find virtio-net PCI device
        let pci_device = find_pci_device(VIRTIO_VENDOR_ID, VIRTIO_NET_DEVICE_ID)
            .ok_or(NetError::DeviceNotFound)?;

        // Get BAR0 (I/O base)
        let io_base = pci_device.get_bar(0) as usize;
        if io_base == 0 {
            return Err(NetError::PciError("BAR0 is invalid".to_string()));
        }

        // Get configuration space base (BAR0 + offset)
        let config_base = io_base + VIRTIO_PCI_CONFIG_OFFSET as usize;

        Ok(VirtioNet {
            pci_device,
            io_base,
            config_base,
            mac_address: [0; 6],
            rx_queue: None,
            tx_queue: None,
            rx_buffers: alloc::vec::Vec::new(),
            tx_buffers: alloc::vec::Vec::new(),
            initialized: false,
        })
    }

    /// Initialize the virtio-net device
    pub fn init(&mut self) -> Result<(), NetError> {
        // Reset device
        self.write_status(0);

        // Acknowledge device
        self.write_status(VIRTIO_STATUS_ACKNOWLEDGE);

        // Set driver status
        self.write_status(VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER);

        // Read device features
        let device_features = self.read_device_features();

        // Negotiate features (we support basic features)
        let driver_features =
            device_features & (VIRTIO_NET_F_MAC | VIRTIO_NET_F_STATUS | VIRTIO_F_VERSION_1);
        self.write_driver_features(driver_features);

        // Set features OK
        self.write_status(
            VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER | VIRTIO_STATUS_FEATURES_OK,
        );

        // Verify features were accepted
        if (self.read_status() & VIRTIO_STATUS_FEATURES_OK) == 0 {
            return Err(NetError::VirtioError(
                "Feature negotiation failed".to_string(),
            ));
        }

        // Read MAC address from configuration space
        unsafe {
            for i in 0..6 {
                self.mac_address[i] = ptr::read_volatile(
                    (self.config_base + VIRTIO_NET_CONFIG_MAC as usize + i) as *const u8,
                );
            }
        }

        // Initialize queues
        self.init_queues()?;

        // Set driver OK
        self.write_status(
            VIRTIO_STATUS_ACKNOWLEDGE
                | VIRTIO_STATUS_DRIVER
                | VIRTIO_STATUS_FEATURES_OK
                | VIRTIO_STATUS_DRIVER_OK,
        );

        self.initialized = true;
        Ok(())
    }

    /// Initialize virtqueues
    fn init_queues(&mut self) -> Result<(), NetError> {
        // Allocate memory for queues (must be page-aligned)
        // Calculate required memory per queue
        let desc_size = core::mem::size_of::<VirtqDesc>() * VIRTQUEUE_SIZE as usize;
        let avail_size = core::mem::size_of::<VirtqAvail>();
        let used_size = core::mem::size_of::<VirtqUsed>();
        let queue_size = desc_size + avail_size + used_size;

        // Allocate RX queue memory
        let rx_memory = unsafe {
            let layout = core::alloc::Layout::from_size_align(queue_size, 4096)
                .map_err(|_| NetError::QueueError("Invalid RX queue layout".to_string()))?;
            let ptr = alloc::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                return Err(NetError::QueueError(
                    "Failed to allocate RX queue memory".to_string(),
                ));
            }
            ptr
        };

        // Allocate TX queue memory
        let tx_memory = unsafe {
            let layout = core::alloc::Layout::from_size_align(queue_size, 4096)
                .map_err(|_| NetError::QueueError("Invalid TX queue layout".to_string()))?;
            let ptr = alloc::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                return Err(NetError::QueueError(
                    "Failed to allocate TX queue memory".to_string(),
                ));
            }
            ptr
        };

        // Initialize RX queue
        unsafe {
            let rx_queue = Virtqueue::new(VIRTQUEUE_SIZE, rx_memory)?;
            self.setup_queue(VIRTIO_NET_RX_QUEUE, &rx_queue)?;
            self.rx_queue = Some(rx_queue);

            // Initialize TX queue
            let tx_queue = Virtqueue::new(VIRTQUEUE_SIZE, tx_memory)?;
            self.setup_queue(VIRTIO_NET_TX_QUEUE, &tx_queue)?;
            self.tx_queue = Some(tx_queue);
        }

        // Allocate RX buffers
        self.allocate_rx_buffers()?;

        Ok(())
    }

    /// Setup a virtqueue
    ///
    /// # Errors
    /// Returns `NetError::QueueError` if queue setup fails
    unsafe fn setup_queue(&mut self, queue_index: u16, queue: &Virtqueue) -> Result<(), NetError> {
        if queue.desc.is_null() {
            return Err(NetError::QueueError(
                "Queue descriptor table is null".to_string(),
            ));
        }

        // Select queue
        self.write_u16(VIRTIO_PCI_QUEUE_SEL, queue_index);

        // Set queue size
        if queue.size == 0 {
            return Err(NetError::QueueError("Queue size is zero".to_string()));
        }
        self.write_u16(VIRTIO_PCI_QUEUE_NUM, queue.size);

        // Get physical address of queue
        let queue_phys = self.virt_to_phys(queue.desc as usize);
        if queue_phys == 0 {
            return Err(NetError::QueueError(
                "Failed to get physical address of queue".to_string(),
            ));
        }

        // Verify alignment (must be page-aligned)
        if (queue_phys & 0xFFF) != 0 {
            return Err(NetError::QueueError("Queue not page-aligned".to_string()));
        }

        // Set queue address (PFN = physical frame number)
        let pfn = queue_phys >> 12; // Page frame number
        if pfn == 0 {
            return Err(NetError::QueueError(
                "Invalid page frame number".to_string(),
            ));
        }
        self.write_u32(VIRTIO_PCI_QUEUE_PFN, pfn as u32);

        Ok(())
    }

    /// Allocate RX buffers
    fn allocate_rx_buffers(&mut self) -> Result<(), NetError> {
        // Allocate buffers for receiving packets
        // Each buffer is 1526 bytes (max Ethernet frame size)
        const BUFFER_SIZE: usize = 1526;
        const NUM_BUFFERS: usize = 32;

        if let Some(ref mut rx_queue) = self.rx_queue {
            for _ in 0..NUM_BUFFERS {
                let layout = core::alloc::Layout::from_size_align(BUFFER_SIZE, 16)
                    .map_err(|_| NetError::QueueError("Invalid buffer layout".to_string()))?;

                unsafe {
                    let ptr = alloc::alloc::alloc_zeroed(layout);
                    if ptr.is_null() {
                        return Err(NetError::QueueError(
                            "Failed to allocate RX buffer".to_string(),
                        ));
                    }

                    let phys = self.virt_to_phys(ptr as usize);

                    // Add buffer to RX queue and get descriptor index
                    let desc_idx = rx_queue
                        .add_buffer(phys, BUFFER_SIZE as u32, VIRTQ_DESC_F_WRITE)
                        .map_err(|e| {
                            // Clean up on error
                            alloc::alloc::dealloc(ptr, layout);
                            e
                        })?;

                    // Store buffer with descriptor mapping
                    self.rx_buffers.push(RxBuffer {
                        phys,
                        ptr,
                        size: BUFFER_SIZE,
                        desc_idx,
                    });

                    rx_queue.pending.push(desc_idx);
                }
            }

            // Notify device about RX buffers
            unsafe {
                rx_queue.notify(VIRTIO_NET_RX_QUEUE, self.io_base);
            }
        } else {
            return Err(NetError::QueueError("RX queue not initialized".to_string()));
        }

        Ok(())
    }

    /// Read device status
    fn read_status(&self) -> u8 {
        unsafe { ptr::read_volatile((self.io_base + VIRTIO_PCI_STATUS as usize) as *const u8) }
    }

    /// Write device status
    fn write_status(&mut self, status: u8) {
        unsafe {
            ptr::write_volatile(
                (self.io_base + VIRTIO_PCI_STATUS as usize) as *mut u8,
                status,
            );
        }
    }

    /// Read device features
    fn read_device_features(&self) -> u64 {
        unsafe {
            let low = ptr::read_volatile(
                (self.io_base + VIRTIO_PCI_DEVICE_FEATURES as usize) as *const u32,
            ) as u64;
            let high = ptr::read_volatile(
                (self.io_base + VIRTIO_PCI_DEVICE_FEATURES as usize + 4) as *const u32,
            ) as u64;
            low | (high << 32)
        }
    }

    /// Write driver features
    fn write_driver_features(&mut self, features: u64) {
        unsafe {
            ptr::write_volatile(
                (self.io_base + VIRTIO_PCI_DRIVER_FEATURES as usize) as *mut u32,
                features as u32,
            );
            ptr::write_volatile(
                (self.io_base + VIRTIO_PCI_DRIVER_FEATURES as usize + 4) as *mut u32,
                (features >> 32) as u32,
            );
        }
    }

    /// Write a 16-bit value to I/O space
    fn write_u16(&mut self, offset: u16, value: u16) {
        unsafe {
            ptr::write_volatile((self.io_base + offset as usize) as *mut u16, value);
        }
    }

    /// Write a 32-bit value to I/O space
    fn write_u32(&mut self, offset: u16, value: u32) {
        unsafe {
            ptr::write_volatile((self.io_base + offset as usize) as *mut u32, value);
        }
    }

    /// Convert virtual address to physical address
    ///
    /// Note: This is a simplified version. In a real implementation,
    /// you would need to use proper page table translation.
    fn virt_to_phys(&self, virt: usize) -> u64 {
        // For now, assume identity mapping
        virt as u64
    }

    /// Handle interrupt from the virtio device
    ///
    /// This should be called from the interrupt handler when a virtio-net interrupt occurs.
    /// It processes received packets and handles transmission completion.
    ///
    /// # Errors
    /// Returns `NetError` if interrupt handling fails
    pub fn handle_interrupt(&mut self) -> Result<(), NetError> {
        if !self.initialized {
            return Err(NetError::DeviceNotInitialized);
        }

        // Check for received packets
        // Note: The actual packet data retrieval is done in receive() method
        // This just acknowledges that packets are available
        if let Some(ref mut rx_queue) = self.rx_queue {
            unsafe {
                // Check if there are any used buffers (packets received)
                // The actual processing happens in receive() method
                let _ = rx_queue.get_used();
            }
        }

        // Check for transmitted packets and free buffers
        if let Some(ref mut tx_queue) = self.tx_queue {
            unsafe {
                while let Some((used_id, _len)) = tx_queue.get_used() {
                    let desc_id = used_id as u16;

                    // Remove from pending list
                    if let Some(pending_pos) = tx_queue.pending.iter().position(|&x| x == desc_id) {
                        tx_queue.pending.remove(pending_pos);
                    }

                    // Find and free the buffer
                    if let Some(buf_pos) = self
                        .tx_buffers
                        .iter()
                        .position(|buf| buf.desc_idx == desc_id)
                    {
                        let buffer = self.tx_buffers.remove(buf_pos);

                        // Deallocate the buffer
                        let layout = core::alloc::Layout::from_size_align(buffer.size, 16)
                            .map_err(|_| {
                                NetError::QueueError(
                                    "Invalid TX buffer layout for deallocation".to_string(),
                                )
                            })?;
                        alloc::alloc::dealloc(buffer.ptr, layout);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the interrupt line for this device
    pub fn interrupt_line(&self) -> u8 {
        self.pci_device.interrupt_line
    }
}

impl NetworkDriver for VirtioNet {
    fn send(&mut self, packet: &[u8]) -> Result<(), NetError> {
        if !self.initialized {
            return Err(NetError::DeviceNotInitialized);
        }

        if packet.len() > 1526 {
            return Err(NetError::InvalidPacket("Packet too large".to_string()));
        }

        if packet.is_empty() {
            return Err(NetError::InvalidPacket("Packet is empty".to_string()));
        }

        // Allocate buffer for TX
        let layout = core::alloc::Layout::from_size_align(packet.len(), 16)
            .map_err(|_| NetError::QueueError("Invalid TX buffer layout".to_string()))?;

        unsafe {
            let tx_buf = alloc::alloc::alloc(layout);
            if tx_buf.is_null() {
                return Err(NetError::QueueError(
                    "Failed to allocate TX buffer".to_string(),
                ));
            }

            // Copy packet to buffer
            ptr::copy_nonoverlapping(packet.as_ptr(), tx_buf, packet.len());

            let phys = self.virt_to_phys(tx_buf as usize);

            // Add to TX queue
            if let Some(ref mut tx_queue) = self.tx_queue {
                let desc_idx = tx_queue
                    .add_buffer(phys, packet.len() as u32, 0)
                    .map_err(|e| {
                        // Clean up on error
                        alloc::alloc::dealloc(tx_buf, layout);
                        e
                    })?;

                tx_queue.pending.push(desc_idx);

                // Store buffer info with descriptor mapping for later cleanup
                self.tx_buffers.push(TxBuffer {
                    phys,
                    ptr: tx_buf,
                    size: packet.len(),
                    desc_idx,
                });

                // Notify device
                tx_queue.notify(VIRTIO_NET_TX_QUEUE, self.io_base);
            } else {
                // Clean up on error
                alloc::alloc::dealloc(tx_buf, layout);
                return Err(NetError::QueueError("TX queue not initialized".to_string()));
            }
        }

        Ok(())
    }

    fn receive(&mut self) -> Result<Option<alloc::vec::Vec<u8>>, NetError> {
        if !self.initialized {
            return Err(NetError::DeviceNotInitialized);
        }

        // Check for used buffers in RX queue
        if let Some(ref mut rx_queue) = self.rx_queue {
            unsafe {
                if let Some((used_id, len)) = rx_queue.get_used() {
                    let desc_id = used_id as u16;

                    // Find the buffer that corresponds to this descriptor ID
                    let buffer_idx = self
                        .rx_buffers
                        .iter()
                        .position(|buf| buf.desc_idx == desc_id)
                        .ok_or_else(|| {
                            NetError::QueueError("Descriptor ID not found in buffers".to_string())
                        })?;

                    let buffer = &self.rx_buffers[buffer_idx];

                    // Validate length
                    if len as usize > buffer.size {
                        return Err(NetError::InvalidPacket(
                            "Received packet exceeds buffer size".to_string(),
                        ));
                    }

                    // Create packet vector safely
                    // Allocate uninitialized memory, then immediately copy valid data
                    let mut packet = alloc::vec::Vec::with_capacity(len as usize);
                    unsafe {
                        // Safety: set_len() is safe here because:
                        // 1. We set the length to exactly the amount we'll copy
                        // 2. We immediately copy valid data from buffer.ptr into the vector
                        // 3. The buffer.ptr is guaranteed to be valid (allocated in allocate_rx_buffers)
                        // 4. len is validated to be <= buffer.size above
                        packet.set_len(len as usize);
                        ptr::copy_nonoverlapping(buffer.ptr, packet.as_mut_ptr(), len as usize);
                    }

                    // Re-add buffer to queue with new descriptor index
                    let new_desc_idx = rx_queue
                        .add_buffer(buffer.phys, buffer.size as u32, VIRTQ_DESC_F_WRITE)
                        .map_err(|e| {
                            NetError::QueueError(format!("Failed to re-add RX buffer: {:?}", e))
                        })?;

                    // Update buffer descriptor mapping
                    self.rx_buffers[buffer_idx].desc_idx = new_desc_idx;

                    // Remove old descriptor from pending and add new one
                    if let Some(pending_idx) = rx_queue.pending.iter().position(|&x| x == desc_id) {
                        rx_queue.pending.remove(pending_idx);
                    }
                    rx_queue.pending.push(new_desc_idx);

                    // Notify device about the new buffer
                    rx_queue.notify(VIRTIO_NET_RX_QUEUE, self.io_base);

                    return Ok(Some(packet));
                }
            }
        }

        Ok(None)
    }

    fn mac_address(&self) -> [u8; 6] {
        self.mac_address
    }

    fn is_link_up(&self) -> bool {
        if !self.initialized {
            return false;
        }

        // Read link status from configuration space
        unsafe {
            let status = ptr::read_volatile(
                (self.config_base + VIRTIO_NET_CONFIG_STATUS as usize) as *const u16,
            );
            (status & 1) != 0 // Bit 0 indicates link up
        }
    }

    fn poll(&mut self) -> Result<(), NetError> {
        // Poll for received packets and handle interrupts
        // This is called regularly by the network stack

        // Check for used TX buffers (packets that were sent)
        if let Some(ref mut tx_queue) = self.tx_queue {
            unsafe {
                while let Some((used_id, _len)) = tx_queue.get_used() {
                    let desc_id = used_id as u16;

                    // Remove from pending list
                    if let Some(pending_pos) = tx_queue.pending.iter().position(|&x| x == desc_id) {
                        tx_queue.pending.remove(pending_pos);
                    }

                    // Find and free the buffer using descriptor ID mapping
                    if let Some(buf_pos) = self
                        .tx_buffers
                        .iter()
                        .position(|buf| buf.desc_idx == desc_id)
                    {
                        let buffer = self.tx_buffers.remove(buf_pos);

                        // Deallocate the buffer
                        let layout = core::alloc::Layout::from_size_align(buffer.size, 16)
                            .map_err(|_| {
                                NetError::QueueError(
                                    "Invalid TX buffer layout for deallocation".to_string(),
                                )
                            })?;
                        alloc::alloc::dealloc(buffer.ptr, layout);
                    } else {
                        // Descriptor ID not found in buffers - this is an error condition
                        // Log it but don't fail the poll operation
                        // In a production system, this would be logged
                    }
                }
            }
        }

        Ok(())
    }
}

// Global virtio-net instance (protected by mutex)
static VIRTIO_NET: Mutex<Option<VirtioNet>> = Mutex::new(None);

/// Initialize the virtio-net driver
///
/// This should be called once during system initialization.
pub fn init_virtio_net() -> Result<(), NetError> {
    let mut driver = VirtioNet::new()?;
    driver.init()?;

    let mut global = VIRTIO_NET.lock();
    *global = Some(driver);
    Ok(())
}

/// Get the global virtio-net driver instance
pub fn get_virtio_net() -> Option<spin::MutexGuard<'static, Option<VirtioNet>>> {
    Some(VIRTIO_NET.lock())
}
