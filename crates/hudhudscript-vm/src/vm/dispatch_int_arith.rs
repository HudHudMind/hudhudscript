//! V2-B0: Int arithmetic packed handlers extracted from dispatch_general.rs.

use crate::vm::dense_ops::*;
use crate::vm::index_helpers::{index_i64_to_usize, numeric_index_i64};
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
                vm.registers[d] = crate::vm::math_fast_paths::do_int_add_i(a, i).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
                Ok(PackedResult::Advance)
            }
            D_INT_MUL_RI => {
                let s = (arg2 & 0xFF) as usize;
                let i = ((arg2 >> 8) as i8) as i64;
                let d = arg1 as usize;
                let a = vm.registers[s];
                vm.registers[d] = crate::vm::math_fast_paths::do_int_mul_i(a, i).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
                Ok(PackedResult::Advance)
            }
            D_INT_SUB_RI => {
                let s = (arg2 & 0xFF) as usize;
                let i = ((arg2 >> 8) as i8) as i64;
                let d = arg1 as usize;
                let a = vm.registers[s];
                vm.registers[d] = crate::vm::math_fast_paths::do_int_sub_i(a, i).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
                Ok(PackedResult::Advance)
            }
            D_NEG_R => {
                let src = arg2 as usize;
                let dst = arg1 as usize;
                let val = vm.registers[src];
                // G3.1: split_tag to distinguish Int from BigInt at same value
                let (tag, payload) = val.split_tag();
                if tag == ReprTag::Int {
                    let n = payload as i64;
                    match n.checked_neg() {
                        Some(v) => vm.registers[dst] = Value16::int(v),
                        None => vm.registers[dst] = Value16::bigint(-num_bigint::BigInt::from(n)),
                    }
                } else if let Some(b) = val.as_bigint() {
                    vm.registers[dst] = Value16::bigint(-b);
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
                // G4 fix: match unpacked Not semantics — is_truthy() for all types
                vm.registers[dst] = Value16::bool_(!vm.registers[src].is_truthy());
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
                } else if let Some(s) = obj.as_str() {
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
                            let ch = if let Some(cached) = vm.str_chars_cache.get(s) {
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
                let s = obj.as_str().ok_or_else(|| {
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
                    let ch = if let Some(cached) = vm.str_chars_cache.get(s) {
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
                        if i >= 268_435_456 {
                            return Err(crate::vm::VM::runtime_error_with_pos(
                                format!("Array index too large: {}", i), bytecode, ip));
                        }
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
                    let idx = s.find(pat).map(|i| i as i64).unwrap_or(-1);
                    vm.registers[dst] = Value16::int(idx);
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
                vm.registers[d] = crate::vm::math_fast_paths::do_int_add(a, b).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
                // GATE-2 onarımı: 56efe60f1 do_int_* imzasından &mut VM'i
                // çıkarınca promotion sayımı bu koldan düşmüştü; sayım artık
                // dispatch sitesinde (telemetry'siz build'de no-op inline).
                vm.record_bigint_promotion(a, b, vm.registers[d]);
                Ok(PackedResult::Advance)
            }
            D_INT_SUB_RR => {
                let s1 = ((arg2 >> 8) & 0xFF) as usize;
                let s2 = (arg2 & 0xFF) as usize;
                let d = arg1 as usize;
                let a = vm.registers[s1];
                let b = vm.registers[s2];
                vm.registers[d] = crate::vm::math_fast_paths::do_int_sub(a, b).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
                vm.record_bigint_promotion(a, b, vm.registers[d]);
                Ok(PackedResult::Advance)
            }
            D_INT_MUL_RR => {
                let s1 = ((arg2 >> 8) & 0xFF) as usize;
                let s2 = (arg2 & 0xFF) as usize;
                let d = arg1 as usize;
                let a = vm.registers[s1];
                let b = vm.registers[s2];
                vm.registers[d] = crate::vm::math_fast_paths::do_int_mul(a, b).map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
                vm.record_bigint_promotion(a, b, vm.registers[d]);
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
                        match a.checked_rem(i) {
                            Some(r) => Value16::int(r),
                            None => Value16::int(0),
                        }
                    }
                    ReprTag::Number => {
                        let a = f64::from_bits(payload);
                        if i == 0 { return Err(crate::vm::VM::runtime_error_with_pos("Modulo by zero", bytecode, ip)); }
                        Value16::number(a % i as f64)
                    }
                    ReprTag::Dynamic => {
                        let sv = vm.registers[s];
                        let iv = Value16::int(i);
                        match crate::vm::bigint_arith::bigint_mod(sv, iv) {
                            Ok(val) => val,
                            Err(e) => {
                                let msg = if e.0 == 399 {
                                    "IntModI: modulo by zero"
                                } else {
                                    "IntModI: src not numeric"
                                };
                                return Err(crate::vm::VM::runtime_error_with_pos(msg, bytecode, ip))
                            }
                        }
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
                    ReprTag::Dynamic => {
                        let v = vm.registers[s];
                        if let Some(a) = v.to_bigint_value() {
                            let b = num_bigint::BigInt::from(i);
                            match dense {
                                D_INT_CMP_LT_I => a < b,
                                D_INT_CMP_LE_I => a <= b,
                                D_INT_CMP_EQ_I => a == b,
                                D_INT_CMP_NE_I => a != b,
                                _ => unreachable!(),
                            }
                        } else {
                            return Err(crate::vm::VM::runtime_error_with_pos("IntCmpI: src not numeric", bytecode, ip));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::dense_ops::D_INT_MUL_RR;
    use hudhudscript_bytecode::{Bytecode, ReprTag, Value16};

    /// Prove that D_INT_MUL_RR overflow is counted as BigInt promotion.
    #[test]
    fn packed_int_mul_rr_overflow_counts_promotion() {
        let mut vm = crate::vm::VM::new();

        // Values that overflow i64 when multiplied: 3037000500 * 3037000500
        let a = Value16::int(3037000500i64);
        let b = Value16::int(3037000500i64);
        let a_reg = 0u8;
        let b_reg = 1u8;
        let d_reg = 2u8;
        vm.registers[a_reg as usize] = a;
        vm.registers[b_reg as usize] = b;

        // arg2 encodes (a_reg << 8) | b_reg
        let arg2: u16 = ((a_reg as u16) << 8) | (b_reg as u16);

        let bc = Bytecode::new();
        let result = dispatch_int_arithmetic(&mut vm, D_INT_MUL_RR, d_reg, arg2, &bc, 0);
        assert!(result.is_ok());

        let actual = vm.registers[d_reg as usize];
        assert!(
            actual.is_bigint(),
            "overflow mul must be BigInt, got {:?}",
            actual
        );

        #[cfg(feature = "telemetry")]
        {
            let snap = vm.telemetry_snapshot();
            assert!(
                snap.bigint_promotion > 0,
                "packed D_INT_MUL_RR overflow must count promotion, got {}",
                snap.bigint_promotion
            );
        }
    }
}
