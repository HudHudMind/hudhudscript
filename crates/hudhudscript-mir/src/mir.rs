//! MIR instruction and function definitions (JIT_AOT_ARCHITECTURE.md §5.1-5.2).

use std::sync::Arc;

pub use crate::types::*;

/// Instructions. Payloads reference values created earlier (linear SSA
/// within a block; block parameters arrive with phi support).
#[derive(Debug, Clone, PartialEq)]
pub enum MirInst {
    // ── arithmetic (§18: signed integer ops are CHECKED semantics) ──
    Add { dst: ValueId, ty: MirType, lhs: ValueId, rhs: ValueId },
    Sub { dst: ValueId, ty: MirType, lhs: ValueId, rhs: ValueId },
    Mul { dst: ValueId, ty: MirType, lhs: ValueId, rhs: ValueId },
    Div { dst: ValueId, ty: MirType, lhs: ValueId, rhs: ValueId },
    Rem { dst: ValueId, ty: MirType, lhs: ValueId, rhs: ValueId },
    Neg { dst: ValueId, ty: MirType, src: ValueId },
    Not { dst: ValueId, src: ValueId },

    // ── comparisons ──
    Cmp { dst: ValueId, op: CmpOp, ty: MirType, lhs: ValueId, rhs: ValueId },

    // ── constants ──
    ConstInt { dst: ValueId, ty: MirType, value: i64 },
    ConstFloat { dst: ValueId, ty: MirType, bits: u64 },
    ConstBool { dst: ValueId, value: bool },
    ConstNull { dst: ValueId },
    /// String constant — index into the function's `string_table`.
    /// Result is a `*const c_char` stored as an i64 bit pattern.
    ConstString { dst: ValueId, index: u32 },

    // ── conversions ──
    IntToFloat { dst: ValueId, src: ValueId },

    // ── string operations (via native ABI helpers, §11) ──
    /// Concatenate two strings: `hudhud_string_concat(a, b) -> *mut c_char`.
    StringConcat { dst: ValueId, lhs: ValueId, rhs: ValueId },
    /// String length in bytes: `hudhud_string_len(s) -> i64`.
    StringLen { dst: ValueId, src: ValueId },
    /// String equality: `hudhud_string_eq(a, b) -> i64 (0|1)`.
    StringEq { dst: ValueId, lhs: ValueId, rhs: ValueId },
    /// Convert i64 to string: `hudhud_int_to_string(v) -> *mut c_char`.
    IntToString { dst: ValueId, src: ValueId },
    /// Convert f64 to string: `hudhud_float_to_string(v) -> *mut c_char`.
    FloatToString { dst: ValueId, src: ValueId },
    /// String substring: `hudhud_string_substring(s, start, end) -> *mut c_char`.
    StringSubstring { dst: ValueId, s: ValueId, start: ValueId, end: ValueId },

    // ── logical operators (eager; short-circuit arrives with CFG lowering) ──
    LogicalAnd { dst: ValueId, lhs: ValueId, rhs: ValueId },
    LogicalOr { dst: ValueId, lhs: ValueId, rhs: ValueId },
    LogicalNot { dst: ValueId, src: ValueId },
    UnaryNeg { dst: ValueId, ty: MirType, src: ValueId },

    // ── array operations (via native ABI helpers, §11) ──
    ArrayNew { dst: ValueId, capacity: ValueId },
    ArrayPush { arr: ValueId, value: ValueId },
    ArrayGet { dst: ValueId, ty: MirType, arr: ValueId, index: ValueId },
    ArraySet { arr: ValueId, index: ValueId, value: ValueId },
    ArrayPop { dst: ValueId, arr: ValueId },
    StringCharAt { dst: ValueId, s: ValueId, index: ValueId },
    ArrayLen { dst: ValueId, arr: ValueId },
    ArrayJoin { dst: ValueId, arr: ValueId, sep: ValueId },

    // ── object operations (via native ABI helpers; key = string handle) ──
    ObjectNew { dst: ValueId },
    ObjectSet { obj: ValueId, key: ValueId, value: ValueId },
    ObjectGet { dst: ValueId, ty: MirType, obj: ValueId, key: ValueId },
    ObjectHas { dst: ValueId, obj: ValueId, key: ValueId },
    ObjectLen { dst: ValueId, obj: ValueId },

    // ── locals ──
    Load { dst: ValueId, ty: MirType, local: LocalId },
    Param { dst: ValueId, ty: MirType, index: u32 },
    Store { local: LocalId, src: ValueId },

    // ── calls ──
    CallStatic { dst: ValueId, ty: MirType, callee: FunctionId, args: Vec<ValueId> },
    CallNative { dst: ValueId, ty: MirType, helper: RuntimeHelperId, args: Vec<ValueId> },

    // ── runtime/GC (§17) ──
    GcSafepoint,
    Trap { kind: TrapKind },
    Unreachable,
}

impl MirInst {
    /// Type of the value produced by this instruction (`None` for
    /// side-effecting ops that define no value).
    pub fn result_ty(&self) -> Option<MirType> {
        Some(match self {
            MirInst::Add { ty, .. }
            | MirInst::Sub { ty, .. }
            | MirInst::Mul { ty, .. }
            | MirInst::Div { ty, .. }
            | MirInst::Rem { ty, .. }
            | MirInst::Neg { ty, .. }
            | MirInst::Load { ty, .. }
            | MirInst::CallStatic { ty, .. }
            | MirInst::CallNative { ty, .. }
            | MirInst::ConstInt { ty, .. }
            | MirInst::ConstFloat { ty, .. }
            | MirInst::Param { ty, .. } => *ty,
            MirInst::Not { .. } | MirInst::Cmp { .. } | MirInst::ConstBool { .. } => MirType::Bool,
            MirInst::ConstNull { .. } => MirType::Generic,
            MirInst::ConstString { .. } => MirType::Ref(RefKind::String),
            MirInst::StringConcat { .. } => MirType::Ref(RefKind::String),
            MirInst::IntToString { .. } => MirType::Ref(RefKind::String),
            MirInst::FloatToString { .. } => MirType::Ref(RefKind::String),
            MirInst::StringSubstring { .. } => MirType::Ref(RefKind::String),
            MirInst::StringLen { .. } => MirType::I64,
            MirInst::StringEq { .. } => MirType::Bool,
            MirInst::IntToFloat { .. } => MirType::F64,
            MirInst::LogicalAnd { .. } | MirInst::LogicalOr { .. }
            | MirInst::LogicalNot { .. } => MirType::Bool,
            MirInst::UnaryNeg { ty, .. } => *ty,
            MirInst::ArrayNew { .. } => MirType::Generic,
            MirInst::ArrayGet { ty, .. } => *ty,
            MirInst::ArrayLen { .. } => MirType::I64,
            MirInst::ArrayJoin { .. } => MirType::Ref(RefKind::String),
            MirInst::ArrayPush { .. } | MirInst::ArraySet { .. }
            | MirInst::ObjectSet { .. } => return None,
            MirInst::ArrayPop { .. } => MirType::I64,
            MirInst::StringCharAt { .. } => MirType::Ref(RefKind::String),
            MirInst::ObjectNew { .. } => MirType::Generic,
            MirInst::ObjectGet { ty, .. } => *ty,
            MirInst::ObjectHas { .. } => MirType::Bool,
            MirInst::ObjectLen { .. } => MirType::I64,
            MirInst::Store { .. }
            | MirInst::GcSafepoint
            | MirInst::Trap { .. }
            | MirInst::Unreachable => return None,
        })
    }

    /// The value defined by this instruction, if any.
    pub fn result_value(&self) -> Option<ValueId> {
        Some(match self {
            MirInst::Add { dst, .. }
            | MirInst::Sub { dst, .. }
            | MirInst::Mul { dst, .. }
            | MirInst::Div { dst, .. }
            | MirInst::Rem { dst, .. }
            | MirInst::Neg { dst, .. }
            | MirInst::Not { dst, .. }
            | MirInst::Cmp { dst, .. }
            | MirInst::ConstInt { dst, .. }
            | MirInst::ConstFloat { dst, .. }
            | MirInst::ConstBool { dst, .. }
            | MirInst::ConstNull { dst, .. }
            | MirInst::Param { dst, .. }
            | MirInst::Load { dst, .. }
            | MirInst::CallStatic { dst, .. }
            | MirInst::CallNative { dst, .. }
            | MirInst::ConstString { dst, .. }
            | MirInst::StringConcat { dst, .. }
            | MirInst::StringLen { dst, .. }
            | MirInst::StringEq { dst, .. }
            | MirInst::IntToString { dst, .. }
            | MirInst::FloatToString { dst, .. }
            | MirInst::StringSubstring { dst, .. }
            | MirInst::IntToFloat { dst, .. }
            | MirInst::LogicalAnd { dst, .. }
            | MirInst::LogicalOr { dst, .. }
            | MirInst::LogicalNot { dst, .. }
            | MirInst::UnaryNeg { dst, .. }
            | MirInst::ArrayNew { dst, .. }
            | MirInst::ArrayGet { dst, .. }
            | MirInst::ArrayLen { dst, .. }
            | MirInst::ArrayJoin { dst, .. } => *dst,
            MirInst::ArrayPush { .. } | MirInst::ArraySet { .. }
            | MirInst::ObjectSet { .. } => return None,
            MirInst::ArrayPop { dst, .. }
            | MirInst::StringCharAt { dst, .. }
            | MirInst::ObjectNew { dst, .. }
            | MirInst::ObjectGet { dst, .. }
            | MirInst::ObjectHas { dst, .. }
            | MirInst::ObjectLen { dst, .. } => *dst,
            MirInst::Store { .. }
            | MirInst::GcSafepoint
            | MirInst::Trap { .. }
            | MirInst::Unreachable => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirTerminator {
    Return(ValueId),
    ReturnVoid,
    Branch {
        target: BlockId,
        args: Vec<ValueId>,
    },
    CondBranch {
        cond: ValueId,
        then_block: BlockId,
        then_args: Vec<ValueId>,
        else_block: BlockId,
        else_args: Vec<ValueId>,
    },
    Trap(TrapKind),
    Unreachable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirBlock {
    pub id: BlockId,
    pub params: Vec<(MirType, ValueId)>,
    pub insts: Vec<MirInst>,
    pub terminator: Option<MirTerminator>,
}

/// A whole compilation unit in MIR form.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MirModule {
    pub functions: Vec<MirFunction>,
}

impl MirModule {
    pub fn function(&self, name: &str) -> Option<&MirFunction> {
        self.functions.iter().find(|f| f.name.as_ref() == name)
    }
}

/// A fully lowered function.
#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub name: Arc<str>,
    pub param_tys: Vec<MirType>,
    pub return_ty: MirType,
    pub entry: BlockId,
    pub blocks: Vec<MirBlock>,
    pub function_names: Arc<Vec<Arc<str>>>,
    pub string_table: Arc<Vec<Arc<str>>>,
}

impl MirFunction {
    pub fn block(&self, id: BlockId) -> Option<&MirBlock> {
        self.blocks.iter().find(|b| b.id == id)
    }
}
