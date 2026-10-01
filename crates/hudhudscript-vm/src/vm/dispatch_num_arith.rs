//! V2-B0: Num/float arithmetic packed handlers extracted from dispatch_general.rs.

use crate::vm::dense_ops::*;
use crate::vm::index_helpers::{index_i64_to_usize, numeric_index_i64};
use crate::vm::PackedResult;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, ReprTag, Value16};

/// Test-only: dış birim test paketi `num_bigint`'i doğrudan dependency olarak
/// listeyemediği için buradan yeniden dışa aktarılır.
#[doc(hidden)]
pub use num_bigint;

#[rustfmt::skip]
#[doc(hidden)]
#[inline(always)]
pub fn dispatch_num_arithmetic(
    vm: &mut crate::vm::VM, dense: u8, arg1: u8, arg2: u16,
    bytecode: &Bytecode, ip: usize,
) -> CompileResult<PackedResult> {
    let is_dynamic = |vm: &crate::vm::VM, reg: usize| vm.registers[reg].split_tag().0 == ReprTag::Dynamic;
    match dense {
            D_NUM_ADD_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = vm.registers[src1].split_tag();
                let (t2, p2) = vm.registers[src2].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
                let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                vm.registers[dst] = Value16::number(a + b);
                Ok(PackedResult::Advance)
            }
            D_NUM_SUB_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = vm.registers[src1].split_tag();
                let (t2, p2) = vm.registers[src2].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
                let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                vm.registers[dst] = Value16::number(a - b);
                Ok(PackedResult::Advance)
            }
            D_NUM_MUL_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = vm.registers[src1].split_tag();
                let (t2, p2) = vm.registers[src2].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
                let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                vm.registers[dst] = Value16::number(a * b);
                Ok(PackedResult::Advance)
            }
            D_NUM_DIV_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = vm.registers[src1].split_tag();
                let (t2, p2) = vm.registers[src2].split_tag();
                if t1 == ReprTag::Dynamic || t2 == ReprTag::Dynamic {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        "Cannot mix BigInt and Number",
                        bytecode,
                        ip,
                    ));
                }
                let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                if b == 0.0 {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        "NumDiv: division by zero",
                        bytecode,
                        ip,
                    ));
                }
                vm.registers[dst] = Value16::number(a / b);
                Ok(PackedResult::Advance)
            }
            // Array index assignment — packed fast path
            D_INDEX_ASSIGN_RRR => {
                let obj_reg = arg1 as usize;
                let idx_reg = ((arg2 >> 8) & 0xFF) as usize;
                let val_reg = (arg2 & 0xFF) as usize;
                let idx = vm.registers[idx_reg];
                let val = vm.registers[val_reg];
                if let Some(i) = numeric_index_i64(idx).and_then(index_i64_to_usize) {
                    if let Some(arr) = vm.registers[obj_reg].as_array_mut() {
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
                let (tag, payload) = vm.registers[src].split_tag();
                vm.registers[dst] = match tag {
                    ReprTag::Int => Value16::number(payload as i64 as f64 + imm),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) + imm),
                    _ => {
                        return Err(crate::vm::VM::runtime_error_with_pos(
                            "NumAddI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                Ok(PackedResult::Advance)
            }
            D_NUM_SUB_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as f64;
                let dst = arg1 as usize;
                let (tag, payload) = vm.registers[src].split_tag();
                vm.registers[dst] = match tag {
                    ReprTag::Int => Value16::number(payload as i64 as f64 - imm),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) - imm),
                    _ => {
                        return Err(crate::vm::VM::runtime_error_with_pos(
                            "NumSubI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                Ok(PackedResult::Advance)
            }
            D_NUM_MUL_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as f64;
                let dst = arg1 as usize;
                let (tag, payload) = vm.registers[src].split_tag();
                vm.registers[dst] = match tag {
                    ReprTag::Int => Value16::number(payload as i64 as f64 * imm),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) * imm),
                    _ => {
                        return Err(crate::vm::VM::runtime_error_with_pos(
                            "NumMulI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                Ok(PackedResult::Advance)
            }
            D_NUM_DIV_RI => {
                let src = (arg2 & 0xFF) as usize;
                let imm = ((arg2 >> 8) as i8) as f64;
                let dst = arg1 as usize;
                if imm == 0.0 {
                    return Err(crate::vm::VM::runtime_error_with_pos(
                        "NumDivI: division by zero",
                        bytecode,
                        ip,
                    ));
                }
                let (tag, payload) = vm.registers[src].split_tag();
                vm.registers[dst] = match tag {
                    ReprTag::Int => Value16::number(payload as i64 as f64 / imm),
                    ReprTag::Number => Value16::number(f64::from_bits(payload) / imm),
                    _ => {
                        return Err(crate::vm::VM::runtime_error_with_pos(
                            "NumDivI: src not numeric",
                            bytecode,
                            ip,
                        ))
                    }
                };
                Ok(PackedResult::Advance)
            }
            D_STR_REV_R => {
                let src = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let s = vm.registers[src].as_string().unwrap_or_default();
                let rev: String = s.chars().rev().collect();
                vm.registers[dst] = Value16::string(rev);
                Ok(PackedResult::Advance)
            }
            D_NUM_MUL_ADD_ASSIGN => {
                let di = arg1 as usize;
                let mi = ((arg2 >> 8) & 0xFF) as usize;
                let ai = (arg2 & 0xFF) as usize;
                let od = vm.registers[di];
                let mv = vm.registers[mi];
                let av = vm.registers[ai];
                let (ta, pa) = od.split_tag();
                let (tb, pb) = mv.split_tag();
                let (tc, pc) = av.split_tag();
                let all_num = (ta == ReprTag::Int || ta == ReprTag::Number) && (tb == ReprTag::Int || tb == ReprTag::Number) && (tc == ReprTag::Int || tc == ReprTag::Number);
                let result = if all_num {
                    if ta == ReprTag::Number || tb == ReprTag::Number || tc == ReprTag::Number {
                        let a = if ta == ReprTag::Int { pa as i64 as f64 } else { f64::from_bits(pa) };
                        let b = if tb == ReprTag::Int { pb as i64 as f64 } else { f64::from_bits(pb) };
                        let c = if tc == ReprTag::Int { pc as i64 as f64 } else { f64::from_bits(pc) };
                        Value16::number(a * b + c)
                    } else {
                        let a = pa as i64; let b = pb as i64; let c = pc as i64;
                        if let Some(prod) = a.checked_mul(b) {
                            if let Some(sum) = prod.checked_add(c) {
                                Value16::int(sum)
                            } else {
                                let product = crate::vm::bigint_arith::int_mul(od, mv)
                                    .map_err(|code| compile_codes::runtime_error(code.to_string()))?;
                                vm.record_bigint_promotion(od, mv, product);
                                let sum = crate::vm::bigint_arith::int_add(product, av)
                                    .map_err(|code| compile_codes::runtime_error(code.to_string()))?;
                                vm.record_bigint_promotion(product, av, sum);
                                sum
                            }
                        } else {
                            let product = crate::vm::bigint_arith::int_mul(od, mv)
                                .map_err(|code| compile_codes::runtime_error(code.to_string()))?;
                            vm.record_bigint_promotion(od, mv, product);
                            let sum = crate::vm::bigint_arith::int_add(product, av)
                                .map_err(|code| compile_codes::runtime_error(code.to_string()))?;
                            vm.record_bigint_promotion(product, av, sum);
                            sum
                        }
                    }
                } else {
                    let product = crate::vm::bigint_arith::int_mul(od, mv)
                        .map_err(|code| compile_codes::runtime_error(code.to_string()))?;
                    vm.record_bigint_promotion(od, mv, product);
                    crate::vm::bigint_arith::int_add(product, av)
                        .map_err(|code| compile_codes::runtime_error(code.to_string()))?
                };
                vm.registers[di] = result;
                Ok(PackedResult::Advance)
            }
        _ => unreachable!(),
    }
}
