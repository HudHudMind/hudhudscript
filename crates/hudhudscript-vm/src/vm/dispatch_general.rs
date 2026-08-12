use super::{decode_packed, num_as_f64, num_ref_as_f64, numeric_slot};
use crate::vm::{NumericSlot, PackedResult, VM};
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::shared_value::{
    num_add, num_div, num_eq, num_ge, num_gt, num_le, num_lt, num_mod, num_mul, num_neg, num_sub,
};
use hudhudscript_bytecode::{packed_instruction, Bytecode, Instruction, ReprTag, Value16};

impl crate::vm::VM {
    #[inline(always)]
    pub(crate) fn dispatch_chunk5(
        &mut self,
        opcode: u8,
        arg1: u8,
        arg2: u16,
        _constants: &[Value16],
        instructions: &[Instruction],
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileResult<PackedResult> {
        use crate::vm::dense_ops::*;
        let dense = crate::vm::dispatch_table::DENSE_MAP[opcode as usize];
        if dense == 0xFF {
            return Ok(PackedResult::Fallthrough);
        }
        match dense {
            // ── Jumps (hot in loops / conditionals) ─────────────────
            // i32 relative offset packed as i16 (sign-extended back).
            D_JUMP => {
                let offset = (arg2 as i16) as i32;
                let target = (ip as i64).wrapping_add(offset as i64);
                debug_assert!(
                    target >= 0 && target <= instructions.len() as i64,
                    "Jump out of bounds: ip={} offset={}",
                    ip,
                    offset
                );
                Ok(PackedResult::Jump(target as usize))
            }
            D_JUMP_IF_FALSE => {
                let cond = self.registers[255];
                if !cond.is_truthy() {
                    let offset = (arg2 as i16) as i32;
                    let target = (ip as i64).wrapping_add(offset as i64);
                    if target < 0 || target > instructions.len() as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "JumpIfFalse out of bounds: ip={} offset={} → {}",
                            ip, offset, target
                        )));
                    }
                    Ok(PackedResult::Jump(target as usize))
                } else {
                    Ok(PackedResult::Advance)
                }
            }
            D_JUMP_IF_TRUE => {
                let cond = self.registers[255];
                if cond.is_truthy() {
                    let offset = (arg2 as i16) as i32;
                    let target = (ip as i64).wrapping_add(offset as i64);
                    if target < 0 || target > instructions.len() as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "JumpIfTrue out of bounds: ip={} offset={} → {}",
                            ip, offset, target
                        )));
                    }
                    Ok(PackedResult::Jump(target as usize))
                } else {
                    Ok(PackedResult::Advance)
                }
            }
            D_JUMP_IF_FALSE_R => {
                let cond = self.registers[arg1 as usize];
                if !cond.is_truthy() {
                    let offset = (arg2 as i16) as i32;
                    let target = (ip as i64).wrapping_add(offset as i64);
                    if target < 0 || target > instructions.len() as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "JumpIfFalse out of bounds: ip={} offset={} → {}",
                            ip, offset, target
                        )));
                    }
                    Ok(PackedResult::Jump(target as usize))
                } else {
                    Ok(PackedResult::Advance)
                }
            }
            D_JUMP_IF_TRUE_R => {
                let cond = self.registers[arg1 as usize];
                if cond.is_truthy() {
                    let offset = (arg2 as i16) as i32;
                    let target = (ip as i64).wrapping_add(offset as i64);
                    if target < 0 || target > instructions.len() as i64 {
                        return Err(compile_codes::runtime_error(format!(
                            "JumpIfTrue out of bounds: ip={} offset={} → {}",
                            ip, offset, target
                        )));
                    }
                    Ok(PackedResult::Jump(target as usize))
                } else {
                    Ok(PackedResult::Advance)
                }
            }

            // IndexFast + IntIncrSlot — removed, no longer generated

            // ── PERF-36 constant loads on fast dispatch ─────────────
            D_LOAD_NUM_CONST => {
                let idx = arg2 as usize;
                let n = bytecode.get_numeric_constant(idx);
                self.registers[255] = Value16::number(n);
                Ok(PackedResult::Advance)
            }
            D_LOAD_INT_CONST => {
                let idx = arg2 as usize;
                let v = bytecode.get_int_constant(idx);
                self.registers[255] = Value16::int(v);
                Ok(PackedResult::Advance)
            }

            D_INT_SUB_LOCAL_I | D_INT_ADD_LOCAL_I => {
                let dst = arg1 as usize;
                let payload = bytecode.get_super_instr_payload(arg2 as u32);
                let slot_idx = payload.slot as usize;
                let (tag, p) = self.registers[slot_idx].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => {
                        let a = p as i64;
                        let imm = payload.imm as i64;
                        let r = if dense == D_INT_SUB_LOCAL_I {
                            a.checked_sub(imm)
                        } else {
                            a.checked_add(imm)
                        };
                        match r {
                            Some(v) => Value16::int(v),
                            None => {
                                // G3.1: overflow → BigInt via bigint_arith
                                if dense == D_INT_SUB_LOCAL_I {
                                    crate::vm::bigint_arith::int_sub(Value16::int(a), Value16::int(imm))
                                        .unwrap_or_else(|_| Value16::null())
                                } else {
                                    crate::vm::bigint_arith::int_add(Value16::int(a), Value16::int(imm))
                                        .unwrap_or_else(|_| Value16::null())
                                }
                            }
                        }
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(p);
                        let r = if dense == D_INT_SUB_LOCAL_I {
                            a - payload.imm as f64
                        } else {
                            a + payload.imm as f64
                        };
                        Value16::number(r)
                    }
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntSubLocalI/IntAddLocalI: expected numeric local",
                            bytecode,
                            ip,
                        ))
                    }
                };
                Ok(PackedResult::Advance)
            }
            // A3b: explicit Int → Number widening on the fast-dispatch
            // path.  Compiler emits this only where a downstream
            // consumer strictly needs `Value::Number`; runtime widens
            // the top-of-stack.  Non-Int operand is a compiler
            // invariant violation — fall through to the unpacked arm
            // ── Logic ops ───────────────────────────────────────────

            // Register-based comparison ops — packed dispatch (split_tag fast path)
            D_INT_EQ_RR | D_INT_LT_RR | D_INT_LE_RR | D_INT_NE_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = self.registers[src1].split_tag();
                let (t2, p2) = self.registers[src2].split_tag();
                let result = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        let a = p1 as i64;
                        let b = p2 as i64;
                        match dense {
                            D_INT_EQ_RR => a == b,
                            D_INT_LT_RR => a < b,
                            D_INT_LE_RR => a <= b,
                            D_INT_NE_RR => a != b,
                            _ => unreachable!(),
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
                        match dense {
                            D_INT_EQ_RR => a == b,
                            D_INT_LT_RR => a < b,
                            D_INT_LE_RR => a <= b,
                            D_INT_NE_RR => a != b,
                            _ => unreachable!(),
                        }
                    }
                    _ => {
                        // BigInt comparison
                        let v1 = self.registers[src1];
                        let v2 = self.registers[src2];
                        if let (Some(a), Some(b)) = (v1.to_bigint_value(), v2.to_bigint_value()) {
                            match dense {
                                D_INT_EQ_RR => a == b,
                                D_INT_LT_RR => a < b,
                                D_INT_LE_RR => a <= b,
                                D_INT_NE_RR => a != b,
                                _ => unreachable!(),
                            }
                        } else if let (Some(a), Some(b)) = (v1.as_str(), v2.as_str()) {
                            match dense {
                                D_INT_EQ_RR => a == b,
                                D_INT_LT_RR => a < b,
                                D_INT_LE_RR => a <= b,
                                D_INT_NE_RR => a != b,
                                _ => unreachable!(),
                            }
                        } else if let (Some(a), Some(b)) = (v1.as_bool(), v2.as_bool()) {
                            match dense {
                                D_INT_EQ_RR => a == b,
                                D_INT_NE_RR => a != b,
                                D_INT_LT_RR => !a && b,
                                D_INT_LE_RR => !a || a == b,
                                _ => unreachable!(),
                            }
                        } else if v1.is_null() || v2.is_null() {
                            let both = v1.is_null() && v2.is_null();
                            match dense {
                                D_INT_EQ_RR => both,
                                D_INT_NE_RR => !both,
                                _ => false,
                            }
                        } else {
                            // F3: Object/array equality — delegate to object_equality policy
                            let (t1, t2) = (v1.split_tag().0, v2.split_tag().0);
                            if t1 == ReprTag::Dynamic && t2 == ReprTag::Dynamic {
                                match self.object_equality {
                                    crate::vm::config_types::ObjectEquality::Identity => {
                                        let p1 = v1.split_tag().1;
                                        let p2 = v2.split_tag().1;
                                        match dense {
                                            D_INT_EQ_RR => p1 == p2,
                                            D_INT_NE_RR => p1 != p2,
                                            D_INT_LT_RR => p1 < p2,
                                            D_INT_LE_RR => p1 <= p2,
                                            _ => false,
                                        }
                                    }
                                    crate::vm::config_types::ObjectEquality::Never => {
                                        match dense {
                                            D_INT_EQ_RR => false,
                                            D_INT_NE_RR => true,
                                            D_INT_LT_RR => false,
                                            D_INT_LE_RR => false,
                                            _ => false,
                                        }
                                    }
                                    crate::vm::config_types::ObjectEquality::Deep => {
                                        let eq = v1.values_equal(&v2);
                                        match dense {
                                            D_INT_EQ_RR => eq,
                                            D_INT_NE_RR => !eq,
                                            D_INT_LT_RR => false,
                                            D_INT_LE_RR => false,
                                            _ => false,
                                        }
                                    }
                                }
                            } else {
                                // B8: non-Dynamic types — handle all comparison ops, not just EQ/NE
                                match dense {
                                    D_INT_EQ_RR => false,
                                    D_INT_NE_RR => true,
                                    D_INT_LT_RR => false,
                                    D_INT_LE_RR => false,
                                    _ => false,
                                }
                            }
                        }
                    }
                };
                self.registers[dst] = Value16::bool_(result);
                Ok(PackedResult::Advance)
            }
            D_MOVE_RR => {
                let dst = arg1 as usize;
                let src = arg2 as usize;
                self.registers[dst] = self.registers[src];
                Ok(PackedResult::Advance)
            }
            // V2-B0: Delegated to dispatch_int_arith.rs
            D_INT_ADD_RI
            | D_INT_MUL_RI
            | D_INT_SUB_RI
            | D_INT_ADD_RR
            | D_INT_SUB_RR
            | D_INT_MUL_RR
            | D_INDEX_RRR
            | D_INDEX_ARRAY_RRR
            | D_INDEX_STRING_ASCII_RRR
            | D_INDEX_ASSIGN_RRR
            | D_INT_MOD_I
            | D_INT_CMP_LT_I
            | D_INT_CMP_LE_I
            | D_INT_CMP_EQ_I
            | D_INT_CMP_NE_I
            // G4.2: connect previously-unreachable handlers
            | D_NEG_R
            | D_NOT_R
            | D_ARRAY_PUSH_RRR
            | D_STRCAT_RRR
            | D_STRCAT_MUT_RR
            | D_STRING_INDEX_OF_RRR
            | D_STRING_CONTAINS_RRR
            => {
                return crate::vm::dispatch_int_arith::dispatch_int_arithmetic(
                    self, dense, arg1, arg2, bytecode, ip,
                );
            }
            D_INT_MOD_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let a_val = self.registers[src1];
                let b_val = self.registers[src2];
                if a_val.is_int() && b_val.is_int() {
                    let a = a_val.as_int_unchecked();
                    let b = b_val.as_int_unchecked();
                    if b == 0 {
                        return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                    }
                    self.registers[dst] = Value16::int(a % b);
                } else if (a_val.is_number() || a_val.is_int()) && (b_val.is_number() || b_val.is_int()) {
                    let a = a_val.as_number_fast().unwrap_or(0.0);
                    let b = b_val.as_number_fast().unwrap_or(0.0);
                    if b == 0.0 {
                        return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip));
                    }
                    self.registers[dst] = Value16::number(a % b);
                } else {
                    let result = crate::vm::bigint_arith::bigint_mod(a_val, b_val)
                        .map_err(|e| Self::runtime_error_with_pos(&format!("Modulo error: {:?}", e), bytecode, ip))?;
                    self.registers[dst] = result;
                }
                Ok(PackedResult::Advance)
            }
            // Float register arithmetic — packed fast path
            // V2-B0: Delegated to dispatch_num_arith.rs
            D_NUM_ADD_RR | D_NUM_SUB_RR | D_NUM_MUL_RR | D_NUM_ADD_RI | D_NUM_SUB_RI
            | D_NUM_MUL_RI
            // G4.2: connect previously-unreachable num handlers
            | D_NUM_DIV_RR | D_NUM_DIV_RI | D_NUM_MUL_ADD_ASSIGN
            => {
                return crate::vm::dispatch_num_arith::dispatch_num_arithmetic(
                    self, dense, arg1, arg2, bytecode, ip,
                );
            }
            D_LOAD_INT_CONST_R => {
                let dst = arg1 as usize;
                let ci = arg2 as usize;
                self.registers[dst] = Value16::int(bytecode.int_constants[ci]);
                Ok(PackedResult::Advance)
            }
            D_STR_CAT => {
                let r = self.registers[255];
                let l = self.registers[255];
                let result = hudhudscript_bytecode::shared_value::shared_add(&l, &r)?;
                self.registers[255] = result;
                Ok(PackedResult::Advance)
            }

            D_RETURN_R => {
                // The loop driver copies registers[src] to last_return and the
                // caller's destination. Keep 255 untouched — builtin fast-return
                // paths still rely on it being set by their own handlers.
                Ok(PackedResult::Return { src: arg1 })
            }

            // R1: LE/LT JUMP super-instructions (moved from dispatch_control.rs)
            D_INT_LE_JUMP_IF_FALSE => {
                let super_idx = arg2 as u32;
                let sp = bytecode.get_super_instr_payload(super_idx);
                let (tag, p) = self.registers[sp.slot as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => (p as i64) <= sp.imm as i64,
                    ReprTag::Number => f64::from_bits(p) <= sp.imm as f64,
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntLeJumpIfFalse: expected numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                if !cond {
                    let target = (ip as i64).wrapping_add(sp.offset as i64);
                    Ok(PackedResult::Jump(target as usize))
                } else {
                    Ok(PackedResult::Advance)
                }
            }
            D_INT_LT_JUMP_IF_FALSE => {
                let super_idx = arg2 as u32;
                let sp = bytecode.get_super_instr_payload(super_idx);
                let (tag, p) = self.registers[sp.slot as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => (p as i64) < sp.imm as i64,
                    ReprTag::Number => f64::from_bits(p) < sp.imm as f64,
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            "IntLtJumpIfFalse: expected numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                if !cond {
                    let target = (ip as i64).wrapping_add(sp.offset as i64);
                    Ok(PackedResult::Jump(target as usize))
                } else {
                    Ok(PackedResult::Advance)
                }
            }

            // GÖREV 5: packed compare+jump (side-table)
            D_INT_LT_RR_JUMP_P | D_INT_LE_RR_JUMP_P => {
                let payload_idx = arg2 as u32;
                let p = bytecode.cmp_jump_payloads[payload_idx as usize];
                let (t1, p1) = self.registers[p.src1 as usize].split_tag();
                let (t2, p2) = self.registers[p.src2 as usize].split_tag();
                let cond: bool = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        let a = p1 as i64;
                        let b = p2 as i64;
                        if dense == D_INT_LT_RR_JUMP_P {
                            a < b
                        } else {
                            a <= b
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
                        if dense == D_INT_LT_RR_JUMP_P {
                            a < b
                        } else {
                            a <= b
                        }
                    }
                    _ => {
                        // BigInt comparison
                        let v1 = self.registers[p.src1 as usize];
                        let v2 = self.registers[p.src2 as usize];
                        if let (Some(a), Some(b)) = (v1.to_bigint_value(), v2.to_bigint_value()) {
                            if dense == D_INT_LT_RR_JUMP_P { a < b } else { a <= b }
                        } else {
                            return Err(Self::runtime_error_with_pos(
                                "CmpJump: operands not numeric", bytecode, ip));
                        }
                    }
                };
                if !cond {
                    Ok(PackedResult::Jump(p.target as usize))
                } else {
                    Ok(PackedResult::Advance)
                }
            }

            // G4: genel cmp+branch — op arg1'de, payload indeksi arg2'de.
            // Karşılaştırma çekirdeği TEK yerde (branch.rs::cmp_rr_generic);
            // unpacked IntCmpRRJumpIfFalse ile birebir aynı semantik.
            D_INT_CMP_RR_JUMP_P => {
                let p = bytecode.cmp_jump_payloads[arg2 as usize];
                let v1 = self.registers[p.src1 as usize];
                let v2 = self.registers[p.src2 as usize];
                let cond = crate::vm::execute::branch::cmp_rr_generic(v1, v2, arg1)
                    .map_err(|m| Self::runtime_error_with_pos(m, bytecode, ip))?;
                if !cond {
                    Ok(PackedResult::Jump(p.target as usize))
                } else {
                    Ok(PackedResult::Advance)
                }
            }

            // G12: unboxed float — tagsız f64 slot aritmetiği.
            D_F_LOAD_NUM => {
                let v = self.registers[arg2 as usize];
                let n = if let Some(n) = v.as_number_fast() { n } else {
                    return Err(Self::runtime_error_with_pos(
                        "FLoadNum: src not numeric (compiler bug — tip kanıtsız emit)",
                        bytecode, ip));
                };
                self.f_slots[arg1 as usize] = n;
                Ok(PackedResult::Advance)
            }
            D_F_STORE_NUM => {
                self.registers[arg1 as usize] = Value16::number(self.f_slots[arg2 as usize]);
                Ok(PackedResult::Advance)
            }
            D_F_ADD => { self.f_slots[arg1 as usize] = self.f_slots[(arg2 >> 8) as usize] + self.f_slots[(arg2 & 0xFF) as usize]; Ok(PackedResult::Advance) }
            D_F_SUB => { self.f_slots[arg1 as usize] = self.f_slots[(arg2 >> 8) as usize] - self.f_slots[(arg2 & 0xFF) as usize]; Ok(PackedResult::Advance) }
            D_F_MUL => { self.f_slots[arg1 as usize] = self.f_slots[(arg2 >> 8) as usize] * self.f_slots[(arg2 & 0xFF) as usize]; Ok(PackedResult::Advance) }
            D_F_DIV => { self.f_slots[arg1 as usize] = self.f_slots[(arg2 >> 8) as usize] / self.f_slots[(arg2 & 0xFF) as usize]; Ok(PackedResult::Advance) }
            D_F_SIN => { self.f_slots[arg1 as usize] = self.f_slots[arg2 as usize].sin(); Ok(PackedResult::Advance) }
            D_F_COS => { self.f_slots[arg1 as usize] = self.f_slots[arg2 as usize].cos(); Ok(PackedResult::Advance) }
            D_F_SQRT => { self.f_slots[arg1 as usize] = self.f_slots[arg2 as usize].sqrt(); Ok(PackedResult::Advance) }
            D_F_CONST => { self.f_slots[arg1 as usize] = bytecode.get_numeric_constant(arg2 as usize); Ok(PackedResult::Advance) }
            D_F_MOVE => { self.f_slots[arg1 as usize] = self.f_slots[arg2 as usize]; Ok(PackedResult::Advance) }

            // All other packed opcodes — fall through to full match
            _ => Ok(PackedResult::Fallthrough),
        }
    }
}
