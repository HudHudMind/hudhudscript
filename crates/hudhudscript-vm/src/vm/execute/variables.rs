#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_variables(
        &mut self,
        _instr: &Instruction,
        _ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        // IndexAssignFast removed — compiler emits IndexAssign.
        // Variable ops use LoadGlobal/StoreGlobal/DeclGlobal.
        Ok(StepAction::Advance)
    }
}
