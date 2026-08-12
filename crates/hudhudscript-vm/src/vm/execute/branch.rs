#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_branch(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::JumpIfFalse { src, offset } => {
                let v = &self.registers[*src as usize];
                if !v.is_truthy() {
                    let new_ip = (ip as i64).wrapping_add(*offset as i64);
                    if new_ip < 0 || new_ip > instructions.len() as i64 {
                        return Err(Self::runtime_error_with_pos(
                            format!("JumpIfFalse out of bounds: ip={} offset={}", ip, offset),
                            bytecode,
                            ip,
                        ));
                    }
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::JumpIfTrue { src, offset } => {
                if self.registers[*src as usize].is_truthy() {
                    let new_ip = (ip as i64).wrapping_add(*offset as i64);
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
            }

            Instruction::IntLeRRJumpIfFalse { src1, src2, offset } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                let cond = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => (p1 as i64) <= (p2 as i64),
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
                        a <= b
                    }
                    _ => {
                        // Non-numeric operands are not comparable with <=.
                        // Treat as false to keep fused loop-condition comparisons
                        // from aborting when they encounter unexpected tags.
                        false
                    }
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntLtRRJumpIfFalse { src1, src2, offset } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                let cond = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => (p1 as i64) < (p2 as i64),
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
                        a < b
                    }
                    _ => {
                        // Non-numeric operands are not comparable with <.
                        // Treat as false to keep fused loop-condition comparisons
                        // from aborting when they encounter unexpected tags.
                        false
                    }
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntCmpIJumpIfFalse {
                src,
                imm,
                op,
                offset,
            } => {
                let (tag, p) = self.registers[*src as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => {
                        let a = p as i64;
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
                                    &format!("IntCmpIJumpIfFalse: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(p);
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
                                    &format!("IntCmpIJumpIfFalse: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    ReprTag::Dynamic => {
                        let v = self.registers[*src as usize];
                        if let Some(a) = v.to_bigint_value() {
                            let b = num_bigint::BigInt::from(*imm as i64);
                            match *op {
                                0 => a < b, 1 => a <= b, 2 => a > b,
                                3 => a >= b, 4 => a == b, 5 => a != b,
                                _ => return Err(Self::runtime_error_with_pos(
                                    &format!("IntCmpIJumpIfFalse: unknown op {}", op), bytecode, ip))
                            }
                        } else {
                            return Err(Self::runtime_error_with_pos(
                                "IntCmpIJumpIfFalse: src not numeric", bytecode, ip))
                        }
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntCmpIJumpIfFalse: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntCmpRRJumpIfFalse {
                src1,
                src2,
                op,
                offset,
            } => {
                // G4: karşılaştırma çekirdeği TEK yerde (cmp_rr_generic) —
                // packed D_INT_CMP_RR_JUMP_P ile birebir aynı semantik.
                let v1 = self.registers[*src1 as usize];
                let v2 = self.registers[*src2 as usize];
                let cond = cmp_rr_generic(v1, v2, *op)
                    .map_err(|m| Self::runtime_error_with_pos(m, bytecode, ip))?;
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            // G4: payload-tablolu genel cmp+branch — packed kaçırılırsa
            // (teoride hep packed'dir; prepack her zaman sığar) unpacked yol.
            Instruction::IntCmpRRJumpPacked { op, payload_idx } => {
                let p = bytecode.cmp_jump_payloads[*payload_idx as usize];
                let v1 = self.registers[p.src1 as usize];
                let v2 = self.registers[p.src2 as usize];
                let cond = cmp_rr_generic(v1, v2, *op)
                    .map_err(|m| Self::runtime_error_with_pos(m, bytecode, ip))?;
                if !cond {
                    *ip_ref = p.target as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntAddIJump { reg, imm, offset } => {
                let (tag, p) = self.registers[*reg as usize].split_tag();
                let result = match tag {
                    ReprTag::Int => {
                        match (p as i64).checked_add(*imm as i64) {
                            Some(val) => (ReprTag::Int, val as u64),
                            None => {
                                let imm_v = Value16::int(*imm as i64);
                                let big = Value16::bigint(
                                    num_bigint::BigInt::from(p as i64) + num_bigint::BigInt::from(*imm as i64));
                                self.record_bigint_promotion(Value16::int(p as i64), imm_v, big);
                                self.registers[*reg as usize] = big;
                                *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                                return Ok(StepAction::Jumped);
                            }
                        }
                    }
                    ReprTag::Number => {
                        let val = f64::from_bits(p) + (*imm as f64);
                        (ReprTag::Number, val.to_bits())
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntAddIJump: reg not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                self.registers[*reg as usize] =
                    Value16(hudhudscript_bytecode::Repr::new_inline(result.0, result.1));
                *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                return Ok(StepAction::Jumped);
            }
            Instruction::LoopEndIntAddIJump { reg, imm, offset } => {
                self.loop_headers.pop();
                let (tag, p) = self.registers[*reg as usize].split_tag();
                let result = match tag {
                    ReprTag::Int => {
                        match (p as i64).checked_add(*imm as i64) {
                            Some(val) => (ReprTag::Int, val as u64),
                            None => {
                                let imm_v = Value16::int(*imm as i64);
                                let big = Value16::bigint(
                                    num_bigint::BigInt::from(p as i64) + num_bigint::BigInt::from(*imm as i64));
                                self.record_bigint_promotion(Value16::int(p as i64), imm_v, big);
                                self.registers[*reg as usize] = big;
                                *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                                return Ok(StepAction::Jumped);
                            }
                        }
                    }
                    ReprTag::Number => {
                        let val = f64::from_bits(p) + (*imm as f64);
                        (ReprTag::Number, val.to_bits())
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "LoopEndIntAddIJump: reg not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                self.registers[*reg as usize] =
                    Value16(hudhudscript_bytecode::Repr::new_inline(result.0, result.1));
                *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                return Ok(StepAction::Jumped);
            }
            Instruction::IntSubIJump { reg, imm, offset } => {
                let (tag, p) = self.registers[*reg as usize].split_tag();
                let result = match tag {
                    ReprTag::Int => {
                        match (p as i64).checked_sub(*imm as i64) {
                            Some(val) => (ReprTag::Int, val as u64),
                            None => {
                                let imm_v = Value16::int(*imm as i64);
                                let big = Value16::bigint(
                                    num_bigint::BigInt::from(p as i64) - num_bigint::BigInt::from(*imm as i64));
                                self.record_bigint_promotion(Value16::int(p as i64), imm_v, big);
                                self.registers[*reg as usize] = big;
                                *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                                return Ok(StepAction::Jumped);
                            }
                        }
                    }
                    ReprTag::Number => {
                        let val = f64::from_bits(p) - (*imm as f64);
                        (ReprTag::Number, val.to_bits())
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntSubIJump: reg not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                self.registers[*reg as usize] =
                    Value16(hudhudscript_bytecode::Repr::new_inline(result.0, result.1));
                *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                return Ok(StepAction::Jumped);
            }
            Instruction::IntCmpIJumpIfTrue {
                src,
                imm,
                op,
                offset,
            } => {
                let (tag, p) = self.registers[*src as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => {
                        let a = p as i64;
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
                                    &format!("IntCmpIJumpIfTrue: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(p);
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
                                    &format!("IntCmpIJumpIfTrue: unknown op {}", op),
                                    bytecode,
                                    ip,
                                ))
                            }
                        }
                    }
                    ReprTag::Dynamic => {
                        let v = self.registers[*src as usize];
                        if let Some(a) = v.to_bigint_value() {
                            let b = num_bigint::BigInt::from(*imm as i64);
                            match *op {
                                0 => a < b, 1 => a <= b, 2 => a > b,
                                3 => a >= b, 4 => a == b, 5 => a != b,
                                _ => return Err(Self::runtime_error_with_pos(
                                    &format!("IntCmpIJumpIfTrue: unknown op {}", op), bytecode, ip))
                            }
                        } else {
                            return Err(Self::runtime_error_with_pos(
                                "IntCmpIJumpIfTrue: src not numeric", bytecode, ip))
                        }
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntCmpIJumpIfTrue: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                if cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::CharDispatch { src, table_idx } => {
                let table = bytecode.get_char_dispatch_table(*table_idx);
                let byte_idx = self.registers[*src as usize]
                    .as_string()
                    .and_then(|s| {
                        if s.len() == 1 {
                            Some(s.as_bytes()[0] as usize)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                let offset = table[byte_idx];
                *ip_ref = (ip as i64).wrapping_add(offset as i64).wrapping_add(1) as usize;
                return Ok(StepAction::Jumped);
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}

/// G4 — TEK cmp çekirdeği (Kural 7): `IntCmpRRJumpIfFalse` (unpacked),
/// `IntCmpRRJumpPacked` (unpacked yolu) ve packed `D_INT_CMP_RR_JUMP_P`
/// AYNI karşılaştırma semantiğini buradan alır. op: 0 `<` 1 `<=` 2 `>`
/// 3 `>=` 4 `==` 5 `!=`. Eski el-kopyası merdivenin birebir taşınmışıdır.
pub(crate) fn cmp_rr_generic(v1: Value16, v2: Value16, op: u8) -> Result<bool, &'static str> {
    let (t1, p1) = v1.split_tag();
    let (t2, p2) = v2.split_tag();
    let cond = match (t1, t2) {
        (ReprTag::Int, ReprTag::Int) => {
            let a = p1 as i64;
            let b = p2 as i64;
            match op {
                0 => a < b,
                1 => a <= b,
                2 => a > b,
                3 => a >= b,
                4 => a == b,
                5 => a != b,
                _ => return Err("cmp_rr_generic: unknown op"),
            }
        }
        (ReprTag::Number, ReprTag::Number)
        | (ReprTag::Number, ReprTag::Int)
        | (ReprTag::Int, ReprTag::Number) => {
            let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
            let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
            match op {
                0 => a < b,
                1 => a <= b,
                2 => a > b,
                3 => a >= b,
                4 => a == b,
                5 => a != b,
                _ => return Err("cmp_rr_generic: unknown op"),
            }
        }
        // G9: iki taraf da INLINE string (≤15 bayt, payload'da) ise EQ/NE
        // bit karşılaştırmasıdır — inline temsil kanoniktir (aynı içerik =
        // aynı bitler), heap deref ve strcmp tamamen atlanır. Sıralama
        // op'ları (<, <= …) bayt sırası ≠ bit sırası olduğundan genel
        // merdivene düşer.
        (ReprTag::InlineString, ReprTag::InlineString) if op == 4 || op == 5 => {
            let eq = v1.0 == v2.0;
            if op == 4 { eq } else { !eq }
        }
        _ => {
            if let (Some(a), Some(b)) = (v1.to_bigint_value(), v2.to_bigint_value()) {
                match op {
                    0 => a < b,
                    1 => a <= b,
                    2 => a > b,
                    3 => a >= b,
                    4 => a == b,
                    5 => a != b,
                    _ => return Err("cmp_rr_generic: unknown op"),
                }
            } else if let (Some(a), Some(b)) = (v1.as_str(), v2.as_str()) {
                match op {
                    0 => a < b,
                    1 => a <= b,
                    2 => a > b,
                    3 => a >= b,
                    4 => a == b,
                    5 => a != b,
                    _ => return Err("cmp_rr_generic: unknown op"),
                }
            } else if let (Some(a), Some(b)) = (v1.as_bool(), v2.as_bool()) {
                match op {
                    4 => a == b,
                    5 => a != b,
                    0 => !a && b,
                    1 => !a || a == b,
                    2 => a && !b,
                    3 => a || a == b,
                    _ => return Err("cmp_rr_generic: unknown op"),
                }
            } else if v1.is_null() || v2.is_null() {
                let both_null = v1.is_null() && v2.is_null();
                match op {
                    4 => both_null,
                    5 => !both_null,
                    _ => false,
                }
            } else {
                return Err("IntCmpRRJumpIfFalse: incompatible types");
            }
        }
    };
    Ok(cond)
}

