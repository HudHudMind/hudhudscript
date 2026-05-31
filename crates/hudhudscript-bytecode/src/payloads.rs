use crate::{Bytecode, Instruction, Repr, SymId, Value16};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoopPayload {
    pub start: u32,
    pub end: u32,
}

/// Payload for `Instruction::EnumDecl(u32)` (CROSS-2a).
///
/// Stored externally so the instruction carries only a compact `u32`
/// index — previously `Box<(SymId, Vec<SymId>)>` forced the enum's
/// alignment to 8 B via the `Box` pointer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumDeclPayload {
    pub name: SymId,
    pub variants: Vec<SymId>,
}

/// Payload for `Instruction::ClassDecl(u32)` (CROSS-2a).
///
/// Per method, a corresponding `FunctionChunk` must exist in
/// `bytecode.functions`.  Parent name is optional (set when the class
/// extends another class).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassDeclPayload {
    pub name: SymId,
    pub parent: Option<SymId>,
    pub methods: Vec<SymId>,
    /// Per-method access flags: 0=Public, 1=Private, 2=Protected (same order as methods)
    pub method_access: Vec<u8>,
    /// OOP0004: whether this class is declared abstract
    #[serde(default)]
    pub is_abstract: bool,
}

/// Payload for `Instruction::TraitCheck(u32)` (CROSS-2a).
///
/// Issue #982: SOP trait/protocol enforcement.  Verifies at class
/// declaration time that all required trait methods are present on
/// the class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitCheckPayload {
    pub class_name: SymId,
    pub trait_name: SymId,
    pub required_methods: Vec<SymId>,
    pub class_methods: Vec<SymId>,
}

/// Payload for `Instruction::LoadModule(u32)` (CROSS-2a).
///
/// `path` stays a `String` (file path, not a symbol); `alias` is an
/// optional `SymId` for the variable name bound to the loaded module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadModulePayload {
    pub path: String,
    pub alias: Option<SymId>,
}

/// Payload for `Instruction::DefineFunction(u32)` (CROSS-2a).
///
/// `name` is the variable name the function is bound to; `chunk_name`
/// is the key into `Bytecode::functions` (stays a `String` because it
/// is used directly as a `HashMap` key).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefineFunctionPayload {
    pub name: SymId,
    pub chunk_name: String,
}

/// Payload for `Instruction::ClassStaticDecl(u32)` (CROSS-2a).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassStaticDeclPayload {
    pub class_name: SymId,
    pub static_methods: Vec<SymId>,
    /// Static field names (value stored in constant pool at corresponding index)
    pub static_fields: Vec<SymId>,
}

/// Payload for `Instruction::DestructObject(u32)` (CROSS-2a).
///
/// Issue #668: the list of already-used keys when destructuring into
/// a `{ ...rest }` binding — the VM builds the rest object by
/// excluding these keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestructObjectPayload {
    pub used_keys: Vec<SymId>,
}

/// Payload for the 7 call-family instructions (CROSS-2c).
///
/// `Call`, `TailCall`, `MethodCall`, `NewInstance`, `Spawn`,
/// `SuperCall`, and `MakeGenerator` all previously carried a
/// `(SymId, u8)` tuple inline on the enum.  CROSS-2c moves every one of
/// them to this shared side-table pool so the instruction operand
/// shrinks to a single `u32` index — the last step that lets
/// `Instruction` drop from 12 B to 8 B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallPayload {
    pub sym: SymId,
    pub arg_count: u8,
}

/// Payload for the 4 two-symbol instructions (CROSS-2d).
///
/// Backs `StoreTyped`, `MatchVariant`, `DeclStore`, and `GetStatic`.
/// The fields are bare `u32`s because `StoreTyped`'s first operand is a
/// raw symbol-table index (from `Bytecode::intern_symbol`) while the
/// other three carry two `SymId`s; both round-trip through the
/// underlying `u32` without loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwoSymPayload {
    pub first: u32,
    pub second: u32,
}

/// Payload for A2 super-instructions that fuse three operands (call
/// index + slot + immediate).  `IntSubCall1` is the only current
/// consumer; new super-instructions that would otherwise inflate the
/// `Instruction` enum should allocate their payload here rather than
/// carry inline operands (Kural 7c — side-table pool is the single
/// source for wide super-instruction operands; no inline fallback).
///
/// The pool is populated at compile time; the VM resolves by u32 index
/// on dispatch via `Bytecode::get_super_instr_payload`.  Eight bytes
/// per entry (4 + 4 + 2 → padded to 12 on some targets, but the side
/// table never limits Instruction size).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuperInstrPayload {
    pub call_idx: u32,
    pub slot: u32,
    pub imm: i16,
    /// A2 extension: fused branch offset for `Int*JumpIfFalse` family.
    /// Unused by `IntSubCall1` / `IntAddCall1` (set to 0).
    #[serde(default)]
    pub offset: i32,
}

/// Payload for the 3 optional-symbol instructions (CROSS-2d).
///
/// Backs `Remember`, `Recall`, `Forget` — each one previously carried
/// an `Option<SymId>` (target store name) inline.  `Option<SymId>` is
/// 8 bytes on the enum (niche fill in the discriminant), which alone
/// prevented the 8 B goal.  Externalising keeps the cold data off the
/// dispatch path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptSymPayload {
    pub sym: Option<SymId>,
}

/// A compiled function body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChunk {
    /// Parameter names (in order)
    pub params: Vec<String>,
    /// Bytecode instructions for the body
    pub instructions: Vec<Instruction>,
    /// Constants local to this chunk (Struct-3d-b migration: now Value16)
    pub constants: Vec<Value16>,
    /// Names of variables captured from the enclosing scope (closures / arrow functions)
    #[serde(default)]
    pub captures: Vec<String>,
    /// Whether this function is async
    #[serde(default)]
    pub is_async: bool,
    /// Whether this function is a generator (uses yield) — Issue #667
    #[serde(default)]
    pub is_generator: bool,
    /// Number of local variable slots this function needs (Issue #1033 P2).
    /// The VM pre-allocates a `Vec<Value>` of this size on function entry
    /// for O(1) indexed local access instead of HashMap lookup.
    #[serde(default)]
    pub local_count: u32,
    /// Ordered local variable names — index i corresponds to slot i.
    /// Used by the VM to map symbol indices to local slots.
    #[serde(default)]
    pub local_names: Vec<String>,
    /// Upvalue cell IDs for captured variables (Issue #1066, P9.2).
    ///
    /// Each entry is `(variable_name, cell_id)` where `cell_id` is the
    /// unique upvalue cell identifier assigned by the compiler.  The
    /// compiler will migrate from the flat `captures: Vec<String>` to
    /// this field; both coexist for backward compatibility during the
    /// transition.
    #[serde(default)]
    pub capture_cells: Vec<(String, u32)>,
    /// Highest register index emitted by the compiler for this function.
    /// VM uses this to save only registers [0..=max_register] on call.
    /// 0 means "unknown" → fall back to local_count + 64 formula.
    #[serde(default)]
    pub max_register: u8,
    /// Cached symbol-to-slot mapping for fast local variable lookup.
    /// Lazily initialized on first call; eliminates HashMap lookup per call.
    #[serde(skip)]
    pub sym_to_slot: std::sync::OnceLock<Arc<Vec<i32>>>,

    /// Compile-time parameter → slot mapping.
    /// `param_slots[i]` is the local slot index for parameter `i`.
    /// Eliminates per-call name lookup when binding arguments.
    #[serde(default)]
    pub param_slots: Box<[u16]>,
    /// Per-instruction source positions (line, column) for this chunk.
    ///
    /// Parallel to `instructions` — entry `i` corresponds to
    /// `instructions[i]`. `None` means no position info available
    /// (e.g. compiler-emitted plumbing like implicit `return null`).
    /// Populated at statement boundaries by the compiler so the DAP
    /// debugger can step into function bodies and resolve
    /// `file:line` for breakpoints inside closures / methods.
    ///
    /// `#[serde(default)]` keeps older (v4) bytecode round-trippable
    /// at the serde layer — older chunks simply deserialize with an
    /// empty `source_positions` vector, which the VM's debug hook
    /// treats the same as "no position info" (hook is a no-op).
    #[serde(default)]
    pub source_positions: Vec<Option<(usize, usize)>>,
}

impl FunctionChunk {
    /// Returns `true` if this chunk uses the new cell-based capture
    /// representation (P9.2).  When `true`, the VM should use
    /// `capture_cells` instead of `captures` for upvalue resolution.
    #[inline]
    pub fn has_cell_captures(&self) -> bool {
        !self.capture_cells.is_empty()
    }

    /// Append a source position corresponding to the next emitted instruction.
    ///
    /// Mirrors [`Bytecode::push_source_position`] so function-body
    /// compilation can record per-instruction file/line info for the
    /// DAP debugger's `on_statement` hook.
    #[inline]
    pub fn push_source_position(&mut self, pos: Option<(usize, usize)>) {
        self.source_positions.push(pos);
    }

    /// Look up the source position for a given instruction index inside
    /// this function chunk, returning `None` if the table is empty
    /// (older bytecode) or the index has no position info.
    #[inline]
    pub fn get_source_position(&self, ip: usize) -> Option<(usize, usize)> {
        self.source_positions.get(ip).copied().flatten()
    }

    /// Push an instruction and keep `source_positions` parallel by
    /// appending `None`. Mirror of [`Bytecode::push_instr`].
    #[inline]
    pub fn push_instr(&mut self, instr: Instruction) {
        self.instructions.push(instr);
        self.source_positions.push(None);
    }

    /// Pad `source_positions` to match `instructions.len()` —
    /// post-compile normalization helper. See
    /// [`Bytecode::pad_source_positions`] for rationale.
    pub fn pad_source_positions(&mut self) {
        while self.source_positions.len() < self.instructions.len() {
            self.source_positions.push(None);
        }
        self.source_positions.truncate(self.instructions.len());
    }
}

/// Bytecode value
// ===================================================================
// PERF-2 (2026-04-17) — heavy Value variants Box'd to keep Value small.
//
// Pattern adopted from Rune (`crates/rune/src/runtime/value.rs` — Repr
// with NonNull pointers for heavy variants) and Gluon (`vm/src/value.rs`
// — all non-primitive variants are `GcPtr<T>` thin pointers).  Every
// variant that previously inlined a HashMap/Vec is now a single-word
// pointer.  Pre-PERF-2 Value = 176 bytes (enum padded to largest
// variant, which was `Class` holding 3 HashMaps).  Post-PERF-2 target
// ≤ 32 bytes (size_of test below enforces it).  Zero feature loss;
// only memory layout changes.
// ===================================================================

/// Backing storage for `Value::Function`.  Kept as a struct so the
/// enum variant is a single-pointer `Box<FunctionData>` (8 bytes) rather
/// than inlining ~144 bytes of strings + HashMap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionData {
    pub name: String,
    pub params: Vec<String>,
    /// Index into Bytecode::functions
    pub chunk_name: String,
    /// Captured variables from enclosing scopes (closures / arrow functions).
    /// Stored as shared upvalue cells (`Arc<RwLock<Value16>>`) — Lua 5.2+ /
    /// interpreter-parity semantics (Issue #1078, Kural 2).
    #[serde(default, with = "crate::captures_serde")]
    pub captures: std::collections::HashMap<String, Arc<RwLock<Value16>>>,
}

/// Backing storage for `Value::Class`.  Previously inlined as a
/// struct-like variant with 3 HashMaps — the dominant 176-byte driver of
/// the whole `Value` enum's padded size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassData {
    pub name: String,
    pub methods: std::collections::HashMap<String, Value16>,
    pub fields: std::collections::HashMap<String, Value16>,
    pub parent: Option<Value16>,
    /// Precomputed method resolution table (flattened parent chain)
    pub vtable: std::collections::HashMap<String, Value16>,
    /// Per-method access flags: 0=Public, 1=Private, 2=Protected
    #[serde(default)]
    pub method_access: std::collections::HashMap<String, u8>,
    /// OOP0004: whether this class is abstract
    #[serde(default)]
    pub is_abstract: bool,
}

/// Backing storage for `Value::Instance`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceData {
    pub class_name: String,
    pub fields: std::collections::HashMap<String, Value16>,
    pub class: Value16,
}

/// Backing storage for `Value::Data` (anonymous typed record, no methods).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataData {
    pub type_name: String,
    pub fields: std::collections::HashMap<String, Value16>,
}

/// Backing storage for `Value::Tool` (MCP tool reference — 2 Strings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRef {
    pub server: String,
    pub tool_name: String,
}

/// Backing storage for `Value::Resource` (MCP resource reference — 2 Strings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    pub server: String,
    pub uri: String,
}
