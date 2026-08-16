#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_collections_calls(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;

        match instr {
            // IndexFast removed — compiler now emits Index (register-based)
            Instruction::Jump(offset) => {
                // Relative jump: target = ip + offset (signed add).
                // Audit v3 Finding 4.2.
                let new_ip = (ip as i64).wrapping_add(*offset as i64);
                if new_ip < 0 || new_ip > instructions.len() as i64 {
                    return Err(compile_codes::runtime_error(format!(
                        "Jump out of bounds: ip={} offset={} → {}",
                        ip, offset, new_ip
                    )));
                }
                *ip_ref = new_ip as usize;
                return Ok(StepAction::Jumped);
            }
            Instruction::JumpIfFalse { src, offset } => {
                let cond = self.registers[*src as usize];
                if !cond.is_truthy() {
                    let new_ip = (ip as i64).wrapping_add(*offset as i64);
                    if new_ip < 0 || new_ip > instructions.len() as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "JumpIfFalse out of bounds: ip={} offset={} → {}",
                            ip, offset, new_ip
                        )));
                    }
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::JumpIfTrue { src, offset } => {
                let cond = self.registers[*src as usize];
                if cond.is_truthy() {
                    let new_ip = (ip as i64).wrapping_add(*offset as i64);
                    if new_ip < 0 || new_ip > instructions.len() as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "JumpIfTrue out of bounds: ip={} offset={} → {}",
                            ip, offset, new_ip
                        )));
                    }
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
            }

            // Call removed — replaced by Call. exec_call still used by Call/TailCall/CallSpread
            Instruction::TailCall {
                func_reg: _,
                first_arg_reg,
                arg_count,
            } => {
                let n = *arg_count as usize;
                self.args_scratch.clear();
                self.args_scratch
                    .extend((0..n).map(|i| self.registers[*first_arg_reg as usize + i]));
                let args = std::mem::take(&mut self.args_scratch);
                self.tco_args = Some(args);
                return Ok(StepAction::TailCall);
            }

            // Gap 1 — call-site spread + method-call spread.
            // CallSpread: trampoline-aware — set pending_call, return StepAction::Call.
            Instruction::CallSpread(name_sym) => {
                let args_val = self.registers[255];
                let args_arr = match args_val.as_array() {
                    Some(a) => a,
                    None => {
                        return Err(compile_codes::runtime_error(format!(
                            "CallSpread expects Array on stack, got {}",
                            Self::bytecode_value_type_name(&args_val)
                        )))
                    }
                };
                if args_arr.len() > u8::MAX as usize {
                    return Err(compile_codes::runtime_error(format!(
                        "spread call overflows u8 arg count: {}",
                        args_arr.len()
                    )));
                }
                let argc = args_arr.len() as u8;
                let first_arg = 1u8;
                for (i, v) in args_arr.iter().enumerate() {
                    self.registers[first_arg as usize + i] = v.clone();
                }
                return Ok(StepAction::Call {
                    func_sym: *name_sym,
                    function_idx: u32::MAX,
                    arg_count: argc,
                    first_arg,
                    dst: 255,
                    ip,
                });
            }
            Instruction::MethodCallSpread {
                dst,
                obj,
                args,
                method_sym,
            } => {
                self.exec_method_call_spread(*dst, *obj, *args, *method_sym, bytecode, ip)?;
                if self.pending_vm_call.is_some() {
                    return Ok(StepAction::DeferredCall);
                }
            }

            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
