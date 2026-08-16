#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_actors_decl(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        match instr {
            Instruction::DeclStore { .. } => self.step_actor_core(instr, ctx),
            Instruction::Spawn { .. } | Instruction::Despawn { .. } => {
                self.step_actor_spawn(instr, ctx)
            }
            Instruction::ViewAs { .. } => self.step_view_as(instr, ctx),
            Instruction::Send { .. } | Instruction::Receive { .. } => {
                self.step_actor_messaging(instr, ctx)
            }
            Instruction::Require { .. } | Instruction::Perform { .. } => {
                self.step_actor_misc(instr, ctx)
            }
            _ => Err(Self::runtime_error_with_pos(
                "step_actors_decl: unhandled instruction",
                ctx.bytecode,
                ctx.ip,
            )),
        }
    }
}
