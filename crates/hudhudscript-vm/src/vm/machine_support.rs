use super::machine::VM;
use hudhudscript_bytecode::Value16;

impl VM {
    /// GATE-2: record BigInt promotion/allocation if operands were plain Int.
    /// No-op when telemetry feature is disabled.
    #[inline(always)]

    pub(crate) fn record_bigint_promotion(&mut self, a: Value16, b: Value16, result: Value16) {
        #[cfg(feature = "telemetry")]
        if a.is_int() && b.is_int() && result.is_bigint() {
            self.telemetry.bigint_promotion += 1;
            self.telemetry.bigint_alloc += 1;
        }
        let _ = (a, b, result);
    }
}

impl VM {
    /// Return the last function return value (the result object).
    pub fn last_return_value(&self) -> Value16 {
        self.last_return
    }

    /// Snapshot of GATE-2 telemetry counters (telemetry feature only).
    #[cfg(feature = "telemetry")]
    pub fn telemetry_snapshot(&self) -> crate::vm::telemetry::TelemetrySnapshot {
        let mut snap = self.telemetry.snapshot();
        hudhudscript_bytecode::gc::take_telemetry_alloc_counts(&mut snap.alloc_count_by_kind);
        hudhudscript_bytecode::gc::take_telemetry_gc_stats(
            &mut snap.gc_cycle_count,
            &mut snap.gc_mark_count,
            &mut snap.gc_sweep_count,
            &mut snap.gc_pause_ns_total,
            &mut snap.gc_pause_ns_max,
            &mut snap.gc_heap_bytes_after_sweep,
        );
        snap.int_add_slow_count = crate::vm::math_fast_paths::take_int_add_slow_count();
        snap
    }

    /// P3: fast SymbolId-indexed top-level slot lookup.
    /// `u32::MAX` means no slot.  Low byte = slot index, bit 8 = shared flag.
    #[inline(always)]
    pub(crate) fn main_slot_encoded(&self, sym_id: u32) -> Option<u32> {
        let idx = sym_id as usize;
        if idx < self.main_local_slots.len() {
            let enc = self.main_local_slots[idx];
            if enc != u32::MAX {
                Some(enc)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline(always)]
    pub(crate) fn main_slot_decode(encoded: u32) -> (usize, bool) {
        ((encoded & 0xFF) as usize, ((encoded >> 8) & 1) != 0)
    }

    #[inline(always)]
    pub(crate) fn main_slot_shared_index(encoded: u32) -> usize {
        (encoded >> 9) as usize
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        // PERF-T1-6: Free the leaked Box<Vec<String>> allocations in call_cache.
        for entry in self.call_cache.drain(..) {
            if let Some((_, _, params_ptr)) = entry {
                if !params_ptr.is_null() {
                    unsafe { drop(Box::from_raw(params_ptr as *mut Vec<String>)) };
                }
            }
        }
        // PERF-T2-3: Free owned local_sym_refs (run.rs / generator allocations).
        for ptr in self.owned_local_sym_refs.drain(..) {
            if !ptr.is_null() {
                unsafe { drop(Box::from_raw(ptr as *mut Vec<(u32, usize, Option<usize>)>)) };
            }
        }
    }
}

// SAFETY: VM contains raw pointers (*mut Value16 in RegisterArena,
// *const FunctionChunk in call_cache) but they always point to data
// owned by the same VM instance (arena Vec) or by the Bytecode
// (FunctionChunk), both of which outlive any thread spawn.
unsafe impl Send for VM {}

pub struct HeapGuard {
    prev: *mut hudhudscript_bytecode::gc::GcHeap,
}

impl HeapGuard {
    pub fn new(vm: &mut VM) -> Self {
        let heap_ptr = vm.gc_heap.as_mut() as *mut _;
        let prev = hudhudscript_bytecode::gc::CURRENT_HEAP.with(|c| c.get());
        hudhudscript_bytecode::gc::CURRENT_HEAP.with(|c| c.set(heap_ptr));
        Self { prev }
    }
}

impl Drop for HeapGuard {
    fn drop(&mut self) {
        hudhudscript_bytecode::gc::CURRENT_HEAP.with(|c| c.set(self.prev));
    }
}
