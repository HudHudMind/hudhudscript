//! V2-B0: Instruction impl methods extracted from instruction.rs.

use crate::instruction::Instruction;
use crate::Value16;

impl Instruction {
    /// Returns the highest register index referenced by this instruction.
    /// Used by the compiler to track per-function max register usage.
    pub fn max_register(&self) -> u8 {
        use Instruction::*;
        let mr = match *self {
            JumpIfFalse { src, .. } | JumpIfTrue { src, .. } => src,
            TailCall {
                func_reg,
                first_arg_reg,
                arg_count,
            } => func_reg.max(first_arg_reg.saturating_add(arg_count).saturating_sub(1)),
            ForIn { iter_reg, .. } | IterNext { iter_reg, .. } => iter_reg,
            Throw { src } | Require { src } | Perform { src } | Yield { src } => src,
            Await { src, dst } => dst.max(src),
            Spawn {
                first_arg,
                arg_count,
                ..
            }
            | Send {
                message: first_arg,
                target: arg_count,
            } => first_arg.max(arg_count),
            Despawn { reg } => reg,
            ViewAs { obj, .. } => obj,
            Receive { src, .. } => src,
            MethodCall {
                dst,
                obj,
                first_arg,
                arg_count,
                ..
            } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(obj).max(last_arg)
            }
            NewInstance {
                first_arg,
                arg_count,
                ..
            }
            | MakeGenerator {
                first_arg,
                arg_count,
                ..
            } => first_arg.saturating_add(arg_count).saturating_sub(1),
            SuperCall {
                dst,
                first_arg,
                arg_count,
                ..
            } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(last_arg)
            }
            Call {
                dst,
                first_arg,
                arg_count,
                ..
            } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(last_arg)
            }
            GetProperty { dst, obj, .. } => dst.max(obj),
            DeclStore { src, .. }
            | StoreConst { src, .. }
            | StoreGlobal { src, .. }
            | DeclGlobal { src, .. } => src,
            StoreGlobalConst { .. } => 0,
            LoadGlobal { dst, .. }
            | LoadNumConst { dst, .. }
            | LoadConst { dst, .. }
            | LoadIntConst { dst, .. } => dst,
            StringConcat { regs_start, count, dst } => dst.max(regs_start.saturating_add(count).saturating_sub(1)),
            ArrayPush { dst, arr, val }
            | SetProperty {
                dst, obj: arr, val, ..
            } => dst.max(arr).max(val),
            SpreadIntoArray { dst, src } | SpreadIntoObject { dst, src } => dst.max(src),
            Index { dst, obj, idx } => dst.max(obj).max(idx),
            IndexArray { dst, obj, idx } => dst.max(obj).max(idx),
            IndexStringAscii { dst, obj, idx } => dst.max(obj).max(idx),
            IndexAssign { obj, idx, val } => obj.max(idx).max(val),
            IndexAssignArray { obj, idx, val } => obj.max(idx).max(val),
            StrCat { dst, src1, src2 }
            | StringIndexOf {
                dst,
                haystack: src1,
                needle: src2,
            }
            | StringContains {
                dst,
                haystack: src1,
                needle: src2,
            } => dst.max(src1).max(src2),
            StrCatMut { dst, src2 } => dst.max(src2),
            NumMulAddAssign { dst, mul, add } => dst.max(mul).max(add),
            NumMulAddIndexed { acc, mul, arr, idx } => acc.max(mul).max(arr).max(idx),
            FloatMulAdd { dst, mul1, mul2, add } => dst.max(mul1).max(mul2).max(add),
            FloatAdd { dst, src1, src2 } => dst.max(src1).max(src2),
            FloatMul { dst, src1, src2 } => dst.max(src1).max(src2),
            IntMulMod { dst, src1, src2, src3 } => dst.max(src1).max(src2).max(src3),
            IntMulModI { dst, src1, src2, .. } => dst.max(src1).max(src2),
            StrCharEqRR { dst, src_s, src_i, src_j } => dst.max(src_s).max(src_i).max(src_j),
            IntAdd { dst, src1, src2 }
            | IntSub { dst, src1, src2 }
            | IntMul { dst, src1, src2 }
            | IntCmp {
                dst, src1, src2, ..
            }
            | NumAdd { dst, src1, src2 }
            | NumSub { dst, src1, src2 }
            | NumMul { dst, src1, src2 }
            | NumDiv { dst, src1, src2 }
            | NumMod { dst, src1, src2 }
            | FloatAdd { dst, src1, src2 }
            | FloatMul { dst, src1, src2 } => dst.max(src1).max(src2),
            IntAddI { dst, src, .. }
            | IntSubI { dst, src, .. }
            | IntMulI { dst, src, .. }
            | IntDivI { dst, src, .. }
            | IntModI { dst, src, .. }
            | IntCmpI { dst, src, .. }
            | IntModCmpI { dst, src, .. } => dst.max(src),
            IntCmpIJumpIfFalse { src, .. } => src,
            IntCmpIJumpIfTrue { src, .. } => src,
            IntCmpRRJumpIfFalse { src1, src2, .. } => src1.max(src2),
            IntAddIJump { reg, .. } => reg,
            LoopEndIntAddIJump { reg, .. } => reg,
            IntSubIJump { reg, .. } => reg,
            ReturnConst { .. } => 0,
            NumAddI { dst, src, .. }
            | NumSubI { dst, src, .. }
            | NumMulI { dst, src, .. }
            | NumDivI { dst, src, .. } => dst.max(src),
            IntDiv { dst, src1, src2 } | IntMod { dst, src1, src2 } => dst.max(src1).max(src2),
            IntLeRRJumpIfFalse { src1, src2, .. } | IntLtRRJumpIfFalse { src1, src2, .. } => {
                src1.max(src2)
            }
            IntLtRRJumpPacked(_) | IntLeRRJumpPacked(_) => 255,
            IntAddReturn { src1, src2 } | IntSubReturn { src1, src2 } => src1.max(src2).max(255),
            IntMulReturn { src1, src2 } | IntDivReturn { src1, src2 } => src1.max(src2).max(255),
            IntCmpIReturn { src, .. } => src,
            ArrayPushIntConst { arr, .. } => arr,
            ArrayPushConst { arr, .. } => arr,
            Index2D { dst, obj, idx1, idx2 } => dst.max(obj).max(idx1).max(idx2),
            IndexAssign2D { obj, idx1, idx2, val } => obj.max(idx1).max(idx2).max(val),
            IntMulAddAssign { acc, src1, src2 } => acc.max(src1).max(src2),
            PropertySubAssign { obj, src, .. } => obj.max(src),
            StrCat3 { dst, a, b, c } => dst.max(a).max(b).max(c),
            Neg { dst, src } | Not { dst, src } | Move { dst, src } => dst.max(src),
            Return { src } => src,
            // Instructions with no registers
            Jump(..)
            | Break
            | Continue
            | TryBegin(..)
            | TryEnd
            | FinallyBegin(..)
            | FinallyEnd
            | FinallyExit(..)
            | LoopBegin(..)
            | LoopEnd
            | EnumDecl(..)
            | MatchVariant(..)
            | BindVar(..)
            | ClassDecl(..)
            | TraitCheck(..)
            | LoadModule(..)
            | DefineFunction(..)
            | GetStatic(..)
            | ClassStaticDecl(..)
            | DestructArray(..)
            | DestructObject(..)
            | WriteBackReceiver(..)
            | CallSpread(..)
            | MethodCallSpread(..) => 0,
            Remember { src, .. } => src,
            Recall { dst, src, .. } => dst.max(src),
            Forget { src, .. } => src,
            // Instructions with no registers
            IntLeJumpIfFalse(..)
            | IntLtJumpIfFalse(..)
            | IntSubCall1(..)
            | IntAddCall1(..) => 0,
            MakeArray { dst, .. } => dst,
            // P8: 2-element array
            MakeArray2 { dst, a, b } => dst.max(a).max(b),
            MakeObject { dst, .. } => dst,
            // P2: length/pop fast path
            ArrayLen { dst, obj } | StringLen { dst, obj } | ArrayPop { dst, obj } => dst.max(obj),
            // P5: sqrt intrinsic
            NumSqrt { dst, src } => dst.max(src),
        };
        // Register 255 is reserved for bridge/return-value; it does not count
        // toward per-function max register usage.
        if mr == 255 {
            0
        } else {
            mr
        }
    }
}
