//! VM state sub-types used by `machine.rs`.
//!
//! Split out to keep `machine.rs` under the 400-line source limit.

use crate::vm::call_state::{ReceiverContext, ReturnSink};
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
#[doc(hidden)]
pub struct CallFrame {
    pub chunk_ptr: *const FunctionChunk,
    /// Owns deferred chunks that are not guaranteed to live in `Bytecode`.
    pub owned_chunk: Option<Arc<FunctionChunk>>,
    pub packed: *const Vec<u32>,
    pub func_sym: SymId,
    pub ip: usize,
    pub dst: u8,
    pub reg_base: usize,
    pub reg_size: usize,
    pub saved_finally: Option<Box<crate::vm::types::SavedFinally>>,
    pub has_captures: bool,
    pub debugger_pushed: bool,
    pub call_depth: usize,
    pub owned_local_syms: bool,
    pub class_context: bool,
    pub return_sink: ReturnSink,
    pub receiver_context: Option<Box<ReceiverContext>>,
    /// SOP effect frames discard body errors instead of unwinding them.
    pub swallow_error: bool,
    /// BULGU6: çerçeve GİRİŞİNDEKİ try_frames derinliği — teardown, dökülen
    /// callee try çerçevelerini (koordinatları callee birimine ait eski
    /// catch_abs) bu derinliğe budar; yoksa ana koddaki sonraki Throw
    /// yabancı koordinata atlar (fabrika: Throw@1469 → ObjLitSet@157 panik).
    pub try_depth: usize,
    /// BULGU6: tekil çerçeve seri numarası (try_frames sahiplik ayrımı).
    pub serial: usize,
}
