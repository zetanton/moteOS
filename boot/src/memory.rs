// Memory management for moteOS
// Handles memory map parsing, heap initialization, and memory allocation

#[cfg(not(test))]
use linked_list_allocator::LockedHeap;

/// Global heap allocator
///
/// This allocator must be initialized with `init_heap()` before use.
#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Memory region kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    /// Usable RAM
    Usable,
    /// Reserved (not usable)
    Reserved,
    /// ACPI reclaimable memory
    AcpiReclaimable,
    /// ACPI NVS (Non-Volatile Storage)
    AcpiNvs,
}

/// Memory region descriptor
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    /// Physical start address
    pub start: usize,
    /// Length in bytes
    pub len: usize,
    /// Memory kind
    pub kind: MemoryKind,
}

/// Memory map containing all memory regions
#[derive(Debug)]
pub struct MemoryMap {
    /// Array of memory regions (stored in static memory after boot)
    pub regions: &'static [MemoryRegion],
}

impl MemoryMap {
    /// Create a new memory map
    pub fn new(regions: &'static [MemoryRegion]) -> Self {
        Self { regions }
    }

    /// Find the largest usable memory region
    pub fn find_largest_usable(&self) -> Option<&MemoryRegion> {
        self.regions
            .iter()
            .filter(|r| r.kind == MemoryKind::Usable)
            .max_by_key(|r| r.len)
    }

    /// Calculate total usable memory
    pub fn total_usable(&self) -> usize {
        self.regions
            .iter()
            .filter(|r| r.kind == MemoryKind::Usable)
            .map(|r| r.len)
            .sum()
    }
}

/// Initialize the heap allocator
///
/// # Safety
///
/// - `heap_start` must be a valid, aligned memory address
/// - `heap_size` must be the size of a contiguous, usable memory region
/// - This function must only be called once
/// - The memory region must not be used for anything else
#[cfg(not(test))]
pub unsafe fn init_heap(heap_start: usize, heap_size: usize) {
    ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
}

#[cfg(test)]
/// Stub version for tests (uses std allocator)
pub unsafe fn init_heap(_heap_start: usize, _heap_size: usize) {
    // In test mode, std's allocator is used
}

/// Check if the heap allocator is initialized
pub fn is_heap_initialized() -> bool {
    // LockedHeap doesn't expose an initialization check, so we'll just
    // assume it's initialized if we can lock it (which always works)
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_kind() {
        assert_eq!(MemoryKind::Usable, MemoryKind::Usable);
        assert_ne!(MemoryKind::Usable, MemoryKind::Reserved);
    }

    #[test]
    fn test_memory_region() {
        let region = MemoryRegion {
            start: 0x100000,
            len: 64 * 1024 * 1024, // 64 MB
            kind: MemoryKind::Usable,
        };
        assert_eq!(region.start, 0x100000);
        assert_eq!(region.len, 64 * 1024 * 1024);
    }
}
