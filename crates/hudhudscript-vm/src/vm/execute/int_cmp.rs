#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_int_cmp(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let _ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::IntModCmpI {
                dst,
                src,
                mod_imm,
                cmp_imm,
                op,
            } => {
                if *mod_imm == 0 {
                    return Err(Self::runtime_error_with_pos(
                        "IntModCmpI: modulo by zero",
                        bytecode,
                        ip,
                    ));
                }
                let (tag, payload) = self.registers[*src as usize].split_tag();
                let result = match tag {
                    ReprTag::Int => {
                        let m = (payload as i64) % (*mod_imm as i64);
                        let b = *cmp_imm as i64;
                        match *op {
                            0 => m < b,
                            1 => m <= b,
                            2 => m > b,
                            3 => m >= b,
                            4 => m == b,
                            5 => m != b,
                            _ => {
                                return Err(Self::runtime_error_with_pos(
                                    &format!("IntModCmpI: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    ReprTag::Number => {
                        let m = f64::from_bits(payload) % (*mod_imm as f64);
                        let b = *cmp_imm as f64;
                        match *op {
                            0 => m < b,
                            1 => m <= b,
                            2 => m > b,
                            3 => m >= b,
                            4 => m == b,
                            5 => m != b,
                            _ => {
                                return Err(Self::runtime_error_with_pos(
                                    &format!("IntModCmpI: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntModCmpI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                self.registers[*dst as usize] = Value16::bool_(result);
            }
            Instruction::IntCmpI { dst, src, imm, op } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                let result = match tag {
                    ReprTag::Int => {
                        let a = payload as i64;
                        let b = *imm as i64;
                        match *op {
                            0 => a < b,
                            1 => a <= b,
                            2 => a > b,
                            3 => a >= b,
                            4 => a == b,
                            5 => a != b,
                            _ => {
                                return Err(Self::runtime_error_with_pos(
                                    &format!("IntCmpI: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(payload);
                        let b = *imm as f64;
                        match *op {
                            0 => a < b,
                            1 => a <= b,
                            2 => a > b,
                            3 => a >= b,
                            4 => a == b,
                            5 => a != b,
                            _ => {
                                return Err(Self::runtime_error_with_pos(
                                    &format!("IntCmpI: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntCmpI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                self.registers[*dst as usize] = Value16::bool_(result);
            }
            Instruction::IntCmp {
                dst,
                src1,
                src2,
                op,
            } => {
                let v1 = &self.registers[*src1 as usize];
                let v2 = &self.registers[*src2 as usize];
                let (t1, p1) = v1.split_tag();
                let (t2, p2) = v2.split_tag();
                let result = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        let a = p1 as i64;
                        let b = p2 as i64;
                        match *op {
                            0 => a < b,
                            1 => a <= b,
                            2 => a > b,
                            3 => a >= b,
                            4 => a == b,
                            5 => a != b,
                            _ => {
                                return Err(Self::runtime_error_with_pos(
                                    &format!("IntCmp: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    (ReprTag::Number, ReprTag::Number)
                    | (ReprTag::Int, ReprTag::Number)
                    | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1 == ReprTag::Int {
                            p1 as i64 as f64
                        } else {
                            f64::from_bits(p1)
                        };
                        let b = if t2 == ReprTag::Int {
                            p2 as i64 as f64
                        } else {
                            f64::from_bits(p2)
                        };
                        match *op {
                            0 => a < b,
                            1 => a <= b,
                            2 => a > b,
                            3 => a >= b,
                            4 => a == b,
                            5 => a != b,
                            _ => {
                                return Err(Self::runtime_error_with_pos(
                                    &format!("IntCmp: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    _ => {
                        if let (Some(a), Some(b)) = (v1.as_str(), v2.as_str()) {
                            match *op {
                                0 => a < b,
                                1 => a <= b,
                                2 => a > b,
                                3 => a >= b,
                                4 => a == b,
                                5 => a != b,
                                _ => {
                                    return Err(Self::runtime_error_with_pos(
                                        &format!("IntCmp: unknown op {}", op),
                                        bytecode,
                                        ip,
                                    ))
                                }
                            }
                        } else if let (Some(a), Some(b)) = (v1.as_bool(), v2.as_bool()) {
                            match *op {
                                4 => a == b,
                                5 => a != b,
                                0 => !a && b,
                                1 => !a || a == b,
                                2 => a && !b,
                                3 => a || a == b,
                                _ => {
                                    return Err(Self::runtime_error_with_pos(
                                        &format!("IntCmp: unknown op {}", op),
                                        bytecode,
                                        ip,
                                    ))
                                }
                            }
                        } else if v1.is_null() || v2.is_null() {
                            let both_null = v1.is_null() && v2.is_null();
                            match *op {
                                4 => both_null,
                                5 => !both_null,
                                _ => false,
                            }
                        } else {
                            return Err(Self::runtime_error_with_pos(
                                &format!("IntCmp: incompatible types {:?} {:?}", t1, t2),
                                bytecode,
                                ip,
                            ));
                        }
                    }
                };
                self.registers[*dst as usize] = Value16::bool_(result);
            }
            Instruction::Neg { dst, src } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                let result = match tag {
                    ReprTag::Int => Value16::int(-(payload as i64)),
                    ReprTag::Number => Value16::number(-f64::from_bits(payload)),
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "Neg: expected Int or Number",
                            bytecode,
                            ip,
                        ))
                    }
                };
                self.registers[*dst as usize] = result;
            }
            Instruction::Not { dst, src } => {
                self.registers[*dst as usize] =
                    Value16::bool_(!self.registers[*src as usize].is_truthy());
            }
            Instruction::Move { dst, src } => {
                self.registers[*dst as usize] = self.registers[*src as usize];
            }
            // NumMul/NumAdd/NumSub — packed dispatch path (handles Int or Number operands via as_number_fast)
            Instruction::IntAddReturn { src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                let dst = self.frame_stack.last().map(|f| f.dst).unwrap_or(255);
                let result = crate::vm::bigint_arith::int_add(a, b).map_err(|code| {
                    Self::runtime_error_with_pos(&code.to_string(), bytecode, ip)
                })?;
                self.registers[dst as usize] = result;
                return Ok(StepAction::Return { src: dst });
            }
            Instruction::IntSubReturn { src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                let dst = self.frame_stack.last().map(|f| f.dst).unwrap_or(255);
                let result = crate::vm::bigint_arith::int_sub(a, b).map_err(|code| {
                    Self::runtime_error_with_pos(&code.to_string(), bytecode, ip)
                })?;
                self.registers[dst as usize] = result;
                return Ok(StepAction::Return { src: dst });
            }
            Instruction::IntMulReturn { src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                let dst = self.frame_stack.last().map(|f| f.dst).unwrap_or(255);
                let result = crate::vm::bigint_arith::int_mul(a, b).map_err(|code| {
                    Self::runtime_error_with_pos(&code.to_string(), bytecode, ip)
                })?;
                self.registers[dst as usize] = result;
                return Ok(StepAction::Return { src: dst });
            }
            Instruction::IntDivReturn { src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                let dst = self.frame_stack.last().map(|f| f.dst).unwrap_or(255);
                let result = crate::vm::bigint_arith::int_div(a, b).map_err(|code| {
                    Self::runtime_error_with_pos(&code.to_string(), bytecode, ip)
                })?;
                self.registers[dst as usize] = result;
                return Ok(StepAction::Return { src: dst });
            }
            Instruction::IntCmpIReturn { src, imm, op } => {
                let (tag, p) = self.registers[*src as usize].split_tag();
                let dst = self.frame_stack.last().map(|f| f.dst).unwrap_or(255);
                let result = match tag {
                    ReprTag::Int => {
                        let a = p as i64; let b = *imm as i64;
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => return Err(Self::runtime_error_with_pos(&format!("IntCmpIReturn: unknown op {}", op), bytecode, ip)) }
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(p); let b = *imm as f64;
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => return Err(Self::runtime_error_with_pos(&format!("IntCmpIReturn: unknown op {}", op), bytecode, ip)) }
                    }
                    _ => return Err(Self::runtime_error_with_pos("IntCmpIReturn: src not numeric", bytecode, ip)),
                };
                self.registers[dst as usize] = Value16::bool_(result);
                return Ok(StepAction::Return { src: dst });
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
