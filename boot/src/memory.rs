// Memory management for moteOS
// Handles memory map parsing and memory allocation
// Note: Heap allocator is now provided by the shared crate

// Re-export heap initialization functions from shared crate
pub use shared::{init_heap, is_heap_initialized};

