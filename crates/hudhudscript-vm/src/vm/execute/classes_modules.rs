#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_classes_modules(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        match instr {
            Instruction::ClassDecl { .. }
            | Instruction::TraitCheck { .. }
            | Instruction::NewInstance { .. }
            | Instruction::GetProperty { .. }
            | Instruction::SetProperty { .. }
            | Instruction::PropertySubAssign { .. } => self.step_class_ops(instr, ctx),

            Instruction::LoadModule { .. } | Instruction::DefineFunction { .. } => {
                self.step_module_ops(instr, ctx)
            }

            _ => Err(Self::runtime_error_with_pos(
                "step_classes_modules: unhandled instruction",
                ctx.bytecode,
                ctx.ip,
            )),
        }
    }
}
