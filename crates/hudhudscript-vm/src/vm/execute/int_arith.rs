#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_int_arith(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let _ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::IntAdd { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                self.registers[*dst as usize] = crate::vm::math_fast_paths::do_int_add(a, b).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
            }
            Instruction::IntSub { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                self.registers[*dst as usize] = crate::vm::math_fast_paths::do_int_sub(a, b).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
            }
            Instruction::IntMul { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                self.registers[*dst as usize] = crate::vm::math_fast_paths::do_int_mul(a, b).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
            }
            Instruction::IntAddI { dst, src, imm } => {
                let a = self.registers[*src as usize];
                self.registers[*dst as usize] = crate::vm::math_fast_paths::do_int_add_i(a, *imm as i64).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
            }
            Instruction::IntSubI { dst, src, imm } => {
                let a = self.registers[*src as usize];
                self.registers[*dst as usize] = crate::vm::math_fast_paths::do_int_sub_i(a, *imm as i64).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
            }
            Instruction::IntMulI { dst, src, imm } => {
                let a = self.registers[*src as usize];
                self.registers[*dst as usize] = crate::vm::math_fast_paths::do_int_mul_i(a, *imm as i64).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
            }
            Instruction::IntDivI { dst, src, imm } => {
                if *imm == 0 {
                    return Err(Self::runtime_error_with_pos(
                        "IntDivI: division by zero",
                        bytecode,
                        ip,
                    ));
                }
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => match (payload as i64).checked_div(*imm as i64) {
                        Some(q) => Value16::int(q),
                        None => Value16::bigint(
                            num_bigint::BigInt::from(payload as i64) / num_bigint::BigInt::from(*imm as i64),
                        ),
                    },
                    ReprTag::Number => Value16::number(f64::from_bits(payload) / (*imm as f64)),
                    ReprTag::Dynamic => {
                        let sv = self.registers[*src as usize];
                        let iv = Value16::int(*imm as i64);
                        match crate::vm::bigint_arith::bigint_div(sv, iv) {
                            Ok(val) => val,
                            Err(e) => {
                                let msg = if e.0 == 399 {
                                    "IntDivI: division by zero"
                                } else {
                                    "IntDivI: src not numeric"
                                };
                                return Err(Self::runtime_error_with_pos(msg, bytecode, ip))
                            }
                        }
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntDivI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
            }
            Instruction::IntModI { dst, src, imm } => {
                if *imm == 0 {
                    return Err(Self::runtime_error_with_pos(
                        "IntModI: modulo by zero",
                        bytecode,
                        ip,
                    ));
                }
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => match (payload as i64).checked_rem(*imm as i64) {
                        Some(r) => Value16::int(r),
                        None => Value16::int(0),
                    },
                    ReprTag::Number => Value16::number(f64::from_bits(payload) % (*imm as f64)),
                    ReprTag::Dynamic => {
                        let sv = self.registers[*src as usize];
                        let iv = Value16::int(*imm as i64);
                        match crate::vm::bigint_arith::bigint_mod(sv, iv) {
                            Ok(val) => val,
                            Err(e) => {
                                let msg = if e.0 == 399 {
                                    "IntModI: modulo by zero"
                                } else {
                                    "IntModI: src not numeric"
                                };
                                return Err(Self::runtime_error_with_pos(msg, bytecode, ip))
                            }
                        }
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntModI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
