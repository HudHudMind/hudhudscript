use crate::{
    CallPayload, ClassDeclPayload, ClassStaticDeclPayload, DefineFunctionPayload,
    DestructObjectPayload, EnumDeclPayload, FunctionChunk, Instruction, LoadModulePayload,
    LoopPayload, OptSymPayload, SuperInstrPayload, SymId, TraitCheckPayload, TwoSymPayload,
    Value16,
};
use serde::{Deserialize, Serialize};
/// Bytecode program
use std::cell::RefCell;
use std::sync::Arc;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bytecode {
    /// Version of bytecode format
    pub version: u32,
    /// Constant pool (Struct-3d-b migration: now Value16)
    pub constants: Vec<Value16>,
    /// Instructions
    pub instructions: Vec<Instruction>,
    /// Named function chunks (name -> Arc<FunctionChunk>)
    /// Wrapped in Arc to avoid expensive deep clones on function lookup (#458).
    pub functions: std::cell::RefCell<std::collections::HashMap<String, Arc<FunctionChunk>>>,
    /// Agent action chunks — compiled action bodies keyed by "AgentName.actionName".
    /// These are separate from regular functions because they are dispatched
    /// via `perform` rather than regular call.
    #[serde(default)]
    pub action_registry: std::cell::RefCell<std::collections::HashMap<String, Arc<FunctionChunk>>>,
    /// Source position map: one entry per instruction (line, column).
    /// Used for error reporting (#460). `None` entries mean no position info available.
    #[serde(default)]
    pub source_positions: Vec<Option<(usize, usize)>>,
    /// Packed numeric constant pool (NaN-boxed, 8 bytes each).
    ///
    /// Numeric constants (the most common constant type in arithmetic code)
    /// are stored here as NaN-boxed `u64` values instead of the 176-byte
    /// `Value` enum.  The compiler emits `LoadNumConst(idx)` to reference
    /// entries in this pool.
    #[serde(default)]
    pub numeric_constants: Vec<u64>,
    /// Integer constant pool (A3b - `LoadIntConst(idx)`).
    ///
    /// Populated by the compiler when a numeric literal is integer-valued
    /// (finite, whole, and within `i64` range).  Stored as raw `i64`
    /// alongside the float pool so mixed-type programs can reference
    /// either without widening at compile time.  The runtime widens back
    /// to `f64` inside `pop_number` when existing `NumAdd` / `NumSub` /
    /// ... arms consume the value - fast-path `IntAdd` / `IntSub` lands
    /// in A3c.
    #[serde(default)]
    pub int_constants: Vec<i64>,
    /// Symbol table for string interning (Issue #1032, P1).
    ///
    /// Instructions like `LoadVar`, `StoreVar`, `DeclVar`, `StoreConst`
    /// carry a `u32` index into this table instead of a heap-allocated
    /// `String`. The compiler populates the table during emission via
    /// [`Bytecode::intern_symbol`]; the VM resolves indices back to
    /// `&str` via [`Bytecode::resolve_symbol`].
    ///
    /// This shrinks hot-path instructions from ~56 bytes (with String)
    /// to ~8 bytes (with u32 index), improving L1 cache utilization
    /// and eliminating per-instruction String allocation.
    #[serde(default)]
    pub symbols: Vec<String>,
    /// Side table for compressed symbol lists (Issue #1059, P7.2).
    ///
    /// Instruction variants like `ClassDecl`, `TraitCheck`, `EnumDecl`,
    /// `ClassStaticDecl`, and `DestructObject` carry `Vec<SymId>` payloads
    /// that bloat the `Instruction` enum size.  This table stores those
    /// lists externally; the instruction then carries a compact `u32` index
    /// into this table instead.
    ///
    /// Populated by the compiler via [`Bytecode::add_symbol_list`];
    /// resolved by the VM via [`Bytecode::get_symbol_list`].
    #[serde(default)]
    pub symbol_lists: Vec<Vec<SymId>>,
    /// Reverse index for O(1) symbol interning - `name -> idx into symbols`.
    /// Kept in sync with `symbols`.  Not serialized - rebuilt lazily on first
    /// intern after deserialization.  Audit v3 Finding 2.1, Knuth TAOCP
    /// v.3 §6.4 hash-table amortized O(1).
    #[serde(skip)]
    pub(crate) symbol_index: rustc_hash::FxHashMap<String, u32>,
    /// Reverse index for O(1) numeric constant dedup - `f64 bits -> idx
    /// into numeric_constants`.  Audit v3 Finding 2.2.
    #[serde(skip)]
    pub(crate) numeric_index: rustc_hash::FxHashMap<u64, u32>,
    /// Reverse index for O(1) integer constant dedup - `i64 -> idx into
    /// int_constants` (A3b).  Kept in sync with `int_constants` on insert,
    /// not serialised (rebuilt lazily on first intern after deserialisation
    /// via `rebuild_indices_if_stale`).
    #[serde(skip)]
    pub(crate) int_index: rustc_hash::FxHashMap<i64, u32>,
    /// Reverse index for O(1) symbol-list dedup - `Vec<SymId> -> idx
    /// into symbol_lists`.  Audit v3 Finding 2.2 extension (add_symbol_list
    /// used O(N·L) nested compare across all stored lists on every call).
    #[serde(skip)]
    pub(crate) symbol_list_index: rustc_hash::FxHashMap<Vec<SymId>, u32>,
    /// Side table for loop header payloads (CROSS-2b).
    ///
    /// `Instruction::LoopBegin(u32)` carries a compact index into this
    /// pool instead of inlining two `usize` fields.  Each entry records
    /// the loop's start IP (back-edge target) and end IP (exit target)
    /// as `u32` values.  Shrinks `LoopBegin` from a 24-byte enum variant
    /// to 16 bytes total (8-byte tag + 4-byte idx + padding), driving
    /// the overall `Instruction` enum down to 16 B.
    ///
    /// Keeping this as a parallel side table (instead of boxing) avoids
    /// per-`LoopBegin` heap allocation - the pool is contiguous, rarely
    /// grows beyond a handful of entries per function, and amortizes
    /// cache misses with sequential access.
    #[serde(default)]
    pub loop_payloads: Vec<LoopPayload>,

    // ── CROSS-2a: side-table pools for the 7 declaration-level variants
    // that previously boxed their payloads.  Moving these off the enum
    // drops the tag's alignment requirement from 8 B (`Box<T>` pointer)
    // to 4 B (`u32` index), unlocking future enum-size cuts.
    /// Payloads for `Instruction::EnumDecl(u32)` (CROSS-2a).
    #[serde(default)]
    pub enum_decl_payloads: Vec<EnumDeclPayload>,
    /// Payloads for `Instruction::ClassDecl(u32)` (CROSS-2a).
    #[serde(default)]
    pub class_decl_payloads: Vec<ClassDeclPayload>,
    /// Payloads for `Instruction::TraitCheck(u32)` (CROSS-2a).
    #[serde(default)]
    pub trait_check_payloads: Vec<TraitCheckPayload>,
    /// Payloads for `Instruction::LoadModule(u32)` (CROSS-2a).
    #[serde(default)]
    pub load_module_payloads: Vec<LoadModulePayload>,
    /// Payloads for `Instruction::DefineFunction(u32)` (CROSS-2a).
    #[serde(default)]
    pub define_function_payloads: Vec<DefineFunctionPayload>,
    /// Payloads for `Instruction::ClassStaticDecl(u32)` (CROSS-2a).
    #[serde(default)]
    pub class_static_decl_payloads: Vec<ClassStaticDeclPayload>,
    /// Payloads for `Instruction::DestructObject(u32)` (CROSS-2a).
    #[serde(default)]
    pub destruct_object_payloads: Vec<DestructObjectPayload>,

    // ── CROSS-2c+d: three pools backing the final 14 variants whose
    // externalisation drove `Instruction` down from 12 B to 8 B.
    /// Payloads for the 7 call-family variants (`Call`, `TailCall`,
    /// `MethodCall`, `NewInstance`, `Spawn`, `SuperCall`,
    /// `MakeGenerator`).  Each instruction carries only a `u32` index
    /// into this pool (CROSS-2c+2d).  Not deduplicated - distinct call
    /// sites get distinct entries so future per-site tweaks (inline
    /// caching, call-count profiling) remain possible.
    #[serde(default)]
    pub call_payloads: Vec<CallPayload>,
    /// Payloads for the 4 two-symbol variants (`StoreTyped`,
    /// `MatchVariant`, `DeclStore`, `GetStatic`).  Shared struct
    /// `TwoSymPayload { first, second }` keeps the pool uniform -
    /// `StoreTyped`'s first field is a raw symbol index (not a `SymId`)
    /// while the other three carry `SymId(u32)` pairs; both round-trip
    /// through the bare `u32` storage.
    #[serde(default)]
    pub two_sym_payloads: Vec<TwoSymPayload>,
    /// Payloads for the 3 optional-symbol variants (`Remember`,
    /// `Recall`, `Forget`).  Keeps the `Option<SymId>` off the enum so
    /// the variant operand fits in a bare `u32` index.
    #[serde(default)]
    pub opt_sym_payloads: Vec<OptSymPayload>,
    /// A2 super-instruction payloads.  Keeps wide super-instruction
    /// operands (`call_idx`, `slot`, `imm`) off the `Instruction` enum
    /// so it stays at 8 bytes (CROSS-2c+d target).  Currently backs
    /// `Instruction::IntSubCall1(u32)`; new super-instructions that
    /// need three or more fields share this pool.
    #[serde(default)]
    pub super_instr_payloads: Vec<SuperInstrPayload>,
    /// PERF-B1: Top-level (main chunk) local variable slot names.
    ///
    /// Parallel to `FunctionChunk::local_names` but for the top-level
    /// instruction stream.  The VM populates `sym_to_slot` from these
    /// names in `execute()`, enabling slot-based fast-path for all
    /// `LoadVar`/`StoreVar`/`DeclVar` in top-level code (was slow
    /// HashMap path).  Index = slot number.
    #[serde(default)]
    pub main_local_names: Vec<String>,
    /// Number of top-level local variable slots (= main_local_names.len()).
    #[serde(default)]
    pub main_local_count: u32,
    /// Cached packed instructions (PERF-B2: avoid re-prepack on every execute).
    #[serde(skip)]
    pub packed: RefCell<Option<Vec<u32>>>,
}

/// Payload for `Instruction::LoopBegin` (CROSS-2b).
///
/// `start` is the loop-start IP (back-edge target - where the jump at the
/// bottom of the loop goes to re-evaluate the condition).  `end` is the
/// loop-end IP (exit target used by Break / condition-false paths).
/// Both are absolute IPs in the instruction stream, stored as `u32`
/// (max 4 billion instructions per compilation unit - far beyond any
/// realistic program).
impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}
