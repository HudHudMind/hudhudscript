#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_literals_locals(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let _constants = ctx.constants;
        let _bytecode = ctx.bytecode;
        let _ip = ctx.ip;

        match instr {
            // LoadNumConst/LoadIntConst removed — compiler emits
            // LoadNumConst/LoadIntConst which write to registers directly.

            // ── Local slot operations (LoadLocal/StoreLocal used instead) ──

            // Struct-2a: capture-promotion opcode (BYTECODE_VERSION v13).
            //
            // The compiler emits `PromoteLocal(slot)` in the enclosing
            // function body, one per captured slot, immediately before the
            // `LoadConst(<FunctionValue>)` that materialises a nested
            // closure.  Struct-2c will replace this arm with actual cell
            // promotion — the slot's `Value` will be hoisted to a heap-
            // allocated `Arc<RwLock<Value>>` upvalue, and subsequent
            // `LoadLocal`/`StoreLocal` on the same slot will route through
            // the cell so the nested closure sees mutations.
            //
            // In Struct-2a the arm is a deliberate no-op: the existing
            // flat `FunctionChunk::captures: Vec<String>` call-entry walk
            // still drives upvalue installation, so `PromoteLocal`
            // instructions in the stream have no runtime effect.  That
            // keeps every behaviour-test green while the bytecode layout
            // stabilises at v13.  Kural 7c (no fallback) is preserved
            // because the cell-based path is the single future source —
            // `Vec<String>` is retired in Struct-2c, not kept as an
            // alternative.
            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
