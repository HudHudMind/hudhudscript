//! V2-B0: Int arithmetic packed handlers extracted from dispatch_general.rs.

use crate::vm::dense_ops::*;
use crate::vm::index_helpers::{numeric_index_i64, index_i64_to_usize};
use crate::vm::PackedResult;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, ReprTag, Value16};

#[rustfmt::skip]
pub(crate) fn dispatch_int_arithmetic(
    vm: &mut crate::vm::VM, dense: u8, arg1: u8, arg2: u16,
    bytecode: &Bytecode, ip: usize,
) -> CompileResult<PackedResult> {
    match dense {
            D_INT_ADD_RI => {
                let s = (arg2 & 0xFF) as usize;
                let i = ((arg2 >> 8) as i8) as i64;
                let d = arg1 as usize;
                let a = vm.registers[s];
                let (tag, payload) = a.split_tag();
                vm.registers[d] = match tag {
                    ReprTag::Int => {
                        if let Some(v) = (payload as i64).checked_add(i) { Value16::int(v) }
                        else { crate::vm::bigint_arith::int_add(a, Value16::int(i)).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))? }
                    }
                    ReprTag::Number => Value16::number(f64::from_bits(payload) + i as f64),
                    _ => crate::vm::bigint_arith::int_add(a, Value16::int(i)).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))?,
                };
                Ok(PackedResult::Advance)

            }
            D_INT_MUL_RI => {
                let s = (arg2 & 0xFF) as usize;
                let i = ((arg2 >> 8) as i8) as i64;
                let d = arg1 as usize;
                let a = vm.registers[s];
                let (tag, payload) = a.split_tag();
                vm.registers[d] = match tag {
                    ReprTag::Int => {
                        if let Some(v) = (payload as i64).checked_mul(i) { Value16::int(v) }
                        else { crate::vm::bigint_arith::int_mul(a, Value16::int(i)).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))? }
                    }
                    ReprTag::Number => Value16::number(f64::from_bits(payload) * i as f64),
                    _ => crate::vm::bigint_arith::int_mul(a, Value16::int(i)).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))?,
                };
                Ok(PackedResult::Advance)

            }
            D_INT_SUB_RI => {
                let s = (arg2 & 0xFF) as usize;
                let i = ((arg2 >> 8) as i8) as i64;
                let d = arg1 as usize;
                let a = vm.registers[s];
                let (tag, payload) = a.split_tag();
                vm.registers[d] = match tag {
                    ReprTag::Int => {
                        if let Some(v) = (payload as i64).checked_sub(i) { Value16::int(v) }
                        else { crate::vm::bigint_arith::int_sub(a, Value16::int(i)).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))? }
                    }
                    ReprTag::Number => Value16::number(f64::from_bits(payload) - i as f64),
                    _ => crate::vm::bigint_arith::int_sub(a, Value16::int(i)).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))?,
                };
                Ok(PackedResult::Advance)

            }
            D_NEG_R => {
                let src = arg2 as usize;
                let dst = arg1 as usize;
                let val = vm.registers[src];
                if let Some(n) = val.as_int() {
                    vm.registers[dst] = Value16::int(n.wrapping_neg());
                } else if let Some(n) = val.as_number() {
                    vm.registers[dst] = Value16::number(-n);
                } else {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        "Neg: unsupported type",
                        bytecode,
                        ip,
                    ));
                }
                Ok(PackedResult::Advance)
            }
            D_NOT_R => {
                let src = arg2 as usize;
                let dst = arg1 as usize;
                let val = vm.registers[src];
                if let Some(b) = val.as_bool() {
                    vm.registers[dst] = Value16::bool_(!b);
                } else {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        "Not: expected Bool",
                        bytecode,
                        ip,
                    ));
                }
                Ok(PackedResult::Advance)
            }
            D_ARRAY_PUSH_RRR => {
                let arr_reg = ((arg2 >> 8) & 0xFF) as usize;
                let val_reg = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let val = vm.registers[val_reg];
                if let Some(vec) = vm.registers[arr_reg].as_array_mut() {
                    vec.push(val);
                    let arr_val = vm.registers[arr_reg];
                    vm.registers[dst] = arr_val;
                    Ok(PackedResult::Advance)
                } else {
                    Err(crate::vm::VM::runtime_error_with_pos(
                        "ArrayPush: not an array",
                        bytecode,
                        ip,
                    ))
                }
            }
            D_INDEX_RRR => {
                let obj_reg = ((arg2 >> 8) & 0xFF) as usize;
                let idx_reg = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let obj = vm.registers[obj_reg];
                let idx = vm.registers[idx_reg];

                if let Some(arr) = obj.as_array() {
                    let i = numeric_index_i64(idx)
                        .and_then(index_i64_to_usize)
                        .ok_or_else(|| {
                            crate::vm::VM::runtime_error_with_pos(
                                "Array index must be a non-negative number",
                                bytecode,
                                ip,
                            )
                        })?;
                    if i >= arr.len() {
                        return Err(crate::vm::VM::runtime_error_with_pos(
                            format!("Array index out of bounds: {}", i),
                            bytecode,
                            ip,
                        ));
                    }
                    vm.registers[dst] = arr[i];
                    return Ok(PackedResult::Advance);
                } else if let Some(s) = obj.as_string() {
                    let i = numeric_index_i64(idx)
                        .and_then(index_i64_to_usize)
                        .ok_or_else(|| {
                            crate::vm::VM::runtime_error_with_pos(
                                "String index must be a non-negative number",
                                bytecode,
                                ip,
                            )
                        })?;
                    if let Some(&b) = s.as_bytes().get(i) {
                        if b < 0x80 {
                            vm.registers[dst] = Value16::string_ascii(b);
                        } else {
                            let ch = if let Some(cached) = vm.str_chars_cache.get(s.as_str()) {
                                cached.get(i).cloned()
                            } else {
                                let chars: Vec<char> = s.chars().collect();
                                let ch = chars.get(i).cloned();
                                vm.str_chars_cache.insert(s.to_string(), chars);
                                ch
                            };
                            let ch = ch.ok_or_else(|| {
                                crate::vm::VM::runtime_error_with_pos(
                                    format!("String index out of bounds: {}", i),
                                    bytecode,
                                    ip,
                                )
                            })?;
                            vm.registers[dst] = Value16::string(ch.to_string());
                        }
                        return Ok(PackedResult::Advance);
                    }
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        format!("String index out of bounds: {}", i),
                        bytecode,
                        ip,
                    ));
                }
                Ok(PackedResult::Fallthrough)
            }
            // P1b: specialized index packed fast paths — skip type dispatch
            D_INDEX_ARRAY_RRR => {
                let obj_reg = ((arg2 >> 8) & 0xFF) as usize;
                let idx_reg = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let obj = vm.registers[obj_reg];
                let arr = obj.as_array().ok_or_else(|| {
                    crate::vm::VM::runtime_error_with_pos(
                        "IndexArray: expected array",
                        bytecode,
                        ip,
                    )
                })?;
                let i = numeric_index_i64(vm.registers[idx_reg])
                    .and_then(index_i64_to_usize)
                    .ok_or_else(|| {
                        crate::vm::VM::runtime_error_with_pos(
                            "IndexArray: index must be non-negative number",
                            bytecode,
                            ip,
                        )
                    })?;
                if i >= arr.len() {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        format!("Array index out of bounds: {}", i),
                        bytecode,
                        ip,
                    ));
                }
                vm.registers[dst] = arr[i];
                Ok(PackedResult::Advance)
            }
            D_INDEX_STRING_ASCII_RRR => {
                let obj_reg = ((arg2 >> 8) & 0xFF) as usize;
                let idx_reg = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let obj = vm.registers[obj_reg];
                let s = obj.as_string().ok_or_else(|| {
                    crate::vm::VM::runtime_error_with_pos(
                        "IndexStringAscii: expected string",
                        bytecode,
                        ip,
                    )
                })?;
                let i = numeric_index_i64(vm.registers[idx_reg])
                    .and_then(index_i64_to_usize)
                    .ok_or_else(|| {
                        crate::vm::VM::runtime_error_with_pos(
                            "IndexStringAscii: index must be non-negative number",
                            bytecode,
                            ip,
                        )
                    })?;
                if let Some(&b) = s.as_bytes().get(i) {
                    if b < 0x80 {
                        vm.registers[dst] = Value16::string_ascii(b);
                    } else {
                        let ch = if let Some(cached) = vm.str_chars_cache.get(s.as_str()) {
                            cached.get(i).cloned()
                        } else {
                            let chars: Vec<char> = s.chars().collect();
                            let ch = chars.get(i).cloned();
                            vm.str_chars_cache.insert(s.to_string(), chars);
                            ch
                        };
                        let ch = ch.ok_or_else(|| {
                            crate::vm::VM::runtime_error_with_pos(
                                format!("String index out of bounds: {}", i),
                                bytecode,
                                ip,
                            )
                        })?;
                        vm.registers[dst] = Value16::string(ch.to_string());
                    }
                    return Ok(PackedResult::Advance);
                }
                Err(crate::vm::VM::runtime_error_with_pos(
                    format!("String index out of bounds: {}", i),
                    bytecode,
                    ip,
                ))
            }
            D_INDEX_ASSIGN_RRR => {
                let obj_reg = arg1 as usize;
                let idx_reg = ((arg2 >> 8) & 0xFF) as usize;
                let val_reg = (arg2 & 0xFF) as usize;
                let idx = vm.registers[idx_reg];
                let val = vm.registers[val_reg];
                if let Some(arr) = vm.registers[obj_reg].as_array_mut() {
                    let i = numeric_index_i64(idx)
                        .and_then(index_i64_to_usize)
                        .ok_or_else(|| {
                            crate::vm::VM::runtime_error_with_pos(
                                "Array index must be a non-negative number",
                                bytecode,
                                ip,
                            )
                        })?;
                    if i >= arr.len() {
                        arr.resize(i + 1, Value16::null());
                    }
                    arr[i] = val;
                    return Ok(PackedResult::Advance);
                }
                Ok(PackedResult::Fallthrough)
            }
            D_STRCAT_RRR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let result = hudhudscript_bytecode::shared_value::shared_add(
                    &vm.registers[src1],
                    &vm.registers[src2],
                )?;
                vm.registers[dst] = result;
                Ok(PackedResult::Advance)
            }
            D_STRCAT_MUT_RR => {
                let dst = arg1 as usize;
                let src2 = arg2 as usize;
                let r = vm.registers[src2];
                let dst_ref = &mut vm.registers[dst];
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
                if let (Some(s), Some(pat)) = (
                    vm.registers[haystack].as_str(),
                    vm.registers[needle].as_str(),
                ) {
                    let idx = s.find(pat).map(|i| i as f64).unwrap_or(-1.0);
                    vm.registers[dst] = Value16::number(idx);
                    Ok(PackedResult::Advance)
                } else {
                    Ok(PackedResult::Fallthrough)
                }
            }
            D_STRING_CONTAINS_RRR => {
                let haystack = ((arg2 >> 8) & 0xFF) as usize;
                let needle = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                if let (Some(s), Some(pat)) = (
                    vm.registers[haystack].as_str(),
                    vm.registers[needle].as_str(),
                ) {
                    vm.registers[dst] = Value16::boolean(s.contains(pat));
                    Ok(PackedResult::Advance)
                } else {
                    Ok(PackedResult::Fallthrough)
                }
            }

            // Register-based VM opcodes — packed dispatch
            D_INT_ADD_RR => {
                let s1 = ((arg2 >> 8) & 0xFF) as usize;
                let s2 = (arg2 & 0xFF) as usize;
                let d = arg1 as usize;
                let a = vm.registers[s1];
                let b = vm.registers[s2];
                // P0-B: sequential if-let instead of tuple-match. Int,Int is
                // the common hot case → branches well-predicted by the CPU.
                vm.registers[d] = if let (Some(x), Some(y)) = (a.as_int_fast(), b.as_int_fast()) {
                    if let Some(v) = x.checked_add(y) { Value16::int(v) }
                    else { crate::vm::bigint_arith::int_add(a, b).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))? }
                } else if let (Some(x), Some(y)) = (a.as_number_fast(), b.as_number_fast()) {
                    Value16::number(x + y)
                } else {
                    if let (Some(av), Some(bv)) = (a.as_string(), b.as_string()) { Value16::string(av + &bv) }
                    else if let Some(av) = a.as_string() { Value16::string(av + &vm.value_to_string_inner(&b)) }
                    else if let Some(bv) = b.as_string() { Value16::string(vm.value_to_string_inner(&a) + &bv) }
                    else { crate::vm::bigint_arith::int_add(a, b).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))? }
                };
                Ok(PackedResult::Advance)

            }
            D_INT_SUB_RR => {
                let s1 = ((arg2 >> 8) & 0xFF) as usize;
                let s2 = (arg2 & 0xFF) as usize;
                let d = arg1 as usize;
                let a = vm.registers[s1];
                let b = vm.registers[s2];
                vm.registers[d] = if let (Some(x), Some(y)) = (a.as_int_fast(), b.as_int_fast()) {
                    if let Some(v) = x.checked_sub(y) { Value16::int(v) }
                    else { crate::vm::bigint_arith::int_sub(a, b).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))? }
                } else if let (Some(x), Some(y)) = (a.as_number_fast(), b.as_number_fast()) {
                    Value16::number(x - y)
                } else {
                    crate::vm::bigint_arith::int_sub(a, b).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))?
                };
                Ok(PackedResult::Advance)

            }
            D_INT_MUL_RR => {
                let s1 = ((arg2 >> 8) & 0xFF) as usize;
                let s2 = (arg2 & 0xFF) as usize;
                let d = arg1 as usize;
                let a = vm.registers[s1];
                let b = vm.registers[s2];
                vm.registers[d] = if let (Some(x), Some(y)) = (a.as_int_fast(), b.as_int_fast()) {
                    if let Some(v) = x.checked_mul(y) { Value16::int(v) }
                    else { crate::vm::bigint_arith::int_mul(a, b).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))? }
                } else if let (Some(x), Some(y)) = (a.as_number_fast(), b.as_number_fast()) {
                    Value16::number(x * y)
                } else {
                    crate::vm::bigint_arith::int_mul(a, b).map_err(|code| crate::vm::VM::runtime_error_with_pos(&code.to_string(), bytecode, ip))?
                };
                Ok(PackedResult::Advance)

            }
            D_INT_MOD_I => {
                let s = (arg2 >> 8) as usize;
                let i = (arg2 & 0xFF) as i8 as i64;
                let d = arg1 as usize;
                let (tag, payload) = vm.registers[s].split_tag();
                vm.registers[d] = match tag {
                    ReprTag::Int => {
                        let a = payload as i64;
                        if i == 0 { return Err(crate::vm::VM::runtime_error_with_pos("Modulo by zero", bytecode, ip)); }
                        Value16::int(a % i)
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(payload);
                        if i == 0 { return Err(crate::vm::VM::runtime_error_with_pos("Modulo by zero", bytecode, ip)); }
                        Value16::number(a % i as f64)
                    }
                    _ => return Err(crate::vm::VM::runtime_error_with_pos("IntModI: src not numeric", bytecode, ip)),
                };
                Ok(PackedResult::Advance)
            }
            D_INT_CMP_LT_I | D_INT_CMP_LE_I | D_INT_CMP_EQ_I | D_INT_CMP_NE_I => {
                let s = (arg2 >> 8) as usize;
                let i = (arg2 & 0xFF) as i8 as i64;
                let d = arg1 as usize;
                let (tag, payload) = vm.registers[s].split_tag();
                let result = match tag {
                    ReprTag::Int => {
                        let a = payload as i64;
                        match dense {
                            D_INT_CMP_LT_I => a < i,
                            D_INT_CMP_LE_I => a <= i,
                            D_INT_CMP_EQ_I => a == i,
                            D_INT_CMP_NE_I => a != i,
                            _ => unreachable!(),
                        }
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(payload);
                        let b = i as f64;
                        match dense {
                            D_INT_CMP_LT_I => a < b,
                            D_INT_CMP_LE_I => a <= b,
                            D_INT_CMP_EQ_I => a == b,
                            D_INT_CMP_NE_I => a != b,
                            _ => unreachable!(),
                        }
                    }
                    _ => return Err(crate::vm::VM::runtime_error_with_pos("IntCmpI: src not numeric", bytecode, ip)),
                };
                vm.registers[d] = Value16::bool_(result);
                Ok(PackedResult::Advance)
            }

        _ => unreachable!(),
    }
}
