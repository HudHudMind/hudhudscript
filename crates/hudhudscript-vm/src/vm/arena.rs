//! Simple bump-allocator arena for short-lived Value allocations.
//!
//! Used during expression evaluation to avoid per-value heap allocations.
//! Call arena.reset() at the end of each statement to reclaim all memory.

/// A simple arena that allocates values in a contiguous buffer.
/// Values are NOT individually freed — call reset() to reclaim all at once.
pub struct ValueArena {
    #[allow(dead_code)]
    buffer: Vec<u8>,
    offset: usize,
    capacity: usize,
}

impl ValueArena {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            offset: 0,
            capacity,
        }
    }

    /// Reset the arena, reclaiming all allocated memory.
    /// This is O(1) — just resets the offset.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Current usage in bytes.
    pub fn used(&self) -> usize {
        self.offset
    }

    /// Remaining capacity in bytes.
    pub fn remaining(&self) -> usize {
        self.capacity - self.offset
    }
}

impl Default for ValueArena {
    fn default() -> Self {
        Self::new(64 * 1024) // 64KB default arena
    }
}
