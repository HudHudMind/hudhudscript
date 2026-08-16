#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_super_instructions(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;

        match instr {
            Instruction::IntLeJumpIfFalse(super_idx) => {
                let sp = bytecode.get_super_instr_payload(*super_idx);
                let slot_idx = sp.slot as usize;
                let le = match numeric_slot(Some(&self.registers[slot_idx])) {
                    Some(NumericSlot::Int(a)) => a <= sp.imm as i64,
                    Some(NumericSlot::Num(a)) => num_le(a, sp.imm as f64),
                    None => {
                        return Err(Self::runtime_error_with_pos(
                            "IntLeRRJumpIfFalse: expected Int or Number local",
                            bytecode,
                            ip,
                        ))
                    }
                };
                if !le {
                    let new_ip = (ip as i64).wrapping_add(sp.offset as i64);
                    if new_ip < 0 || new_ip > instructions.len() as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "IntLeRRJumpIfFalse out of bounds: ip={} offset={}",
                            ip, sp.offset
                        )));
                    }
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntLtJumpIfFalse(super_idx) => {
                let sp = bytecode.get_super_instr_payload(*super_idx);
                let slot_idx = sp.slot as usize;
                let lt = match numeric_slot(Some(&self.registers[slot_idx])) {
                    Some(NumericSlot::Int(a)) => a < sp.imm as i64,
                    Some(NumericSlot::Num(a)) => num_lt(a, sp.imm as f64),
                    None => {
                        return Err(Self::runtime_error_with_pos(
                            "IntLtRRJumpIfFalse: expected Int or Number local",
                            bytecode,
                            ip,
                        ))
                    }
                };
                if !lt {
                    let new_ip = (ip as i64).wrapping_add(sp.offset as i64);
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntSubCall1(super_idx) => {
                let sp = bytecode.get_super_instr_payload(*super_idx);
                let slot_idx = sp.slot as usize;
                let arg_reg = sp.arg_reg as usize;
                match numeric_slot(Some(&self.registers[slot_idx])) {
                    Some(NumericSlot::Int(a)) => {
                        self.registers[arg_reg] = match a.checked_sub(sp.imm as i64) {
                            Some(r) => Value16::int(r),
                            None => {
                                #[cfg(feature = "telemetry")]
                                {
                                    self.telemetry.bigint_promotion += 1;
                                    self.telemetry.bigint_alloc += 1;
                                }
                                let big = Value16::int(a);
                                let imm_v = Value16::int(sp.imm as i64);
                                crate::vm::bigint_arith::int_sub(big, imm_v)
                                    .unwrap_or(Value16::int(0))
                            }
                        };
                    }
                    Some(NumericSlot::Num(a)) => {
                        self.registers[arg_reg] = Value16::number(a - sp.imm as f64);
                    }
                    None => {
                        let val = self.registers[slot_idx];
                        self.registers[arg_reg] =
                            crate::vm::bigint_arith::int_sub(val, Value16::int(sp.imm as i64))
                                .unwrap_or(Value16::int(0));
                    }
                }
                let payload = bytecode.get_call_payload(sp.call_idx);
                return Ok(StepAction::Call {
                    func_sym: payload.sym,
                    function_idx: payload.function_idx,
                    arg_count: payload.arg_count,
                    first_arg: sp.arg_reg,
                    dst: sp.call_dst as u8,
                    ip,
                });
            }
            Instruction::IntAddCall1(super_idx) => {
                let sp = bytecode.get_super_instr_payload(*super_idx);
                let slot_idx = sp.slot as usize;
                let arg_reg = sp.arg_reg as usize;
                match numeric_slot(Some(&self.registers[slot_idx])) {
                    Some(NumericSlot::Int(a)) => {
                        self.registers[arg_reg] = match a.checked_add(sp.imm as i64) {
                            Some(r) => Value16::int(r),
                            None => {
                                #[cfg(feature = "telemetry")]
                                {
                                    self.telemetry.bigint_promotion += 1;
                                    self.telemetry.bigint_alloc += 1;
                                }
                                let big = Value16::int(a);
                                let imm_v = Value16::int(sp.imm as i64);
                                crate::vm::bigint_arith::int_add(big, imm_v)
                                    .unwrap_or(Value16::int(0))
                            }
                        };
                    }
                    Some(NumericSlot::Num(a)) => {
                        self.registers[arg_reg] = Value16::number(a + sp.imm as f64);
                    }
                    None => {
                        let val = self.registers[slot_idx];
                        self.registers[arg_reg] =
                            crate::vm::bigint_arith::int_add(val, Value16::int(sp.imm as i64))
                                .unwrap_or(Value16::int(0));
                    }
                }
                let payload = bytecode.get_call_payload(sp.call_idx);
                return Ok(StepAction::Call {
                    func_sym: payload.sym,
                    function_idx: payload.function_idx,
                    arg_count: payload.arg_count,
                    first_arg: sp.arg_reg,
                    dst: sp.call_dst as u8,
                    ip,
                });
            }
            Instruction::SpreadIntoArray { dst, src } => {
                let acc = self.registers[*dst as usize];
                let spread = self.registers[*src as usize];
                let mut arr = acc.as_array().cloned().unwrap_or_default();
                if let Some(items) = spread.as_array() {
                    arr.extend(items.iter().cloned());
                }
                self.registers[*dst as usize] = Value16::array(arr);
            }
            Instruction::SpreadIntoObject { dst, src } => {
                let acc = self.registers[*dst as usize];
                let spread = self.registers[*src as usize];
                let mut map = acc.as_object().cloned().unwrap_or_default();
                if let Some(src_map) = spread.as_object() {
                    for (k, v) in src_map {
                        map.insert(k.clone(), v.clone());
                    }
                }
                self.registers[*dst as usize] = Value16::object(map);
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
