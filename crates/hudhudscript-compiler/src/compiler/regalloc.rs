//! Linear-scan register allocator for register-based VM code generation.
//!
//! Uses a per-thread counter to assign non-overlapping register ranges
//! to each allocator instance. This prevents register aliasing when
//! nested expressions create their own allocators.

use std::cell::Cell;
use crate::error::{compile_codes, CompileResult};

/// Registers per allocator instance.
const REGS_PER_ALLOC: u8 = 32;
/// Maximum valid base before u8 wrap-around would cause silent aliasing.
const MAX_BASE: u8 = 224;

thread_local! {
    static NEXT_BASE: Cell<u8> = const { Cell::new(0) };
}

pub struct RegAlloc {
    /// Base offset for this allocator's register range.
    base: u8,
    /// Next available local register index (0..REGS_PER_ALLOC).
    next: u8,
    /// Freed local indices available for reuse.
    free: Vec<u8>,
    /// Active live ranges: (end_ip, local_register_index).
    active: Vec<(usize, u8)>,
}

impl RegAlloc {
    pub fn new() -> CompileResult<Self> {
        let base = NEXT_BASE.with(|c| {
            let current = c.get();
            if current >= MAX_BASE {
                return Err(compile_codes::generic(
                    format!("RegAlloc: out of register zones (base={})", current)
                ));
            }
            c.set(current + REGS_PER_ALLOC);
            Ok(current)
        })?;
        Ok(Self {
            base,
            next: 0,
            free: Vec::with_capacity(8),
            active: Vec::with_capacity(8),
        })
    }

    /// Local variable–aware constructor.
    /// Temp registers start at `floor` (= next_local_reg), never below.
    pub fn new_with_base(floor: u8) -> CompileResult<Self> {
        let base = NEXT_BASE.with(|c| {
            let current = c.get();
            let effective = current.max(floor);
            if effective >= MAX_BASE {
                return Err(compile_codes::generic(
                    format!("RegAlloc: out of register zones (base={}, floor={})", current, floor)
                ));
            }
            c.set(effective + REGS_PER_ALLOC);
            Ok(effective)
        })?;
        Ok(Self {
            base,
            next: 0,
            free: Vec::with_capacity(8),
            active: Vec::with_capacity(8),
        })
    }

    fn expire(&mut self, current_ip: usize) {
        let mut keep = Vec::with_capacity(self.active.len());
        for (end_ip, reg) in self.active.drain(..) {
            if end_ip <= current_ip {
                self.free.push(reg);
            } else {
                keep.push((end_ip, reg));
            }
        }
        self.active = keep;
    }

    /// Allocate a register. Returns the GLOBAL register index.
    pub fn alloc(&mut self, current_ip: usize, last_use_ip: usize) -> CompileResult<u8> {
        self.expire(current_ip);

        let local = if let Some(reg) = self.free.pop() {
            reg
        } else {
            let reg = self.next;
            self.next += 1;
            if reg >= REGS_PER_ALLOC {
                return Err(compile_codes::generic(
                    format!("RegAlloc: out of registers (base={}, next={})", self.base, reg)
                ));
            }
            reg
        };

        if last_use_ip > current_ip {
            self.active.push((last_use_ip, local));
            self.active.sort_by_key(|(end, _)| *end);
        }

        Ok(self.base + local)
    }

    pub fn free_now(&mut self, reg: u8) {
        let local = reg - self.base;
        self.free.push(local);
    }

    pub fn get_local_reg(&mut self, _slot: u32, current_ip: usize, last_use_ip: usize) -> CompileResult<u8> {
        self.alloc(current_ip, last_use_ip)
    }

    pub fn live_count(&self) -> usize {
        self.next as usize - self.free.len()
    }

    /// Allocate a contiguous block of `count` registers.
    /// Returns the GLOBAL register index of the first register.
    /// All registers in the block are marked as live until `last_use_ip`.
    pub fn alloc_contiguous(&mut self, count: u8, current_ip: usize, last_use_ip: usize) -> CompileResult<u8> {
        self.expire(current_ip);
        if count == 0 {
            return Ok(self.base);
        }
        // Simple strategy: allocate from the end of the current range to avoid
        // fragmentation with the normal alloc/free cycle.
        let start = self.next;
        if start.saturating_add(count) > REGS_PER_ALLOC {
            return Err(compile_codes::generic(
                format!("RegAlloc: out of contiguous registers (base={}, next={}, count={})", self.base, self.next, count)
            ));
        }
        for i in 0..count {
            let local = start + i;
            self.active.push((last_use_ip, local));
        }
        self.active.sort_by_key(|(end, _)| *end);
        self.next = start + count;
        Ok(self.base + start)
    }
}

impl Drop for RegAlloc {
    fn drop(&mut self) {
        NEXT_BASE.with(|c| c.set(c.get().saturating_sub(REGS_PER_ALLOC)));
    }
}

// ---------------------------------------------------------------------------
// Temp register — DEPRECATED. Prefer RegAlloc::alloc() + free_now() so that
// registers are properly reclaimed. This per-thread path exists only for legacy
// call sites and is capped to prevent u8 wrap-around collisions with zone 0.
// ---------------------------------------------------------------------------
const TEMP_REG_BASE: u8 = 128;
const TEMP_REG_LIMIT: u8 = 254; // leave 255 as sentinel / spare

thread_local! {
    static NEXT_TEMP: Cell<u8> = const { Cell::new(TEMP_REG_BASE) };
}

pub fn temp_reg() -> u8 {
    NEXT_TEMP.with(|c| {
        let current = c.get();
        if current < TEMP_REG_BASE {
            c.set(TEMP_REG_BASE + 1);
            return TEMP_REG_BASE;
        }
        let next = if current >= TEMP_REG_LIMIT {
            TEMP_REG_BASE
        } else {
            current + 1
        };
        c.set(next);
        current
    })
}

/// Reset the per-thread temp register counter. Call this at the start
/// of each function compilation to prevent temp register leaks.
pub fn reset_temp_reg() {
    NEXT_TEMP.with(|c| c.set(TEMP_REG_BASE));
}

/// Reset the per-thread base counter. Call at the start of each function
/// compilation so RegAlloc zones don't accumulate across function bodies.
pub fn reset_base() {
    NEXT_BASE.with(|c| c.set(0));
}
