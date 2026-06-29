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
                self.registers[*dst as usize] = Value16::number(
                    self.registers[*src as usize]
                        .as_number_fast()
                        .unwrap_or(0.0)
                        + (*imm as f64),
                );
            }
            Instruction::NumSubI { dst, src, imm } => {
                self.registers[*dst as usize] = Value16::number(
                    self.registers[*src as usize]
                        .as_number_fast()
                        .unwrap_or(0.0)
                        - (*imm as f64),
                );
            }
            Instruction::NumMulI { dst, src, imm } => {
                self.registers[*dst as usize] = Value16::number(
                    self.registers[*src as usize]
                        .as_number_fast()
                        .unwrap_or(0.0)
                        * (*imm as f64),
                );
            }
            Instruction::NumDivI { dst, src, imm } => {
                let divisor = *imm as f64;
                if divisor == 0.0 {
                    return Err(Self::runtime_error_with_pos(
                        "Division by zero",
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*dst as usize] = Value16::number(
                    self.registers[*src as usize]
                        .as_number_fast()
                        .unwrap_or(0.0)
                        / divisor,
                );
            }
            Instruction::NumDiv { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let b = self.registers[*src2 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
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
                let a_val = &self.registers[*src1 as usize];
                let b_val = &self.registers[*src2 as usize];
                debug_assert!(
                    a_val.is_int() && b_val.is_int(),
                    "IntDiv requires Int operands"
                );
                let a = a_val.as_int_unchecked();
                let b = b_val.as_int_unchecked();
                if b == 0 {
                    return Err(Self::runtime_error_with_pos(
                        "Division by zero",
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*dst as usize] = Value16::int(a / b);
            }
            Instruction::IntMod { dst, src1, src2 } => {
                let a_val = &self.registers[*src1 as usize];
                let b_val = &self.registers[*src2 as usize];
                debug_assert!(
                    a_val.is_int() && b_val.is_int(),
                    "IntMod requires Int operands"
                );
                let a = a_val.as_int_unchecked();
                let b = b_val.as_int_unchecked();
                if b == 0 {
                    return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                }
                self.registers[*dst as usize] = Value16::int(a % b);
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
                        Value16::int(a % b)
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
                        let a_int = a as i64;
                        if a_int as f64 == a {
                            Value16::int(a_int % b)
                        } else {
                            Value16::number(a % (b as f64))
                        }
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
                let a = self.registers[*src1 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let b = self.registers[*src2 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a * b);
            }
            Instruction::NumAdd { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let b = self.registers[*src2 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a + b);
            }
            // ISSUE-6: NumMulAddAssign handler — fused NumMul+NumAdd (Horner pattern).
            // Computes: registers[dst] = registers[dst] * registers[mul] + registers[add]
            Instruction::NumMulAddAssign { dst, mul, add } => {
                let a = self.registers[*dst as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let b = self.registers[*mul as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let c = self.registers[*add as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a * b + c);
            }
            Instruction::NumSub { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let b = self.registers[*src2 as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a - b);
            }
            // GÖREV 3: Horner polynomial fusion — acc = acc * mul + arr[idx]
            Instruction::NumMulAddIndexed { acc, mul, arr, idx } => {
                let a = self.registers[*acc as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let b = self.registers[*mul as usize]
                    .as_number_fast()
                    .unwrap_or(0.0);
                let idx_val = self.registers[*idx as usize];
                let c = if let Some(arr_val) = self.registers[*arr as usize].as_array() {
                    let i = crate::vm::index_helpers::numeric_index_i64(idx_val)
                        .and_then(crate::vm::index_helpers::index_i64_to_usize)
                        .unwrap_or(0);
                    arr_val.get(i).and_then(|v| v.as_number_fast()).unwrap_or(0.0)
                } else {
                    0.0
                };
                self.registers[*acc as usize] = Value16::number(a * b + c);
            }
            // ── Float fast-path instructions (single split_tag, no as_number_fast per-op) ──
            // FMA uses mul_add() for hardware-accelerated (a*b)+c in one instruction.
            Instruction::FloatMulAdd { dst, mul1, mul2, add } => {
                // Fast path: read all 3 operands with single split_tag call each.
                let m1v = self.registers[*mul1 as usize];
                let m1 = m1v.as_number_fast().unwrap_or(0.0);
                let m2 = self.registers[*mul2 as usize].as_number_fast().unwrap_or(0.0);
                let a = self.registers[*add as usize].as_number_fast().unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(m1.mul_add(m2, a));
            }
            Instruction::FloatAdd { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize].as_number_fast().unwrap_or(0.0);
                let b = self.registers[*src2 as usize].as_number_fast().unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a + b);
            }
            Instruction::FloatMul { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize].as_number_fast().unwrap_or(0.0);
                let b = self.registers[*src2 as usize].as_number_fast().unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a * b);
            }
            // P4: fused integer multiply-modulo — keeps modular arithmetic
            // in the Int fast path without intermediate Number widening.
            Instruction::IntMulMod { dst, src1, src2, src3 } => {
                let a = self.registers[*src1 as usize].as_int_fast().unwrap_or(0);
                let b = self.registers[*src2 as usize].as_int_fast().unwrap_or(0);
                let m = self.registers[*src3 as usize].as_int_fast().unwrap_or(0);
                if m == 0 {
                    return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                }
                let prod = a.wrapping_mul(b);
                self.registers[*dst as usize] = Value16::int(prod % m);
            }
            Instruction::IntMulModI { dst, src1, src2, imm } => {
                let a = self.registers[*src1 as usize].as_int_fast().unwrap_or(0);
                let b = self.registers[*src2 as usize].as_int_fast().unwrap_or(0);
                let m = *imm as i64;
                if m == 0 {
                    return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                }
                let prod = a.wrapping_mul(b);
                self.registers[*dst as usize] = Value16::int(prod % m);
            }
            // P5: NumSqrt intrinsic
            Instruction::NumSqrt { dst, src } => {
                let val = self.registers[*src as usize];
                let result = if let Some(n) = val.as_number_fast() {
                    Value16::number(n.sqrt())
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "NumSqrt: src not numeric",
                        bytecode, ip,
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
