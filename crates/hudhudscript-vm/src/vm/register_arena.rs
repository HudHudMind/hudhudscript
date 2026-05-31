//! Register arena — eliminates per-call register save/restore by allocating
//! a contiguous slice of a growable Vec for each call frame.

use hudhudscript_bytecode::Value16;
use std::ops::{Index, IndexMut, Range};

/// Arena-based register file.
///
/// Instead of a fixed 256-entry array that is copied on every call,
/// each function frame gets a dynamic slice `arena[base..base+frame_size]`.
/// Advancing/retreating the base pointer replaces memcpy save/restore.
#[derive(Clone)]
pub struct RegisterArena {
    arena: Vec<Value16>,
    base: usize,
    /// Cached pointer to arena[base] — eliminates base+add+bounds_check
    /// per register access. Invalidated only by arena.resize in advance().
    base_ptr: *mut Value16,
}

impl RegisterArena {
    /// Initial capacity: 256 entries (one traditional frame).
    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    /// Create with a custom initial capacity in entries.
    pub fn with_capacity(capacity: usize) -> Self {
        let n = capacity.max(256);
        let mut arena = vec![Value16::null(); n];
        let base_ptr = arena.as_mut_ptr();
        Self {
            arena,
            base: 0,
            base_ptr,
        }
    }

    /// Move the base forward by `frame_size` for a new call frame.
    /// Grows the underlying Vec if necessary.
    /// Each frame always reserves 256 slots because instructions may access
    /// register 255 (special-purpose return/bridge slot) even when
    /// frame_size < 256.
    #[inline(always)]
    pub fn advance(&mut self, frame_size: usize) {
        self.base += frame_size;
        let needed = self.base + 256;
        if needed > self.arena.len() {
            self.arena.resize(needed, Value16::null());
        }
        self.base_ptr = unsafe { self.arena.as_mut_ptr().add(self.base) };
    }

    /// Move the base backward by `frame_size` on return.
    #[inline(always)]
    pub fn retreat(&mut self, frame_size: usize) {
        self.base -= frame_size;
        self.base_ptr = unsafe { self.arena.as_mut_ptr().add(self.base) };
    }

    /// Zero the current frame's registers.
    pub fn zero_frame(&mut self, frame_size: usize) {
        for i in 0..frame_size {
            unsafe { *self.base_ptr.add(i) = Value16::null(); }
        }
    }

    /// Direct mutable access to the whole arena (for bulk ops).
    pub fn arena_mut(&mut self) -> &mut [Value16] {
        // Cache invalidation: caller must not resize the Vec without
        // re-syncing base_ptr. Currently no call sites resize.
        &mut self.arena
    }

    pub fn base(&self) -> usize {
        self.base
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }
}

impl Default for RegisterArena {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for RegisterArena {
    type Output = Value16;
    #[inline(always)]
    fn index(&self, idx: usize) -> &Value16 {
        debug_assert!(self.base + idx < self.arena.len(), "RegisterArena index OOB: base={} idx={} len={}", self.base, idx, self.arena.len());
        unsafe { &*self.base_ptr.add(idx) }
    }
}

impl IndexMut<usize> for RegisterArena {
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut Value16 {
        debug_assert!(self.base + idx < self.arena.len(), "RegisterArena index_mut OOB: base={} idx={} len={}", self.base, idx, self.arena.len());
        unsafe { &mut *self.base_ptr.add(idx) }
    }
}

impl Index<Range<usize>> for RegisterArena {
    type Output = [Value16];
    #[inline(always)]
    fn index(&self, range: Range<usize>) -> &[Value16] {
        unsafe {
            let start = self.base_ptr.add(range.start);
            let len = range.end - range.start;
            std::slice::from_raw_parts(start, len)
        }
    }
}

impl IndexMut<Range<usize>> for RegisterArena {
    #[inline(always)]
    fn index_mut(&mut self, range: Range<usize>) -> &mut [Value16] {
        unsafe {
            let start = self.base_ptr.add(range.start);
            let len = range.end - range.start;
            std::slice::from_raw_parts_mut(start, len)
        }
    }
}
