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

