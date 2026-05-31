use crate::{FunctionChunk, SymId, Value16, BYTECODE_VERSION};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // ── Data movement (register-based equivalents below) ────────────
    // LoadNumConst/LoadIntConst removed — use LoadNumConst/LoadIntConst

    // ── Data structures ───────────────────────────────────────────────
    // IndexFast/IndexAssignFast removed — compiler now emits Index/IndexAssign

    // ── Control flow ──────────────────────────────────────────────────
    Jump(i32),
    JumpIfFalse { src: u8, offset: i16 },
    JumpIfTrue { src: u8, offset: i16 },
    TailCall { func_reg: u8, first_arg_reg: u8, arg_count: u8 },

    // ── Super-instruction fusions (slot-based, not stack-based) ───────
    IntLeJumpIfFalse(u32),    // local_slot <= imm ? fallthrough : jump
    IntLtJumpIfFalse(u32),    // local_slot <  imm ? fallthrough : jump
    IntIncrSlot { slot: u32 },// local_slot += 1
    IntSubCall1(u32),         // local -= imm; call (recursive)
    IntAddCall1(u32),         // local += imm; call (recursive)
    IntSubLocalI { dst: u8, payload_idx: u16 }, // dst = local[slot] - imm  (LoadLocal+IntSubI fusion)
    IntAddLocalI { dst: u8, payload_idx: u16 }, // dst = local[slot] + imm  (LoadLocal+IntAddI fusion)

    // ── Scope / misc ──────────────────────────────────────────────────

    // ── ADT / Pattern Matching ────────────────────────────────────────
    EnumDecl(u32),
    MatchVariant(u32),
    BindVar(SymId),

    // ── Loop / Exception / Concurrency ────────────────────────────────
    Break, Continue,
    ForIn { iter_reg: u8, var_sym_idx: u16, end_offset: i16 },
    IterNext { iter_reg: u8, var_sym_idx: u16, end_offset: i16 },
    TryBegin(i32), TryEnd, Throw { src: u8 },
    FinallyBegin(i32), FinallyEnd, FinallyExit(i32),
    LoopBegin(u32), LoopEnd,
    Spawn { payload_idx: u16, first_arg: u8, arg_count: u8 }, Despawn { reg: u8 }, ViewAs { obj: u8, view_sym: u16 },
    Send { message: u8, target: u8 }, Receive { var_sym_idx: u16, src: u8 }, Require { src: u8 }, Perform { src: u8 },
    Await { src: u8, dst: u8 }, Yield { src: u8 },

    // ── Class / Module ────────────────────────────────────────────────
    ClassDecl(u32), NewInstance { payload_idx: u16, first_arg: u8, arg_count: u8 }, TraitCheck(u32),
    LoadModule(u32), DefineFunction(u32),
    MethodCall { dst: u8, obj: u8, payload_idx: u16, first_arg: u8, arg_count: u8 },
    SuperCall { dst: u8, payload_idx: u16, first_arg: u8, arg_count: u8 },
    MakeGenerator { payload_idx: u16, first_arg: u8, arg_count: u8 },
    CallSpread(SymId), MethodCallSpread(SymId),
    GetProperty { dst: u8, obj: u8, prop_sym: u16 },
    GetStatic(u32), ClassStaticDecl(u32),
    DeclStore { payload_idx: u16, src: u8 },
    WriteBackReceiver(SymId),
    DestructArray(u16, bool), DestructObject(u32),
    Remember { store_idx: u16, src: u8 }, Recall { store_idx: u16, src: u8, dst: u8 }, Forget { store_idx: u16, src: u8 },
    ArrayPush { dst: u8, arr: u8, val: u8 },
    SpreadIntoArray { dst: u8, src: u8 }, SpreadIntoObject { dst: u8, src: u8 },
    // ═══════════════════════════════════════════════════════════════════
    // REGISTER-BASED VM — THE ONLY ARITHMETIC & COMPARISON PATH
    // ═══════════════════════════════════════════════════════════════════

    // Arithmetic: dst = src1 OP src2
    IntAdd { dst: u8, src1: u8, src2: u8 },
    IntSub { dst: u8, src1: u8, src2: u8 },
    IntMul { dst: u8, src1: u8, src2: u8 },

    // Arithmetic: dst = src OP imm
    IntAddI { dst: u8, src: u8, imm: i16 },
    IntSubI { dst: u8, src: u8, imm: i16 },
    IntMulI { dst: u8, src: u8, imm: i16 },
    IntDivI { dst: u8, src: u8, imm: i16 },
    IntModI { dst: u8, src: u8, imm: i16 },

    // Comparison: dst = bool(src1 OP src2), op: 0..=5
    IntCmp { dst: u8, src1: u8, src2: u8, op: u8 },
    IntCmpI { dst: u8, src: u8, imm: i16, op: u8 },
    NumAdd { dst: u8, src1: u8, src2: u8 },
    NumAddI { dst: u8, src: u8, imm: i16 },
    NumSub { dst: u8, src1: u8, src2: u8 },
    NumSubI { dst: u8, src: u8, imm: i16 },
    NumMulI { dst: u8, src: u8, imm: i16 },
    NumDivI { dst: u8, src: u8, imm: i16 },
    NumMul { dst: u8, src1: u8, src2: u8 },

    // Number (float) arithmetic
    NumDiv { dst: u8, src1: u8, src2: u8 },
    NumMod { dst: u8, src1: u8, src2: u8 },

    // Integer division (truncation toward zero), dst = src1 / src2
    IntDiv { dst: u8, src1: u8, src2: u8 },

    // Integer modulo, dst = src1 % src2
    IntMod { dst: u8, src1: u8, src2: u8 },

    // Fused comparison + branch (register operands)
    IntLeRRJumpIfFalse { src1: u8, src2: u8, offset: i16 },
    IntLtRRJumpIfFalse { src1: u8, src2: u8, offset: i16 },
    IntLeRIJumpIfFalse { src: u8, imm: i16, offset: i16 },
    IntLtRIJumpIfFalse { src: u8, imm: i16, offset: i16 },

    // Fused arithmetic + return: IntAdd/IntSub dst into r255 and return
    IntAddReturn { src1: u8, src2: u8 },
    IntSubReturn { src1: u8, src2: u8 },

    // Constant loads into register
    LoadIntConst { dst: u8, const_idx: u16 },
    LoadConst { dst: u8, const_idx: u16 },
    LoadNumConst { dst: u8, const_idx: u16 },

    // K3-1: LoadLocal/StoreLocal removed — compiler emits Move directly.

    // Stack-to-register bridge — REMOVED, replaced by Move via register 255

    // Return from register
    Return { src: u8 },

    // ── Register-based structural ops ───────────────────────────────
    Index { dst: u8, obj: u8, idx: u8 },
    MakeArray { dst: u8, count: u16 },
    MakeObject { dst: u8, count: u16 },
    Call { dst: u8, payload_idx: u16, first_arg: u8, arg_count: u8 },

    // ── Global variable access (register-based) ─────────────────────
    LoadGlobal { dst: u8, sym: u16 },
    StoreGlobal { src: u8, sym: u16 },
    DeclGlobal { src: u8, sym: u16 },
    StoreConst { src: u8, sym: u16 },

    // String operations (register-based)
    StrCat { dst: u8, src1: u8, src2: u8 },
    /// StrCatMut — in-place string append (dst == src1 implied).
    /// ONLY emitted by compiler for self-assignment: `s = s + expr`.
    /// Caller guarantees dst is the ONLY live reference besides local_slots,
    /// and local_slots will be overwritten immediately after.
    StrCatMut { dst: u8, src2: u8 },
    StringIndexOf { dst: u8, haystack: u8, needle: u8 },
    StringContains { dst: u8, haystack: u8, needle: u8 },

    // Property access (register-based)
    SetProperty { dst: u8, obj: u8, val: u8, prop_sym: u16 },

    // Index assignment (register-based)
    IndexAssign { obj: u8, idx: u8, val: u8 },

    // Unary ops (register-based)
    Neg { dst: u8, src: u8 },
    Not { dst: u8, src: u8 },
    Move { dst: u8, src: u8 },
}

impl Instruction {
    /// Returns the highest register index referenced by this instruction.
    /// Used by the compiler to track per-function max register usage.
    pub fn max_register(&self) -> u8 {
        use Instruction::*;
        let mr = match *self {
            JumpIfFalse { src, .. } | JumpIfTrue { src, .. } => src,
            TailCall { func_reg, first_arg_reg, arg_count } => {
                func_reg.max(first_arg_reg.saturating_add(arg_count).saturating_sub(1))
            }
            ForIn { iter_reg, .. } | IterNext { iter_reg, .. } => iter_reg,
            Throw { src } | Require { src } | Perform { src } | Yield { src } => src,
            Await { src, dst } => dst.max(src),
            Spawn { first_arg, arg_count, .. } | Send { message: first_arg, target: arg_count } => {
                first_arg.max(arg_count)
            }
            Despawn { reg } => reg,
            ViewAs { obj, .. } => obj,
            Receive { src, .. } => src,
            MethodCall { dst, obj, first_arg, arg_count, .. } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(obj).max(last_arg)
            }
            NewInstance { first_arg, arg_count, .. } | MakeGenerator { first_arg, arg_count, .. } => {
                first_arg.saturating_add(arg_count).saturating_sub(1)
            }
            SuperCall { dst, first_arg, arg_count, .. } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(last_arg)
            }
            Call { dst, first_arg, arg_count, .. } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(last_arg)
            }
            GetProperty { dst, obj, .. } => dst.max(obj),
            DeclStore { src, .. } | StoreConst { src, .. } | StoreGlobal { src, .. } | DeclGlobal { src, .. } => src,
            LoadGlobal { dst, .. } | LoadNumConst { dst, .. } | LoadConst { dst, .. } | LoadIntConst { dst, .. } => dst,
            ArrayPush { dst, arr, val } | SetProperty { dst, obj: arr, val, .. } => dst.max(arr).max(val),
            SpreadIntoArray { dst, src } | SpreadIntoObject { dst, src } => dst.max(src),
            Index { dst, obj, idx } => dst.max(obj).max(idx),
            IndexAssign { obj, idx, val } => obj.max(idx).max(val),
            StrCat { dst, src1, src2 } | StringIndexOf { dst, haystack: src1, needle: src2 }
            | StringContains { dst, haystack: src1, needle: src2 } => dst.max(src1).max(src2),
            StrCatMut { dst, src2 } => dst.max(src2),
            IntAdd { dst, src1, src2 } | IntSub { dst, src1, src2 } | IntMul { dst, src1, src2 }
            | IntCmp { dst, src1, src2, .. } | NumAdd { dst, src1, src2 } | NumSub { dst, src1, src2 }
            | NumMul { dst, src1, src2 } | NumDiv { dst, src1, src2 } | NumMod { dst, src1, src2 } => {
                dst.max(src1).max(src2)
            }
            IntAddI { dst, src, .. } | IntSubI { dst, src, .. } | IntMulI { dst, src, .. } | IntDivI { dst, src, .. } | IntModI { dst, src, .. } | IntCmpI { dst, src, .. } => dst.max(src),
            NumAddI { dst, src, .. } | NumSubI { dst, src, .. } | NumMulI { dst, src, .. } | NumDivI { dst, src, .. } => dst.max(src),
            IntDiv { dst, src1, src2 } | IntMod { dst, src1, src2 } => dst.max(src1).max(src2),
            IntLeRRJumpIfFalse { src1, src2, .. } | IntLtRRJumpIfFalse { src1, src2, .. } => src1.max(src2),
            IntAddReturn { src1, src2 } | IntSubReturn { src1, src2 } => src1.max(src2).max(255),
            IntLeRIJumpIfFalse { src, .. } | IntLtRIJumpIfFalse { src, .. } => src,
            Neg { dst, src } | Not { dst, src } | Move { dst, src } => dst.max(src),
            Return { src } => src,
            // Instructions with no registers
            Jump(..) | Break | Continue | TryBegin(..) | TryEnd | FinallyBegin(..)
            | FinallyEnd | FinallyExit(..) | LoopBegin(..) | LoopEnd
            | EnumDecl(..) | MatchVariant(..) | BindVar(..) | ClassDecl(..)
            | TraitCheck(..) | LoadModule(..) | DefineFunction(..) | GetStatic(..)
            | ClassStaticDecl(..) | DestructArray(..) | DestructObject(..)
            | WriteBackReceiver(..) | CallSpread(..) | MethodCallSpread(..) => 0,
            Remember { src, .. } => src,
            Recall { dst, src, .. } => dst.max(src),
            Forget { src, .. } => src,
            // Instructions with no registers
            IntLeJumpIfFalse(..) | IntLtJumpIfFalse(..) | IntIncrSlot { .. }
            | IntSubCall1(..) | IntAddCall1(..) => 0,
            IntSubLocalI { dst, .. } | IntAddLocalI { dst, .. } => dst,
            MakeArray { dst, .. } => dst,
            MakeObject { dst, .. } => dst,
        };
        // Register 255 is reserved for bridge/return-value; it does not count
        // toward per-function max register usage.
        if mr == 255 { 0 } else { mr }
    }
}
