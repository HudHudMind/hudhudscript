
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
        if dense == 0xFF { return Ok(PackedResult::Fallthrough); }
        match dense {

            // ── Jumps (hot in loops / conditionals) ─────────────────
            // i32 relative offset packed as i16 (sign-extended back).
            D_JUMP => {
                let offset = (arg2 as i16) as i32;
                let target = (ip as i64).wrapping_add(offset as i64);
                debug_assert!(target >= 0 && target <= instructions.len() as i64,
                    "Jump out of bounds: ip={} offset={}", ip, offset);
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

            // Variable ops — handled via register-based load/store/declare
            D_LOAD_VAR | D_STORE_VAR | D_DECL_VAR => Ok(PackedResult::Fallthrough),

            // ── PERF-36 constant loads on fast dispatch ─────────────
            D_LOAD_NUM_CONST => {
                let idx = arg2 as usize;
                let n = bytecode.get_numeric_constant(idx);
                self.registers[255] = Value16::number(n);
                Ok(PackedResult::Advance)
            }
            // A3b: integer-pool load on the fast-dispatch path.  Matches
            // LoadNumConst's hot-path shape — critical because the
            // compiler routes every integer-valued numeric literal
            // through here (fib's `n - 1` / `n - 2` literals before
            // I6 fusion steals them into NumSubISlot).
            D_LOAD_INT_CONST => {
                let idx = arg2 as usize;
                let v = bytecode.get_int_constant(idx);
                self.registers[255] = Value16::int(v);
                Ok(PackedResult::Advance)
            }
            // IndexFast + IntIncrSlot — fall through to unpacked
            D_INDEX_FAST_ISLOT => Ok(PackedResult::Fallthrough),
            D_INT_INCR_SLOT => Ok(PackedResult::Fallthrough),
            D_INT_SUB_LOCAL_I | D_INT_ADD_LOCAL_I => {
                let dst = arg1 as usize;
                let payload = bytecode.get_super_instr_payload(arg2 as u32);
                let slot_idx = payload.slot as usize;
                let (tag, p) = self.registers[slot_idx].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => {
                        let a = p as i64;
                        let r = if dense == D_INT_SUB_LOCAL_I { a.wrapping_sub(payload.imm as i64) }
                                else { a.wrapping_add(payload.imm as i64) };
                        Value16::int(r)
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(p);
                        let r = if dense == D_INT_SUB_LOCAL_I { a - payload.imm as f64 }
                                else { a + payload.imm as f64 };
                        Value16::number(r)
                    }
                    _ => return Err(Self::runtime_error_with_pos(
                        "IntSubLocalI/IntAddLocalI: expected numeric local", bytecode, ip)),
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
                        let a = p1 as i64; let b = p2 as i64;
                        match dense { D_INT_EQ_RR => a==b, D_INT_LT_RR => a<b, D_INT_LE_RR => a<=b, D_INT_NE_RR => a!=b, _ => unreachable!() }
                    }
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1==ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2==ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        match dense { D_INT_EQ_RR => a==b, D_INT_LT_RR => a<b, D_INT_LE_RR => a<=b, D_INT_NE_RR => a!=b, _ => unreachable!() }
                    }
                    _ => {
                        // String/bool/null fallback — use safe type checks
                        let v1 = &self.registers[src1];
                        let v2 = &self.registers[src2];
                        if let (Some(a), Some(b)) = (v1.as_str(), v2.as_str()) {
                            match dense { D_INT_EQ_RR => a==b, D_INT_LT_RR => a<b, D_INT_LE_RR => a<=b, D_INT_NE_RR => a!=b, _ => unreachable!() }
                        } else if let (Some(a), Some(b)) = (v1.as_bool(), v2.as_bool()) {
                            match dense { D_INT_EQ_RR => a==b, D_INT_NE_RR => a!=b, D_INT_LT_RR => !a&&b, D_INT_LE_RR => !a||a==b, _ => unreachable!() }
                        } else if v1.is_null() || v2.is_null() {
                            let both = v1.is_null() && v2.is_null();
                            match dense { D_INT_EQ_RR => both, D_INT_NE_RR => !both, _ => false }
                        } else {
                            match dense { D_INT_EQ_RR => false, D_INT_NE_RR => true, _ => false }
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
            D_INT_ADD_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as i64;
                let dst = arg1 as usize;
                let (tag, payload) = self.registers[src].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_add(imm)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) + imm as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntAddI: src not numeric", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_INT_MUL_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as i64;
                let dst = arg1 as usize;
                let (tag, payload) = self.registers[src].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_mul(imm)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) * imm as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntMulI: src not numeric", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_INT_SUB_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as i64;
                let dst = arg1 as usize;
                let (tag, payload) = self.registers[src].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_sub(imm)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) - imm as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntSubI: src not numeric", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_NEG_R => {
                let src = arg2 as usize;
                let dst = arg1 as usize;
                let val = self.registers[src];
                if let Some(n) = val.as_int() {
                    self.registers[dst] = Value16::int(n.wrapping_neg());
                } else if let Some(n) = val.as_number() {
                    self.registers[dst] = Value16::number(-n);
                } else {
                    return Err(Self::runtime_error_with_pos("Neg: unsupported type", bytecode, ip));
                }
                Ok(PackedResult::Advance)
            }
            D_NOT_R => {
                let src = arg2 as usize;
                let dst = arg1 as usize;
                let val = self.registers[src];
                if let Some(b) = val.as_bool() {
                    self.registers[dst] = Value16::bool_(!b);
                } else {
                    return Err(Self::runtime_error_with_pos("Not: expected Bool", bytecode, ip));
                }
                Ok(PackedResult::Advance)
            }
            D_ARRAY_PUSH_RRR => {
                let arr_reg = ((arg2 >> 8) & 0xFF) as usize;
                let val_reg = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let val = self.registers[val_reg];
                if let Some(vec) = self.registers[arr_reg].as_array_mut() {
                    vec.push(val);
                    let arr_val = self.registers[arr_reg];
                    self.registers[dst] = arr_val;
                    Ok(PackedResult::Advance)
                } else {
                    Err(Self::runtime_error_with_pos("ArrayPush: not an array", bytecode, ip))
                }
            }
            D_INDEX_RRR => {
                let obj_reg = ((arg2 >> 8) & 0xFF) as usize;
                let idx_reg = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let obj = self.registers[obj_reg];
                let idx = self.registers[idx_reg];
                let idx_i = idx.as_number_fast().map(|n| n as i64);
                if let (Some(arr), Some(i)) = (obj.as_array(), idx_i) {
                    let i = i as usize;
                    if i < arr.len() {
                        self.registers[dst] = arr[i];
                        return Ok(PackedResult::Advance);
                    }
                    return Err(Self::runtime_error_with_pos(
                        format!("Array index out of bounds: {}", i),
                        bytecode, ip,
                    ));
                } else if let (Some(s), Some(i)) = (obj.as_string(), idx_i) {
                    let i = i as usize;
                    if let Some(&b) = s.as_bytes().get(i) {
                        if b < 0x80 {
                            self.registers[dst] = Value16::string_ascii(b);
                        } else {
                            let ch = s.chars().nth(i).map(|c| c.to_string());
                            let ch = ch.ok_or_else(|| Self::runtime_error_with_pos(
                                format!("String index out of bounds: {}", i),
                                bytecode, ip,
                            ))?;
                            self.registers[dst] = Value16::string(ch);
                        }
                        return Ok(PackedResult::Advance);
                    }
                    return Err(Self::runtime_error_with_pos(
                        format!("String index out of bounds: {}", i),
                        bytecode, ip,
                    ));
                }
                Ok(PackedResult::Fallthrough)
            }
            D_STRCAT_RRR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let result = hudhudscript_bytecode::shared_value::shared_add(
                    &self.registers[src1], &self.registers[src2]
                )?;
                self.registers[dst] = result;
                Ok(PackedResult::Advance)
            }
            D_STRCAT_MUT_RR => {
                let dst = arg1 as usize;
                let src2 = arg2 as usize;
                let r = self.registers[src2];
                let dst_ref = &mut self.registers[dst];
                if let (Some(s), Some(r_str)) = (dst_ref.as_string_mut(), r.as_str()) {
                    s.push_str(r_str);
                } else {
                    let l = *dst_ref;
                    *dst_ref = hudhudscript_bytecode::shared_value::shared_add(&l, &r)?;
                }
                Ok(PackedResult::Advance)
            }
            D_STRING_INDEX_OF_RRR => {
                let haystack = ((arg2 >> 8) & 0xFF) as usize;
                let needle = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                if let (Some(s), Some(pat)) = (self.registers[haystack].as_str(), self.registers[needle].as_str()) {
                    let idx = s.find(pat).map(|i| i as f64).unwrap_or(-1.0);
                    self.registers[dst] = Value16::number(idx);
                    Ok(PackedResult::Advance)
                } else {
                    Ok(PackedResult::Fallthrough)
                }
            }
            D_STRING_CONTAINS_RRR => {
                let haystack = ((arg2 >> 8) & 0xFF) as usize;
                let needle = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                if let (Some(s), Some(pat)) = (self.registers[haystack].as_str(), self.registers[needle].as_str()) {
                    self.registers[dst] = Value16::boolean(s.contains(pat));
                    Ok(PackedResult::Advance)
                } else {
                    Ok(PackedResult::Fallthrough)
                }
            }

            // Register-based VM opcodes — packed dispatch
            D_INT_ADD_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = self.registers[src1].split_tag();
                let (t2, p2) = self.registers[src2].split_tag();
                self.registers[dst] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_add(p2 as i64)),
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1==ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2==ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        Value16::number(a + b)
                    }
                    _ => {
                        let a_val = self.registers[src1];
                        let b_val = self.registers[src2];
                        if let (Some(a), Some(b)) = (a_val.as_string(), b_val.as_string()) {
                            Value16::string(a + &b)
                        } else {
                            return Err(Self::runtime_error_with_pos("AddRR: unsupported types", bytecode, ip));
                        }
                    }
                };
                Ok(PackedResult::Advance)
            }
            D_INT_SUB_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = self.registers[src1].split_tag();
                let (t2, p2) = self.registers[src2].split_tag();
                self.registers[dst] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_sub(p2 as i64)),
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1==ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2==ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        Value16::number(a - b)
                    }
                    _ => return Err(Self::runtime_error_with_pos("SubRR: unsupported types", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_INT_MUL_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = self.registers[src1].split_tag();
                let (t2, p2) = self.registers[src2].split_tag();
                self.registers[dst] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_mul(p2 as i64)),
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1==ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2==ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        Value16::number(a * b)
                    }
                    _ => return Err(Self::runtime_error_with_pos("MulRR: unsupported types", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_INT_MOD_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let a_val = &self.registers[src1];
                let b_val = &self.registers[src2];
                debug_assert!(a_val.is_int() && b_val.is_int());
                let a = a_val.as_int_unchecked();
                let b = b_val.as_int_unchecked();
                if b == 0 { return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip)); }
                self.registers[dst] = Value16::int(a % b);
                Ok(PackedResult::Advance)
            }
            // Float register arithmetic — packed fast path
            D_NUM_ADD_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = self.registers[src1].split_tag();
                let (t2, p2) = self.registers[src2].split_tag();
                let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                self.registers[dst] = Value16::number(a + b);
                Ok(PackedResult::Advance)
            }
            D_NUM_SUB_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = self.registers[src1].split_tag();
                let (t2, p2) = self.registers[src2].split_tag();
                let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                self.registers[dst] = Value16::number(a - b);
                Ok(PackedResult::Advance)
            }
            D_NUM_MUL_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = self.registers[src1].split_tag();
                let (t2, p2) = self.registers[src2].split_tag();
                let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                self.registers[dst] = Value16::number(a * b);
                Ok(PackedResult::Advance)
            }
            D_NUM_DIV_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = self.registers[src1].split_tag();
                let (t2, p2) = self.registers[src2].split_tag();
                let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                if b == 0.0 {
                    return Err(Self::runtime_error_with_pos("NumDiv: division by zero", bytecode, ip));
                }
                self.registers[dst] = Value16::number(a / b);
                Ok(PackedResult::Advance)
            }
            // Array index assignment — packed fast path
            D_INDEX_ASSIGN_RRR => {
                let obj_reg = arg1 as usize;
                let idx_reg = ((arg2 >> 8) & 0xFF) as usize;
                let val_reg = (arg2 & 0xFF) as usize;
                let idx = self.registers[idx_reg];
                let val = self.registers[val_reg];
                if let Some(i) = idx.as_number_fast().map(|n| n as usize) {
                    if let Some(arr) = self.registers[obj_reg].as_array_mut() {
                        if i >= arr.len() {
                            arr.resize(i + 1, Value16::null());
                        }
                        arr[i] = val;
                        return Ok(PackedResult::Advance);
                    }
                }
                Ok(PackedResult::Fallthrough)
            }
            // Float immediate arithmetic — packed fast path
            D_NUM_ADD_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as f64;
                let dst = arg1 as usize;
                let (tag, payload) = self.registers[src].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => Value16::number(payload as i64 as f64 + imm),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) + imm),
                    _ => return Err(Self::runtime_error_with_pos("NumAddI: src not numeric", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_NUM_SUB_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as f64;
                let dst = arg1 as usize;
                let (tag, payload) = self.registers[src].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => Value16::number(payload as i64 as f64 - imm),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) - imm),
                    _ => return Err(Self::runtime_error_with_pos("NumSubI: src not numeric", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_NUM_MUL_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as f64;
                let dst = arg1 as usize;
                let (tag, payload) = self.registers[src].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => Value16::number(payload as i64 as f64 * imm),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) * imm),
                    _ => return Err(Self::runtime_error_with_pos("NumMulI: src not numeric", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_NUM_DIV_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as f64;
                let dst = arg1 as usize;
                if imm == 0.0 {
                    return Err(Self::runtime_error_with_pos("NumDivI: division by zero", bytecode, ip));
                }
                let (tag, payload) = self.registers[src].split_tag();
                self.registers[dst] = match tag {
                    ReprTag::Int => Value16::number(payload as i64 as f64 / imm),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) / imm),
                    _ => return Err(Self::runtime_error_with_pos("NumDivI: src not numeric", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
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
                let src = arg1 as usize;
                let val = self.registers[src];
                self.registers[255] = val;
                self.last_return = val;
                Ok(PackedResult::Return)
            }

            // R1: LE/LT JUMP super-instructions (moved from dispatch_control.rs)
            D_INT_LE_JUMP_IF_FALSE => {
                let super_idx = arg2 as u32;
                let sp = bytecode.get_super_instr_payload(super_idx);
                let (tag, p) = self.registers[sp.slot as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => (p as i64) <= sp.imm as i64,
                    ReprTag::Number => f64::from_bits(p) <= sp.imm as f64,
                    _ => return Err(Self::runtime_error_with_pos("IntLeJumpIfFalse: expected numeric", bytecode, ip)),
                };
                if !cond {
                    let target = (ip as i64).wrapping_add(sp.offset as i64);
                    Ok(PackedResult::Jump(target as usize))
                } else { Ok(PackedResult::Advance) }
            }
            D_INT_LT_JUMP_IF_FALSE => {
                let super_idx = arg2 as u32;
                let sp = bytecode.get_super_instr_payload(super_idx);
                let (tag, p) = self.registers[sp.slot as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => (p as i64) < sp.imm as i64,
                    ReprTag::Number => f64::from_bits(p) < sp.imm as f64,
                    _ => return Err(Self::runtime_error_with_pos("IntLtJumpIfFalse: expected numeric", bytecode, ip)),
                };
                if !cond {
                    let target = (ip as i64).wrapping_add(sp.offset as i64);
                    Ok(PackedResult::Jump(target as usize))
                } else { Ok(PackedResult::Advance) }
            }

            // All other packed opcodes — fall through to full match
            _ => Ok(PackedResult::Fallthrough),
        }
    }
}
