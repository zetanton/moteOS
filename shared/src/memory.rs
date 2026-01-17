// Memory map types shared across moteOS crates

/// Memory region kind
#[repr(u8)]
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
#[repr(C)]
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
#[repr(C)]
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
