//! G12: floop methods for Compiler
use super::*;

impl Compiler {
    pub(super) fn target_floop_push(
        &mut self,
        slots: Vec<(String, u8)>,
        consts: Vec<(u64, u8)>,
        temp_base: u8,
    ) {
        self.floop_stack.push(crate::compiler::FloopCtx {
            slots: slots.into_iter().collect(),
            consts: consts.into_iter().collect(),
            temp_next: temp_base,
            temp_base,
        });
    }

    pub(super) fn target_floop_pop(&mut self) {
        self.floop_stack.pop();
    }

    pub(super) fn target_floop_slot(&self, name: &str) -> Option<u8> {
        self.floop_stack.last()?.slots.get(name).copied()
    }

    pub(super) fn target_floop_const_slot(&self, bits: u64) -> Option<u8> {
        self.floop_stack.last()?.consts.get(&bits).copied()
    }

    pub(super) fn target_floop_temp(&mut self) -> Option<u8> {
        let ctx = self.floop_stack.last_mut()?;
        if ctx.temp_next >= 64 {
            return None;
        }
        let t = ctx.temp_next;
        ctx.temp_next += 1;
        Some(t)
    }

    pub(super) fn target_floop_temp_pop(&mut self) {
        if let Some(ctx) = self.floop_stack.last_mut() {
            if ctx.temp_next > ctx.temp_base {
                ctx.temp_next -= 1;
            }
        }
    }

    pub(super) fn target_floop_captured(&self, name: &str) -> bool {
        let local_captured = self
            .locals
            .iter()
            .rev()
            .find(|l| l.name == name)
            .map(|l| l.is_captured)
            .unwrap_or(false);
        local_captured
            || self
                .fn_ctx
                .as_ref()
                .map(|c| c.nested_captured.contains(name))
                .unwrap_or(false)
    }
}
