//! VM state sub-types used by `machine.rs`.
//!
//! Split out to keep `machine.rs` under the 400-line source limit.

use hudhudscript_bytecode::{FunctionChunk, SymId};
use std::sync::Arc;

/// PERF0011: Combined pre-computed chunk metadata cache.
/// Merges packed instructions + local symbol info into one lookup
/// (was two separate FxHashMaps with the same key).
#[derive(Clone)]
pub(crate) struct ChunkCache {
    pub(crate) packed: Arc<Vec<u32>>,
    pub(crate) local_syms: Arc<Vec<(u32, usize, Option<usize>)>>,
    pub(crate) max_sym: u32,
}

/// T3-1-B: Call frame for the trampoline loop.
pub(crate) struct CallFrame {
    pub(crate) chunk_ptr: *const FunctionChunk,
    pub(crate) packed: *const Vec<u32>,
    pub(crate) func_sym: SymId,
    pub(crate) ip: usize,
    pub(crate) dst: u8,
    pub(crate) reg_base: usize,
    pub(crate) reg_size: usize,
    pub(crate) saved_finally: Option<Box<crate::vm::types::SavedFinally>>,
    pub(crate) has_captures: bool,
    pub(crate) debugger_pushed: bool,
    pub(crate) call_depth: usize,
    pub(crate) owned_local_syms: bool,
    pub(crate) class_context: bool,
}
