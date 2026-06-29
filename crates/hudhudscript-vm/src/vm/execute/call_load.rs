#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_call_load(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        match instr {
            Instruction::Call {
                dst,
                payload_idx,
                first_arg,
                arg_count: _,
            } => {
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                return Ok(StepAction::Call {
                    func_sym: payload.sym,
                    function_idx: payload.function_idx,
                    arg_count: payload.arg_count,
                    first_arg: *first_arg,
                    dst: *dst,
                    ip,
                });
            }
            Instruction::LoadConst { dst, const_idx } => {
                self.registers[*dst as usize] = ctx.constants[*const_idx as usize];
            }
            Instruction::LoadIntConst { dst, const_idx } => {
                self.registers[*dst as usize] =
                    Value16::int(bytecode.int_constants[*const_idx as usize]);
            }
            Instruction::LoadNumConst { dst, const_idx } => {
                let bits = bytecode.numeric_constants[*const_idx as usize];
                self.registers[*dst as usize] = Value16::number(f64::from_bits(bits));
            }
            Instruction::Return { src } => {
                return Ok(StepAction::Return { src: *src });
            }
            Instruction::GetProperty { dst, obj, prop_sym } => {
                let obj_v = self.registers[*obj as usize];
                let val = self.get_property_ic(
                    obj_v,
                    &hudhudscript_bytecode::SymId(*prop_sym as u32),
                    ip,
                )?;
                self.registers[*dst as usize] = val;
            }
            Instruction::StoreConst { src, sym } => {
                let sym_key = hudhudscript_bytecode::interner::SymbolId(*sym as u32);
                let name = bytecode.resolve_symbol(*sym as u32);
                let value = self.registers[*src as usize];
                self.set_var(&name, value)?;
                self.immutables.insert(sym_key);
                if let Some(entry) = self.call_cache.get_mut(sym_key.0 as usize) {
                    *entry = None;
                }
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
