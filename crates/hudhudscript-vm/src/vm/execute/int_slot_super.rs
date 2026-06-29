#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_int_slot_super(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        match instr {
            Instruction::IntAdd { .. }
            | Instruction::IntSub { .. }
            | Instruction::IntMul { .. }
            | Instruction::IntAddI { .. }
            | Instruction::IntSubI { .. }
            | Instruction::IntMulI { .. }
            | Instruction::IntDivI { .. }
            | Instruction::IntModI { .. } => self.step_int_arith(instr, ctx),

            Instruction::IntModCmpI { .. }
            | Instruction::IntCmp { .. }
            | Instruction::IntCmpI { .. }
            | Instruction::IntAddReturn { .. }
            | Instruction::IntSubReturn { .. }
            | Instruction::IntMulReturn { .. }
            | Instruction::IntDivReturn { .. }
            | Instruction::IntCmpIReturn { .. }
            | Instruction::Neg { .. }
            | Instruction::Not { .. }
            | Instruction::Move { .. } => self.step_int_cmp(instr, ctx),

            Instruction::NumAdd { .. }
            | Instruction::NumSub { .. }
            | Instruction::NumMul { .. }
            | Instruction::NumDiv { .. }
            | Instruction::NumMod { .. }
            | Instruction::NumAddI { .. }
            | Instruction::NumSubI { .. }
            | Instruction::NumMulI { .. }
            | Instruction::NumDivI { .. }
            | Instruction::NumMulAddAssign { .. }
            | Instruction::NumMulAddIndexed { .. }
            | Instruction::IntDiv { .. }
            | Instruction::IntMod { .. }
            | Instruction::IntMulMod { .. }
            | Instruction::IntMulModI { .. }
            // P5: NumSqrt intrinsic
            | Instruction::NumSqrt { .. } => self.step_num_arith(instr, ctx),

            Instruction::Jump(..)
            | Instruction::JumpIfFalse { .. }
            | Instruction::JumpIfTrue { .. }
            | Instruction::IntLeRRJumpIfFalse { .. }
            | Instruction::IntLtRRJumpIfFalse { .. }
            | Instruction::IntLtRRJumpPacked(_)
            | Instruction::IntLeRRJumpPacked(_)
            | Instruction::IntCmpIJumpIfFalse { .. }
            | Instruction::IntCmpRRJumpIfFalse { .. }
            | Instruction::IntAddIJump { .. }
            | Instruction::LoopEndIntAddIJump { .. }
            | Instruction::IntSubIJump { .. }
            | Instruction::IntCmpIJumpIfTrue { .. } => self.step_branch(instr, ctx),

            Instruction::Index { .. }
            | Instruction::IndexArray { .. }
            | Instruction::IndexStringAscii { .. }
            | Instruction::IndexAssign { .. }
            | Instruction::IndexAssignArray { .. }
            | Instruction::Index2D { .. } | Instruction::IndexAssign2D { .. } => {
                self.step_indexing(instr, ctx)
            }

            Instruction::MakeArray { .. }
            | Instruction::MakeObject { .. }
            | Instruction::ArrayPushIntConst { .. }
            | Instruction::ArrayPushConst { .. }
            | Instruction::ArrayPush { .. }
            | Instruction::IntMulAddAssign { .. }
            // P2: fast path length/pop
            | Instruction::ArrayLen { .. }
            | Instruction::StringLen { .. }
            | Instruction::ArrayPop { .. } => self.step_collections_fast(instr, ctx),

            // P8: MakeArray2
            Instruction::MakeArray2 { .. } => self.step_collections_fast(instr, ctx),

            Instruction::Call { .. }
            | Instruction::StoreConst { .. }
            | Instruction::LoadNumConst { .. }
            | Instruction::Return { .. }
            | Instruction::GetProperty { .. } => self.step_call_load(instr, ctx),

            Instruction::StrCat { .. }
            | Instruction::StrCat3 { .. }
            | Instruction::StrCatMut { .. }
            | Instruction::StringConcat { .. }
            | Instruction::StringIndexOf { .. }
            | Instruction::StringContains { .. }
            | Instruction::StrCharEqRR { .. } => self.step_string_ops(instr, ctx),

            _ => Err(Self::runtime_error_with_pos(
                "step_int_slot_super: unhandled instruction",
                ctx.bytecode,
                ctx.ip,
            )),
        }
    }
}
