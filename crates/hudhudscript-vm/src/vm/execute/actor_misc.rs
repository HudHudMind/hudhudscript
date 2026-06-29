#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_actor_misc(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::Require { src, .. } => {
                let condition = self.registers[*src as usize];
                if !condition.is_truthy() {
                    // Gap 3 (interpreter parity) — match interpreter's
                    // error text so tests asserting
                    // `err.contains("require")` / `err.contains("condition")`
                    // pass.  Previously the VM emitted
                    // "Requirement not met" (no lowercase "require").
                    return Err(compile_codes::runtime_error(
                        "require condition not met".to_string(),
                    ));
                }
            }
            Instruction::Perform { .. } => {
                return Err(Self::runtime_error_with_pos(
                    "legacy Perform instruction is unsupported; compiler must emit action call",
                    bytecode,
                    ctx.ip,
                ));
            }

            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
