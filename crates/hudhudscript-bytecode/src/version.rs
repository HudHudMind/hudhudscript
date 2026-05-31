use crate::{
    Bytecode, CallPayload, ClassDeclPayload, ClassStaticDeclPayload, DefineFunctionPayload,
    DestructObjectPayload, EnumDeclPayload, FunctionChunk, Instruction, LoadModulePayload,
    LoopPayload, OptSymPayload, SymId, TraitCheckPayload, TwoSymPayload, Value16,
};
/// Promise state — re-exported from `hudhudscript-errors` (Issue #901).
pub type PromiseState = hudhudscript_errors::PromiseState<Value16>;

/// Generator state — re-exported from `hudhudscript-errors` (Issue #900).
pub type GeneratorState = hudhudscript_errors::GeneratorState<Value16>;

/// Promise state for Value16 — re-exported from `hudhudscript-errors`.
pub type PromiseState16 = hudhudscript_errors::PromiseState<Value16>;

/// Generator state for Value16 — re-exported from `hudhudscript-errors`.
pub type GeneratorState16 = hudhudscript_errors::GeneratorState<Value16>;

/// Expected bytecode format version. Incremented when the bytecode layout changes
/// in a backwards-incompatible way.
///
/// # Version history
/// - v4: initial compact-symbol + packed-numeric format.
/// - v5: `FunctionChunk::source_positions` added for per-instruction
///   file/line tracking inside function bodies (DAP / debugger step-into).
/// - v6: Jump offsets `usize` → `i32` relative (PERF-12 / S2.3).
/// - v7: `LoadLocal(u32)` / `StoreLocal(u32)` / `DeclLocal(u32)` added
///   as slot-direct local-variable instructions (S2.2a / Audit Finding 3.1).
///   Compiler doesn't emit them yet — subsequent sub-issues (S2.2b/c)
///   migrate function-body locals off the name-indirection path.
/// - v8: Wire encoding migrated bincode → postcard (PERF-49, Audit v3
///   Finding 15.1).  Postcard varint encoding is typically 40-60% smaller
///   than bincode's fixed-width layout.  Old bincode `.hudb`/`.hudc` caches
///   are rejected at load — recompile via `hudhud compile` to regenerate.
/// - v9: `TailCall(SymId, u8)` added for tail-call optimisation (CROSS-1).
///   Self-recursive tail calls emit `TailCall` instead of `Call + Return`;
///   the VM rebinds params in-place and resets ip to 0 without pushing a
///   new stack frame. Invariant: compiler only emits `TailCall` for
///   calls whose callee is the current function. Other-function tail
///   calls remain a future extension (cross-fn TCO).
/// - v10: `Instruction` enum shrunk 24 B → 16 B (CROSS-2b):
///   * `LoopBegin(usize, usize)` → `LoopBegin(u32)` — start/end externalised
///     to the new side table `Bytecode::loop_payloads: Vec<LoopPayload>`.
///   * `LoadConst(usize)` / `LoadNumConst(usize)` → `LoadConst(u32)` /
///     `LoadNumConst(u32)` — pool indexing capped at 2^32 (the constants
///     / numeric_constants pools never exceed that in practice).
///   Cache-line-friendly dispatch: one 16 B instruction fits in 1/4 of a
///   64 B L1 line, vs 1/2.67 for 24 B.
/// - v11: `Instruction` enum alignment 8 B → 4 B (CROSS-2a).
///   Seven `Box<T>` variants were externalised to dedicated side-table
///   pools, eliminating the last 8-byte-aligned payloads and unblocking
///   future 12 B / 8 B enum-size targets (CROSS-2c / CROSS-2d).
///   * `EnumDecl(Box<(SymId, Vec<SymId>)>)` → `EnumDecl(u32)` indexing
///     `Bytecode::enum_decl_payloads: Vec<EnumDeclPayload>`.
///   * `ClassDecl(Box<(SymId, Option<SymId>, Vec<SymId>)>)` → `ClassDecl(u32)`
///     indexing `Bytecode::class_decl_payloads: Vec<ClassDeclPayload>`.
///   * `TraitCheck(Box<(SymId, SymId, Vec<SymId>, Vec<SymId>)>)` →
///     `TraitCheck(u32)` indexing
///     `Bytecode::trait_check_payloads: Vec<TraitCheckPayload>`.
///   * `LoadModule(Box<(String, Option<SymId>)>)` → `LoadModule(u32)`
///     indexing `Bytecode::load_module_payloads: Vec<LoadModulePayload>`.
///   * `DefineFunction(Box<(SymId, String)>)` → `DefineFunction(u32)`
///     indexing `Bytecode::define_function_payloads: Vec<DefineFunctionPayload>`.
///   * `ClassStaticDecl(Box<(SymId, Vec<SymId>)>)` → `ClassStaticDecl(u32)`
///     indexing `Bytecode::class_static_decl_payloads: Vec<ClassStaticDeclPayload>`.
///   * `DestructObject(Box<Vec<SymId>>)` → `DestructObject(u32)` indexing
///     `Bytecode::destruct_object_payloads: Vec<DestructObjectPayload>`.
/// - v12: `Instruction` enum shrunk 12 B → 8 B (CROSS-2c+2d).
///   14 remaining (SymId, u8) / two-sym / Option<SymId> variants had
///   their payloads externalised to three pools:
///   * 7 call-family variants — `Call`, `TailCall`, `MethodCall`,
///     `NewInstance`, `Spawn`, `SuperCall`, `MakeGenerator` — each
///     `(SymId, u8)` → `u32` indexing
///     `Bytecode::call_payloads: Vec<CallPayload>`.
///   * 4 two-symbol variants — `StoreTyped`, `MatchVariant`, `DeclStore`,
///     `GetStatic` — each `(sym, sym)` → `u32` indexing
///     `Bytecode::two_sym_payloads: Vec<TwoSymPayload>`.
///   * 3 optional-symbol variants — `Remember`, `Recall`, `Forget` —
///     each `Option<SymId>` → `u32` indexing
///     `Bytecode::opt_sym_payloads: Vec<OptSymPayload>`.
///   Every remaining Instruction variant now carries at most a single
///   4-byte operand, so the enum layout collapses to 8 bytes total
///   (4-byte tag + 4-byte payload).  Eight instructions fit per 64 B
///   L1 line vs four at 16 B — dispatch cache locality is the
///   dominant win for fib / tight-loop workloads.
///
/// - v15: `Value16::int(i64)` variant added alongside `Value16::number(f64)`
///   (A3a scaffolding — Lua/Julia Option C integer path). No opcode emits
///   `Int` yet; the variant is dormant until A3b adds `LoadIntConst` and
///   A3c adds `IntAdd` / `IntSub` / `IntMul` fast-path dispatch. Every
///   pattern-match site handles `Int(i)` as "widen `i as f64` and run the
///   Number path" so end-to-end semantics stay identical to pre-A3a.
///   `CompactValue` receives the mirror change (`CompactValue16::int(i64)`).
/// - v16: A3b — `LoadIntConst(u32)` + `NumberFromInt` opcodes; the
///   compiler now emits integer-valued numeric literals as
///   `Value16::int(i64)` via the new `int_constants` pool, and the runtime
///   widens `Int` back to `Number` through the `pop_number` helper so
///   existing `NumAdd`/`NumSub`/... arms stay untouched.  `NumberFromInt`
///   is a stack-top widening step reserved for explicit float-expected
///   call sites (constant folding still lowers to f64 on mixed-type
///   operations).  Fast-path `IntAdd`/`IntSub`/... is the A3c follow-up
///   — this version only flips the Int producer on.
/// - v17: A3c — integer fast-path opcodes.  Adds 12 base `IntAdd` /
///   `IntSub` / `IntMul` / `IntDiv` / `IntMod` / `IntNeg` / `IntLt` /
///   `IntLe` / `IntGt` / `IntGe` / `IntEq` / `IntNe` arithmetic ops that
///   consume `Value16::int(i64)` directly (no f64 widening), plus 7 fused
///   `IntSubISlot` / `IntAddISlot` / `IntLtISlot` / `IntLeISlot` /
///   `IntGtISlot` / `IntGeISlot` / `IntEqISlot` variants paralleling the
///   I6 `Num*ISlot` family for integer operands.  The compiler's
///   `infer_type` now distinguishes `Int` from `Number`: pure-integer
///   expressions (`n - 1` where both sides are proven `Int`) emit the
///   new `IntXxx` opcodes; mixed `Int × Number` inserts an explicit
///   `NumberFromInt` widen step to keep the result `Number`-typed.
///   Wrapping overflow semantics (Rust `wrapping_*`) for Int arithmetic,
///   matching Lua/Julia integer-overflow behavior.  Kural 7c: no silent
///   coercion — every widening is explicit via `NumberFromInt`.
/// - v18: A2 — super-instruction fusion.  Adds three fused opcodes that
///   collapse consecutive hot bigrams into a single dispatch step:
///   * `LeJumpIfFalse(i32)` — fuses `Le + JumpIfFalse(offset)` (fib base
///     case guard).  Pops two operands, compares with `<=`, and branches
///     to `ip + offset` when the result is false.  Matches the branch-
///     if-not-greater idiom hot in recursion base cases and loop bounds.
///   * `IntSubCall1 { call_idx: u32, slot: u32, imm: i16 }` — fuses
///     `IntSubISlot{slot, imm} + Call(call_idx)` when the call payload's
///     arg_count equals 1.  Computes `local_slots[slot] - imm` and
///     immediately dispatches the call with that single argument on the
///     stack.  Fires 2× per fib frame (`fib(n-1)`, `fib(n-2)`).
///   * `AddReturn` — fuses `Add + Return` (fib result-combining tail).
///     Executes the generic `Add` then signals the outer dispatcher to
///     return.  Preserves `shared_add` semantics (string concat, mixed-
///     type numeric promotion) so it is safe to emit anywhere `Add +
///     Return` appears.
///   Addresses A3c's observation that fib's bottleneck is dispatch count
///   (~1.25 ns/instr × 2^30 calls), not arithmetic: each fused opcode
///   removes one dispatch step per fire.  Kural 7c: super-instructions
///   are pattern-matched, not fallback — each combines two opcodes whose
///   semantics compose exactly.  The peephole runs AFTER
///   `fuse_slot_immediate_with_positions` and re-stitches jump offsets
///   and loop-payload indices identically to the I6 fusion pass.
pub const BYTECODE_VERSION: u32 = 21;
