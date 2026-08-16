#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_num_arith(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let _ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::NumAddI { dst, src, imm } => {
                let src_val = &self.registers[*src as usize];
                if src_val.split_tag().0 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*dst as usize] =
                    Value16::number(src_val.as_number_fast().unwrap_or(0.0) + (*imm as f64));
            }
            Instruction::NumSubI { dst, src, imm } => {
                let src_val = &self.registers[*src as usize];
                if src_val.split_tag().0 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*dst as usize] =
                    Value16::number(src_val.as_number_fast().unwrap_or(0.0) - (*imm as f64));
            }
            Instruction::NumMulI { dst, src, imm } => {
                let src_val = &self.registers[*src as usize];
                if src_val.split_tag().0 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*dst as usize] =
                    Value16::number(src_val.as_number_fast().unwrap_or(0.0) * (*imm as f64));
            }
            Instruction::NumDivI { dst, src, imm } => {
                let src_val = &self.registers[*src as usize];
                if src_val.split_tag().0 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
                let divisor = *imm as f64;
                if divisor == 0.0 {
                    return Err(Self::runtime_error_with_pos(
                        "Division by zero",
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*dst as usize] =
                    Value16::number(src_val.as_number_fast().unwrap_or(0.0) / divisor);
            }
            Instruction::NumDiv { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
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
                if b == 0.0 {
                    return Err(Self::runtime_error_with_pos(
                        "Division by zero",
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*dst as usize] = Value16::number(a / b);
            }
            Instruction::IntDiv { dst, src1, src2 } => {
                let a_val = self.registers[*src1 as usize];
                let b_val = self.registers[*src2 as usize];
                // Fast path: pure Int operands
                if a_val.is_int() && b_val.is_int() {
                    let a = a_val.as_int_unchecked();
                    let b = b_val.as_int_unchecked();
                    if b == 0 {
                        return Err(Self::runtime_error_with_pos(
                            "Division by zero",
                            bytecode,
                            ip,
                        ));
                    }
                    self.registers[*dst as usize] = match a.checked_div(b) {
                        Some(q) => Value16::int(q),
                        None => Value16::bigint(
                            num_bigint::BigInt::from(a) / num_bigint::BigInt::from(b),
                        ),
                    };
                } else if (a_val.is_number() || a_val.is_int())
                    && (b_val.is_number() || b_val.is_int())
                {
                    let a = a_val.as_number_fast().unwrap_or(0.0);
                    let b = b_val.as_number_fast().unwrap_or(0.0);
                    if b == 0.0 {
                        return Err(Self::runtime_error_with_pos(
                            "Division by zero",
                            bytecode,
                            ip,
                        ));
                    }
                    self.registers[*dst as usize] = Value16::number(a / b);
                } else {
                    // Slow path: BigInt or mixed operands
                    let result =
                        crate::vm::bigint_arith::bigint_div(a_val, b_val).map_err(|e| {
                            Self::runtime_error_with_pos(
                                &format!("Division error: {:?}", e),
                                bytecode,
                                ip,
                            )
                        })?;
                    self.registers[*dst as usize] = result;
                }
            }
            Instruction::IntMod { dst, src1, src2 } => {
                let a_val = self.registers[*src1 as usize];
                let b_val = self.registers[*src2 as usize];
                if a_val.is_int() && b_val.is_int() {
                    let a = a_val.as_int_unchecked();
                    let b = b_val.as_int_unchecked();
                    if b == 0 {
                        return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                    }
                    self.registers[*dst as usize] = match a.checked_rem(b) {
                        Some(r) => Value16::int(r),
                        None => Value16::int(0),
                    };
                } else if (a_val.is_number() || a_val.is_int())
                    && (b_val.is_number() || b_val.is_int())
                {
                    let a = a_val.as_number_fast().unwrap_or(0.0);
                    let b = b_val.as_number_fast().unwrap_or(0.0);
                    if b == 0.0 {
                        return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                    }
                    self.registers[*dst as usize] = Value16::number(a % b);
                } else {
                    let result =
                        crate::vm::bigint_arith::bigint_mod(a_val, b_val).map_err(|e| {
                            Self::runtime_error_with_pos(
                                &format!("Modulo error: {:?}", e),
                                bytecode,
                                ip,
                            )
                        })?;
                    self.registers[*dst as usize] = result;
                }
            }
            Instruction::NumMod { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        let a = p1 as i64;
                        let b = p2 as i64;
                        if b == 0 {
                            return Err(Self::runtime_error_with_pos(
                                "Modulo by zero",
                                bytecode,
                                ip,
                            ));
                        }
                        match a.checked_rem(b) {
                            // P6: NumMod result must be Number (float), even for int%int.
                            Some(r) => Value16::number(r as f64),
                            None => Value16::number(0.0),
                        }
                    }
                    (ReprTag::Number, ReprTag::Int) => {
                        let a = f64::from_bits(p1);
                        let b = p2 as i64;
                        if b == 0 {
                            return Err(Self::runtime_error_with_pos(
                                "Modulo by zero",
                                bytecode,
                                ip,
                            ));
                        }
                        // P6: NumMod result is always Number, even when the float
                        // value is exactly representable as an integer.
                        Value16::number(a % (b as f64))
                    }
                    _ => {
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
                        if b == 0.0 {
                            return Err(Self::runtime_error_with_pos(
                                "Modulo by zero",
                                bytecode,
                                ip,
                            ));
                        }
                        Value16::number(if b == 1.0 { a.fract() } else { a % b })
                    }
                };
            }
            Instruction::NumMul { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
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
                self.registers[*dst as usize] = Value16::number(a * b);
            }
            Instruction::NumAdd { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
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
                self.registers[*dst as usize] = Value16::number(a + b);
            }
            // ISSUE-6: NumMulAddAssign handler — fused NumMul+NumAdd (Horner pattern).
            // Computes: registers[dst] = registers[dst] * registers[mul] + registers[add]
            Instruction::NumMulAddAssign { dst, mul, add } => {
                let (t1, p1) = self.registers[*dst as usize].split_tag();
                let (t2, p2) = self.registers[*mul as usize].split_tag();
                let (t3, p3) = self.registers[*add as usize].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic || t3 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
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
                let c = if t3 == ReprTag::Int {
                    p3 as i64 as f64
                } else {
                    f64::from_bits(p3)
                };
                self.registers[*dst as usize] = Value16::number(a * b + c);
            }
            Instruction::NumSub { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
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
                self.registers[*dst as usize] = Value16::number(a - b);
            }
            // GÖREV 3: Horner polynomial fusion — acc = acc * mul + arr[idx]
            Instruction::NumMulAddIndexed { acc, mul, arr, idx } => {
                let (t1, p1) = self.registers[*acc as usize].split_tag();
                let (t2, p2) = self.registers[*mul as usize].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
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
                let idx_val = self.registers[*idx as usize];
                let c = if let Some(arr_val) = self.registers[*arr as usize].as_array() {
                    let i = crate::vm::index_helpers::numeric_index_i64(idx_val)
                        .and_then(crate::vm::index_helpers::index_i64_to_usize)
                        .unwrap_or(0);
                    arr_val
                        .get(i)
                        .and_then(|v| v.as_number_fast())
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                self.registers[*acc as usize] = Value16::number(a * b + c);
            }
            // ── Float fast-path instructions (single split_tag, no as_number_fast per-op) ──
            // FMA uses mul_add() for hardware-accelerated (a*b)+c in one instruction.
            Instruction::FloatMulAdd {
                dst,
                mul1,
                mul2,
                add,
            } => {
                // Fast path: read all 3 operands with single split_tag call each.
                let m1v = self.registers[*mul1 as usize];
                let m1 = m1v.as_number_fast().unwrap_or(0.0);
                let m2 = self.registers[*mul2 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let a = self.registers[*add as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(m1.mul_add(m2, a));
            }
            Instruction::FloatAdd { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let b = self.registers[*src2 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a + b);
            }
            Instruction::FloatMul { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let b = self.registers[*src2 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a * b);
            }
            // P4: fused integer multiply-modulo — keeps modular arithmetic
            // in the Int fast path without intermediate Number widening.
            Instruction::IntMulMod {
                dst,
                src1,
                src2,
                src3,
            } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                let m = self.registers[*src3 as usize].as_int_fast().unwrap_or(0);
                if m == 0 {
                    return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                }
                let prod = crate::vm::bigint_arith::int_mul(a, b).map_err(|_| {
                    Self::runtime_error_with_pos("IntMulMod: invalid operands", bytecode, ip)
                })?;
                #[cfg(feature = "telemetry")]
                self.record_bigint_promotion(a, b, prod);
                self.registers[*dst as usize] =
                    crate::vm::bigint_arith::int_mod(prod, self.registers[*src3 as usize])
                        .map_err(|_| {
                            Self::runtime_error_with_pos("IntMulMod: modulo failed", bytecode, ip)
                        })?;
            }
            Instruction::IntMulModI {
                dst,
                src1,
                src2,
                imm,
            } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                let m = *imm as i64;
                if m == 0 {
                    return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                }
                let prod = crate::vm::bigint_arith::int_mul(a, b).map_err(|_| {
                    Self::runtime_error_with_pos("IntMulModI: invalid operands", bytecode, ip)
                })?;
                #[cfg(feature = "telemetry")]
                self.record_bigint_promotion(a, b, prod);
                let m_val = Value16::int(m);
                self.registers[*dst as usize] = crate::vm::bigint_arith::int_mod(prod, m_val)
                    .map_err(|_| {
                        Self::runtime_error_with_pos("IntMulModI: modulo failed", bytecode, ip)
                    })?;
            }
            // P5: NumSqrt intrinsic
            // G8: sin/cos intrinsics — NumSqrt deseninin eşleri.
            Instruction::NumSin { dst, src } => {
                let val = self.registers[*src as usize];
                let result = if let Some(n) = val.as_number_fast() {
                    Value16::number(n.sin())
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "NumSin: src not numeric",
                        bytecode,
                        ip,
                    ));
                };
                self.registers[*dst as usize] = result;
            }
            Instruction::NumCos { dst, src } => {
                let val = self.registers[*src as usize];
                let result = if let Some(n) = val.as_number_fast() {
                    Value16::number(n.cos())
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "NumCos: src not numeric",
                        bytecode,
                        ip,
                    ));
                };
                self.registers[*dst as usize] = result;
            }
            // G12: unpacked yol (packed her zaman sığar; bütünlük için).
            Instruction::FLoadNum { fslot, src } => {
                let v = self.registers[*src as usize];
                let n = if let Some(n) = v.as_number_fast() {
                    n
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "FLoadNum: src not numeric",
                        ctx.bytecode,
                        ctx.ip,
                    ));
                };
                self.f_slots[*fslot as usize] = n;
            }
            Instruction::FStoreNum { dst, fslot } => {
                self.registers[*dst as usize] = Value16::number(self.f_slots[*fslot as usize]);
            }
            Instruction::FAdd { d, a, b } => {
                self.f_slots[*d as usize] = self.f_slots[*a as usize] + self.f_slots[*b as usize];
            }
            Instruction::FSub { d, a, b } => {
                self.f_slots[*d as usize] = self.f_slots[*a as usize] - self.f_slots[*b as usize];
            }
            Instruction::FMul { d, a, b } => {
                self.f_slots[*d as usize] = self.f_slots[*a as usize] * self.f_slots[*b as usize];
            }
            Instruction::FDiv { d, a, b } => {
                self.f_slots[*d as usize] = self.f_slots[*a as usize] / self.f_slots[*b as usize];
            }
            Instruction::FSin { d, s } => {
                self.f_slots[*d as usize] = self.f_slots[*s as usize].sin();
            }
            Instruction::FCos { d, s } => {
                self.f_slots[*d as usize] = self.f_slots[*s as usize].cos();
            }
            Instruction::FSqrt { d, s } => {
                self.f_slots[*d as usize] = self.f_slots[*s as usize].sqrt();
            }
            Instruction::FConst { d, const_idx } => {
                self.f_slots[*d as usize] = ctx.bytecode.get_numeric_constant(*const_idx as usize);
            }
            Instruction::FMove { d, s } => {
                self.f_slots[*d as usize] = self.f_slots[*s as usize];
            }
            Instruction::NumSqrt { dst, src } => {
                let val = self.registers[*src as usize];
                let result = if let Some(n) = val.as_number_fast() {
                    Value16::number(n.sqrt())
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "NumSqrt: src not numeric",
                        bytecode,
                        ip,
                    ));
                };
                self.registers[*dst as usize] = result;
            }
            // E2: fused add/sub-and-return instructions.  The computed result is
            // written to the frame's destination register; the loop driver then
            // copies that register to last_return / caller dst.
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
