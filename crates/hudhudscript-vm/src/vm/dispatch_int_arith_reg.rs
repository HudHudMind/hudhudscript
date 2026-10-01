//! V2-B0: Register-based packed int-op handlers split from dispatch_int_arith.rs.

use crate::vm::dense_ops::*;
use crate::vm::PackedResult;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, ReprTag, Value16};

#[rustfmt::skip]
#[doc(hidden)]
#[inline(always)]
pub(super) fn dispatch_int_arith_reg(
    vm: &mut crate::vm::VM, dense: u8, arg1: u8, arg2: u16,
    bytecode: &Bytecode, ip: usize,
) -> CompileResult<PackedResult> {
    match dense {
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
