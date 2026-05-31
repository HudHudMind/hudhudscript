#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_int_slot_super(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;
        if matches!(instr, Instruction::IntCmp {..} | Instruction::IntCmpI {..} | Instruction::JumpIfFalse {..} | Instruction::JumpIfTrue {..} | Instruction::Jump(..)) {
        }

        match instr {
            // Slot-based super-instructions → step_super_instructions()
            // ── Register-based VM instructions ─────────────────────────
            Instruction::IntAdd { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_add(p2 as i64)),
                    (ReprTag::Number, ReprTag::Number) => {
                        Value16::number(f64::from_bits(p1) + f64::from_bits(p2))
                    }
                    (ReprTag::Int, ReprTag::Number) => Value16::number(p1 as i64 as f64 + f64::from_bits(p2)),
                    (ReprTag::Number, ReprTag::Int) => Value16::number(f64::from_bits(p1) + p2 as i64 as f64),
                    _ => {
                        let a = &self.registers[*src1 as usize];
                        let b = &self.registers[*src2 as usize];
                        if let (Some(a), Some(b)) = (a.as_string(), b.as_string()) {
                            Value16::string(a + &b)
                        } else {
                            return Err(Self::runtime_error_with_pos("IntAdd: operands not numeric/string", bytecode, ip));
                        }
                    }
                };
            }
            Instruction::IntSub { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_sub(p2 as i64)),
                    (ReprTag::Number, ReprTag::Number) => {
                        Value16::number(f64::from_bits(p1) - f64::from_bits(p2))
                    }
                    (ReprTag::Int, ReprTag::Number) => Value16::number(p1 as i64 as f64 - f64::from_bits(p2)),
                    (ReprTag::Number, ReprTag::Int) => Value16::number(f64::from_bits(p1) - p2 as i64 as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntSub: operands not numeric", bytecode, ip)),
                };
            }
            Instruction::IntMul { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_mul(p2 as i64)),
                    (ReprTag::Number, ReprTag::Number) => {
                        Value16::number(f64::from_bits(p1) * f64::from_bits(p2))
                    }
                    (ReprTag::Int, ReprTag::Number) => Value16::number(p1 as i64 as f64 * f64::from_bits(p2)),
                    (ReprTag::Number, ReprTag::Int) => Value16::number(f64::from_bits(p1) * p2 as i64 as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntMul: operands not numeric", bytecode, ip)),
                };
            }
            Instruction::IntAddI { dst, src, imm } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_add(*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) + (*imm as f64)),
                    _ => return Err(Self::runtime_error_with_pos("IntAddI: src not numeric", bytecode, ip)),
                };
            }
            Instruction::IntSubI { dst, src, imm } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_sub(*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) - (*imm as f64)),
                    _ => return Err(Self::runtime_error_with_pos("IntSubI: src not numeric", bytecode, ip)),
                };
            }
            Instruction::IntMulI { dst, src, imm } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => Value16::int((payload as i64).wrapping_mul(*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) * (*imm as f64)),
                    _ => return Err(Self::runtime_error_with_pos("IntMulI: src not numeric", bytecode, ip)),
                };
            }
            Instruction::IntDivI { dst, src, imm } => {
                if *imm == 0 { return Err(Self::runtime_error_with_pos("IntDivI: division by zero", bytecode, ip)); }
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => Value16::int((payload as i64) / (*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) / (*imm as f64)),
                    _ => return Err(Self::runtime_error_with_pos("IntDivI: src not numeric", bytecode, ip)),
                };
            }
            Instruction::IntModI { dst, src, imm } => {
                if *imm == 0 { return Err(Self::runtime_error_with_pos("IntModI: modulo by zero", bytecode, ip)); }
                let (tag, payload) = self.registers[*src as usize].split_tag();
                self.registers[*dst as usize] = match tag {
                    ReprTag::Int => Value16::int((payload as i64) % (*imm as i64)),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) % (*imm as f64)),
                    _ => return Err(Self::runtime_error_with_pos("IntModI: src not numeric", bytecode, ip)),
                };
            }
            Instruction::IntCmpI { dst, src, imm, op } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                let result = match tag {
                    ReprTag::Int => {
                        let a = payload as i64;
                        let b = *imm as i64;
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => return Err(Self::runtime_error_with_pos(&format!("IntCmpI: unknown op {}", op), bytecode, ip)) }
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(payload);
                        let b = *imm as f64;
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => return Err(Self::runtime_error_with_pos(&format!("IntCmpI: unknown op {}", op), bytecode, ip)) }
                    }
                    _ => return Err(Self::runtime_error_with_pos("IntCmpI: src not numeric", bytecode, ip)),
                };
                self.registers[*dst as usize] = Value16::bool_(result);
            }
            Instruction::IntCmp { dst, src1, src2, op } => {
                let v1 = &self.registers[*src1 as usize];
                let v2 = &self.registers[*src2 as usize];
                let (t1, p1) = v1.split_tag();
                let (t2, p2) = v2.split_tag();
                let result = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        let a = p1 as i64;
                        let b = p2 as i64;
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => return Err(Self::runtime_error_with_pos(&format!("IntCmp: unknown op {}", op), bytecode, ip)) }
                    }
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => return Err(Self::runtime_error_with_pos(&format!("IntCmp: unknown op {}", op), bytecode, ip)) }
                    }
                    _ => {
                        if let (Some(a), Some(b)) = (v1.as_str(), v2.as_str()) {
                            match *op { 0 => a < b, 1 => a <= b, 2 => a > b, 3 => a >= b, 4 => a == b, 5 => a != b, _ => return Err(Self::runtime_error_with_pos(&format!("IntCmp: unknown op {}", op), bytecode, ip)) }
                        } else if let (Some(a), Some(b)) = (v1.as_bool(), v2.as_bool()) {
                            match *op { 4 => a == b, 5 => a != b, 0 => !a && b, 1 => !a || a == b, 2 => a && !b, 3 => a || a == b, _ => return Err(Self::runtime_error_with_pos(&format!("IntCmp: unknown op {}", op), bytecode, ip)) }
                        } else if v1.is_null() || v2.is_null() {
                            let both_null = v1.is_null() && v2.is_null();
                            match *op { 4 => both_null, 5 => !both_null, _ => false }
                        } else {
                            return Err(Self::runtime_error_with_pos(&format!("IntCmp: incompatible types {:?} {:?}", t1, t2), bytecode, ip));
                        }
                    }
                };
                self.registers[*dst as usize] = Value16::bool_(result);
            }
            Instruction::NumAddI { dst, src, imm } => {
                self.registers[*dst as usize] = Value16::number(
                    self.registers[*src as usize].as_number_unchecked() + (*imm as f64));
            }
            Instruction::NumSubI { dst, src, imm } => {
                self.registers[*dst as usize] = Value16::number(
                    self.registers[*src as usize].as_number_unchecked() - (*imm as f64));
            }
            Instruction::NumMulI { dst, src, imm } => {
                self.registers[*dst as usize] = Value16::number(
                    self.registers[*src as usize].as_number_unchecked() * (*imm as f64));
            }
            Instruction::NumDivI { dst, src, imm } => {
                self.registers[*dst as usize] = Value16::number(
                    self.registers[*src as usize].as_number_unchecked() / (*imm as f64));
            }
            Instruction::NumDiv { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize].as_number_unchecked();
                let b = self.registers[*src2 as usize].as_number_unchecked();
                if b == 0.0 { return Err(Self::runtime_error_with_pos("Division by zero", bytecode, ip)); }
                self.registers[*dst as usize] = Value16::number(a / b);
            }
            Instruction::IntDiv { dst, src1, src2 } => {
                let a_val = &self.registers[*src1 as usize];
                let b_val = &self.registers[*src2 as usize];
                debug_assert!(a_val.is_int() && b_val.is_int(), "IntDiv requires Int operands");
                let a = a_val.as_int_unchecked();
                let b = b_val.as_int_unchecked();
                if b == 0 { return Err(Self::runtime_error_with_pos("Division by zero", bytecode, ip)); }
                self.registers[*dst as usize] = Value16::int(a / b);
            }
            Instruction::IntMod { dst, src1, src2 } => {
                let a_val = &self.registers[*src1 as usize];
                let b_val = &self.registers[*src2 as usize];
                debug_assert!(a_val.is_int() && b_val.is_int(), "IntMod requires Int operands");
                let a = a_val.as_int_unchecked();
                let b = b_val.as_int_unchecked();
                if b == 0 { return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip)); }
                self.registers[*dst as usize] = Value16::int(a % b);
            }
            Instruction::NumMod { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => {
                        let a = p1 as i64; let b = p2 as i64;
                        if b == 0 { return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip)); }
                        Value16::int(a % b)
                    }
                    (ReprTag::Number, ReprTag::Int) => {
                        let a = f64::from_bits(p1); let b = p2 as i64;
                        if b == 0 { return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip)); }
                        let a_int = a as i64;
                        if a_int as f64 == a { Value16::int(a_int % b) }
                        else { Value16::number(a % (b as f64)) }
                    }
                    _ => {
                        let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        if b == 0.0 { return Err(Self::runtime_error_with_pos("Modulo by zero", bytecode, ip)); }
                        Value16::number(if b == 1.0 { a.fract() } else { a % b })
                    }
                };
            }
            Instruction::LoadIntConst { dst, const_idx } => {
                self.registers[*dst as usize] = Value16::int(bytecode.int_constants[*const_idx as usize]);
            }
            Instruction::LoadConst { dst, const_idx } => {
                self.registers[*dst as usize] = ctx.constants[*const_idx as usize];
            }
            Instruction::LoadNumConst { dst, const_idx } => {
                let bits = bytecode.numeric_constants[*const_idx as usize];
                self.registers[*dst as usize] = Value16::number(f64::from_bits(bits));
            }

            Instruction::JumpIfFalse { src, offset } => {
                let v = &self.registers[*src as usize];
                if !v.is_truthy() {
                    let new_ip = (ip as i64).wrapping_add(*offset as i64);
                    if new_ip < 0 || new_ip > instructions.len() as i64 {
                        return Err(Self::runtime_error_with_pos(format!("JumpIfFalse out of bounds: ip={} offset={}", ip, offset), bytecode, ip));
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

            Instruction::Index { dst, obj, idx } => {
                let obj_val = &self.registers[*obj as usize];
                let idx_val = &self.registers[*idx as usize];
                let idx_i = idx_val.as_number_fast().map(|n| n as usize);
                let result = if let (Some(arr), Some(i)) = (obj_val.as_array(), idx_i) {
                    if i < arr.len() { arr[i].clone() } else {
                        return Err(Self::runtime_error_with_pos(format!("Array index out of bounds: {}", i), bytecode, ip));
                    }
                } else if let (Some(s), Some(i)) = (obj_val.as_string(), idx_i) {
                    // ASCII fast path: byte index avoids O(n) char iteration
                    let bytes = s.as_bytes();
                    if i < bytes.len() && bytes[i].is_ascii() {
                        Value16::string_ascii(bytes[i])
                    } else {
                        s.chars().nth(i).map(|c| Value16::string(c.to_string())).unwrap_or(Value16::null())
                    }
                } else if let Some(map) = obj_val.as_object() {
                    let key = idx_val.as_string().unwrap_or_default();
                    // SOP: subject instance state read via index — read from live instance
                    if map.get("__type").and_then(|v| v.as_string()).as_deref() == Some("subject_instance") {
                        if let Some(id) = map.get("__instance_id").and_then(|v| v.as_string()) {
                            if let Some(inst) = self.subject_instances.get(&id) {
                                if let Some(val) = inst.state.get(&key) {
                                    *val
                                } else {
                                    map.get(&key).copied().unwrap_or(Value16::null())
                                }
                            } else {
                                // SOP0005: despawned subject accessed
                                return Err(compile_codes::runtime_error(format!(
                                    "Cannot access index '{}' on despawned subject '{}'",
                                    key, id
                                )));
                            }
                        } else {
                            map.get(&key).copied().unwrap_or(Value16::null())
                        }
                    } else {
                        map.get(&key).copied().unwrap_or(Value16::null())
                    }
                } else if let Some(inst) = obj_val.as_instance_data() {
                    let key = idx_val.as_string().unwrap_or_default();
                    inst.fields.get(&key).copied().unwrap_or(Value16::null())
                } else {
                    return Err(Self::runtime_error_with_pos("Index: expected array or string", bytecode, ip));
                };
                self.registers[*dst as usize] = result;
            }
            // MakeArray count>0 path removed — compiler always uses count:0 + ArrayPush
            Instruction::MakeArray { dst, count } => {
                // Always count=0: elements are added via ArrayPush
                self.registers[*dst as usize] = Value16::array(Vec::new());
            }
            Instruction::MakeObject { dst, count } => {
                // Compiler always emits count=0 with SetProperty for each key-value.
                // count>0 path removed — was a shadow accumulator bug.
                let _n = *count as usize;
                let properties = std::collections::HashMap::new();
                self.registers[*dst as usize] = Value16::object(properties);
            }
            Instruction::Call { dst, payload_idx, first_arg, arg_count: _ } => {
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                return Ok(StepAction::Call {
                    func_sym: payload.sym,
                    arg_count: payload.arg_count,
                    first_arg: *first_arg,
                    dst: *dst,
                });
            }
            Instruction::LoadGlobal { dst, sym } => {
                let name = self.sym_cache.entry(*sym as u32).or_insert_with_key(|&s| bytecode.resolve_symbol(s)).clone();
                let value = self.get_var_cloned(&name).ok_or_else(|| {
                    Self::runtime_error_with_pos(format!("Undefined variable: {}", name), bytecode, ip)
                })?;
                self.registers[*dst as usize] = value;
            }
            Instruction::StoreGlobal { src, sym } => {
                let name = self.sym_cache.entry(*sym as u32).or_insert_with_key(|&s| bytecode.resolve_symbol(s)).clone();
                if self.immutables.contains(&name) {
                    return Err(Self::runtime_error_with_pos(
                        format!("Cannot assign to constant '{}'", name),
                        bytecode,
                        ip,
                    ));
                }
                let value = self.registers[*src as usize];
                let _ = self.set_var(&name, value);
                // Mirror to main-frame register slot so top-level code
                // that reads via LoadLocal sees StoreGlobal updates from
                // inner functions (function_no_return parity).
                for (slot, slot_name) in bytecode.main_local_names.iter().enumerate() {
                    if slot_name == &name {
                        self.registers[slot] = value;
                        break;
                    }
                }
                if let Some(sym_id) = hudhudscript_bytecode::interner::try_resolve_id(&name) {
                    if let Some(entry) = self.call_cache.get_mut(sym_id as usize) {
                        *entry = None;
                    }
                }
            }
            Instruction::DeclGlobal { src, sym } => {
                let name = bytecode.resolve_symbol(*sym as u32);
                let value = self.registers[*src as usize];
                self.globals.insert(name, value);
            }
            Instruction::StoreConst { src, sym } => {
                let name = bytecode.resolve_symbol(*sym as u32);
                let value = self.registers[*src as usize];
                // K2-3: local_immutables eliminated — const check is compile-time.
                self.set_var(&name, value)?;
                self.immutables.insert(name.to_string());
                if let Some(sym_id) = hudhudscript_bytecode::interner::try_resolve_id(&name) {
                    if let Some(entry) = self.call_cache.get_mut(sym_id as usize) {
                        *entry = None;
                    }
                }
            }

            // StrCat removed — compiler emits StrCat (register-based)

            Instruction::StrCat { dst, src1, src2 } => {
                let l = self.registers[*src1 as usize];
                let r = self.registers[*src2 as usize];
                self.registers[*dst as usize] = hudhudscript_bytecode::shared_value::shared_add(&l, &r)?;
            }
            Instruction::StrCatMut { dst, src2 } => {
                let r = self.registers[*src2 as usize];
                let dst_ref = &mut self.registers[*dst as usize];
                if let (Some(s), Some(r_str)) = (dst_ref.as_string_mut(), r.as_str()) {
                    s.push_str(r_str);
                } else {
                    let l = *dst_ref;
                    *dst_ref = hudhudscript_bytecode::shared_value::shared_add(&l, &r)?;
                }
            }
            Instruction::StringIndexOf { dst, haystack, needle } => {
                let s = self.registers[*haystack as usize].as_str_unchecked();
                let pat = self.registers[*needle as usize].as_str_unchecked();
                let idx = s.find(pat).map(|i| i as f64).unwrap_or(-1.0);
                self.registers[*dst as usize] = Value16::number(idx);
            }
            Instruction::StringContains { dst, haystack, needle } => {
                let s = self.registers[*haystack as usize].as_str_unchecked();
                let pat = self.registers[*needle as usize].as_str_unchecked();
                self.registers[*dst as usize] = Value16::boolean(s.contains(pat));
            }
            Instruction::ArrayPush { dst, arr, val } => {
                let v = self.registers[*val as usize];
                let mut arr_val = self.registers[*arr as usize];
                debug_assert!(arr_val.0.tag() == ReprTag::Dynamic);
                arr_val.as_array_mut_unchecked().push(v);
                self.registers[*dst as usize] = arr_val;
            }
            Instruction::GetProperty { dst, obj, prop_sym } => {
                let obj_v = self.registers[*obj as usize];
                let val = self.resolve_property(obj_v, &hudhudscript_bytecode::SymId(*prop_sym as u32))?;
                self.registers[*dst as usize] = val;
            }
            Instruction::Neg { dst, src } => {
                let (tag, payload) = self.registers[*src as usize].split_tag();
                let result = match tag {
                    ReprTag::Int => Value16::int(-(payload as i64)),
                    ReprTag::Number => Value16::number(-f64::from_bits(payload)),
                    _ => return Err(Self::runtime_error_with_pos("Neg: expected Int or Number", bytecode, ip)),
                };
                self.registers[*dst as usize] = result;
            }
            Instruction::Not { dst, src } => {
                self.registers[*dst as usize] = Value16::bool_(!self.registers[*src as usize].is_truthy());
            }
            Instruction::IndexAssign { obj, idx, val } => {
                let idx_val = self.registers[*idx as usize];
                let new_val = self.registers[*val as usize];
                let obj_ref = &mut self.registers[*obj as usize];
                if let Some(arr) = obj_ref.as_array_mut() {
                    let i = idx_val.as_number_fast().map(|n| n as usize).ok_or_else(||
                        Self::runtime_error_with_pos("Array index must be a number", bytecode, ip))?;
                    if i >= arr.len() { arr.resize(i + 1, Value16::null()); }
                    arr[i] = new_val;
                } else if let Some(map) = obj_ref.as_object_mut() {
                    let key = idx_val.as_string().unwrap_or_default();
                    map.insert(key, new_val);
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot index-assign into non-array/object", bytecode, ip,
                    ));
                }
            }

            Instruction::Return { src } => {
                self.registers[255] = self.registers[*src as usize];
                return Ok(StepAction::Return);
            }
            Instruction::IntLeRRJumpIfFalse { src1, src2, offset } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                let cond = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => (p1 as i64) <= (p2 as i64),
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        a <= b
                    }
                    _ => return Err(Self::runtime_error_with_pos("IntLeRRJumpIfFalse: operands not numeric", bytecode, ip)),
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
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        a < b
                    }
                    _ => return Err(Self::runtime_error_with_pos("IntLtRRJumpIfFalse: operands not numeric", bytecode, ip)),
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntLeRIJumpIfFalse { src, imm, offset } => {
                let (tag, p) = self.registers[*src as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => (p as i64) <= (*imm as i64),
                    ReprTag::Number => f64::from_bits(p) <= (*imm as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntLeRIJumpIfFalse: src not numeric", bytecode, ip)),
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntLtRIJumpIfFalse { src, imm, offset } => {
                let (tag, p) = self.registers[*src as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => (p as i64) < (*imm as i64),
                    ReprTag::Number => f64::from_bits(p) < (*imm as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntLtRIJumpIfFalse: src not numeric", bytecode, ip)),
                };
                if !cond {
                    let new_ip = (ip as i64).wrapping_add(*offset as i64);
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::Move { dst, src } => {
                self.registers[*dst as usize] = self.registers[*src as usize];
            }
            // NumMul/NumAdd/NumSub — packed dispatch path (handles Int or Number operands via as_number_fast)
            Instruction::NumMul { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize].as_number_fast().unwrap_or(0.0);
                let b = self.registers[*src2 as usize].as_number_fast().unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a * b);
            }
            Instruction::NumAdd { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize].as_number_fast().unwrap_or(0.0);
                let b = self.registers[*src2 as usize].as_number_fast().unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a + b);
            }
            Instruction::NumSub { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize].as_number_fast().unwrap_or(0.0);
                let b = self.registers[*src2 as usize].as_number_fast().unwrap_or(0.0);
                self.registers[*dst as usize] = Value16::number(a - b);
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
