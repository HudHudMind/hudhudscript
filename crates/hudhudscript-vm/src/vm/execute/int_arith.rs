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
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        Value16::int((p1 as i64).wrapping_add(p2 as i64))
                    }
                    (ReprTag::Number, ReprTag::Number) => {
                        Value16::number(f64::from_bits(p1) + f64::from_bits(p2))
                    }
                    (ReprTag::Int, ReprTag::Number) => {
                        Value16::number(p1 as i64 as f64 + f64::from_bits(p2))
                    }
                    (ReprTag::Number, ReprTag::Int) => {
                        Value16::number(f64::from_bits(p1) + p2 as i64 as f64)
                    }
                    _ => {
                        let a = &self.registers[*src1 as usize];
                        let b = &self.registers[*src2 as usize];
                        if let (Some(a), Some(b)) = (a.as_string(), b.as_string()) {
                            Value16::string(a + &b)
                        } else {
                            return Err(Self::runtime_error_with_pos(
                                "IntAdd: operands not numeric/string",
                                bytecode,
                                ip,
                            ));
                        }
                    }
                };
            }
            Instruction::IntSub { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        Value16::int((p1 as i64).wrapping_sub(p2 as i64))
                    }
                    (ReprTag::Number, ReprTag::Number) => {
                        Value16::number(f64::from_bits(p1) - f64::from_bits(p2))
                    }
                    (ReprTag::Int, ReprTag::Number) => {
                        Value16::number(p1 as i64 as f64 - f64::from_bits(p2))
                    }
                    (ReprTag::Number, ReprTag::Int) => {
                        Value16::number(f64::from_bits(p1) - p2 as i64 as f64)
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntSub: operands not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
            }
            Instruction::IntMul { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        Value16::int((p1 as i64).wrapping_mul(p2 as i64))
                    }
                    (ReprTag::Number, ReprTag::Number) => {
                        Value16::number(f64::from_bits(p1) * f64::from_bits(p2))
                    }
                    (ReprTag::Int, ReprTag::Number) => {
                        Value16::number(p1 as i64 as f64 * f64::from_bits(p2))
                    }
                    (ReprTag::Number, ReprTag::Int) => {
                        Value16::number(f64::from_bits(p1) * p2 as i64 as f64)
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntMul: operands not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
            }
            Instruction::IntAddI { dst, src, imm } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_add(*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) + (*imm as f64)),
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntAddI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
            }
            Instruction::IntSubI { dst, src, imm } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_sub(*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) - (*imm as f64)),
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntSubI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
            }
            Instruction::IntMulI { dst, src, imm } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_mul(*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) * (*imm as f64)),
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntMulI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
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
                    ReprTag::Int => Value16::int((payload as i64) / (*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) / (*imm as f64)),
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
                    ReprTag::Int => Value16::int((payload as i64) % (*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) % (*imm as f64)),
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
