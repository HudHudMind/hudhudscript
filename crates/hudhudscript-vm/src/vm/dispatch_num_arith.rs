//! V2-B0: Num/float arithmetic packed handlers extracted from dispatch_general.rs.

use crate::vm::dense_ops::*;
use crate::vm::index_helpers::{numeric_index_i64, index_i64_to_usize};
use crate::vm::PackedResult;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, ReprTag, Value16};

#[rustfmt::skip]
pub(crate) fn dispatch_num_arithmetic(
    vm: &mut crate::vm::VM, dense: u8, arg1: u8, arg2: u16,
    bytecode: &Bytecode, ip: usize,
) -> CompileResult<PackedResult> {
    match dense {
            D_NUM_ADD_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = vm.registers[src1].split_tag();
                let (t2, p2) = vm.registers[src2].split_tag();
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
                vm.registers[dst] = Value16::number(a + b);
                Ok(PackedResult::Advance)
            }
            D_NUM_SUB_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = vm.registers[src1].split_tag();
                let (t2, p2) = vm.registers[src2].split_tag();
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
                vm.registers[dst] = Value16::number(a - b);
                Ok(PackedResult::Advance)
            }
            D_NUM_MUL_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = vm.registers[src1].split_tag();
                let (t2, p2) = vm.registers[src2].split_tag();
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
                vm.registers[dst] = Value16::number(a * b);
                Ok(PackedResult::Advance)
            }
            D_NUM_DIV_RR => {
                let src1 = ((arg2 >> 8) & 0xFF) as usize;
                let src2 = (arg2 & 0xFF) as usize;
                let dst = arg1 as usize;
                let (t1, p1) = vm.registers[src1].split_tag();
                let (t2, p2) = vm.registers[src2].split_tag();
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
                vm.registers[di] = if all_num {
                    if ta == ReprTag::Number || tb == ReprTag::Number || tc == ReprTag::Number {
                        let a = if ta == ReprTag::Int { pa as i64 as f64 } else { f64::from_bits(pa) };
                        let b = if tb == ReprTag::Int { pb as i64 as f64 } else { f64::from_bits(pb) };
                        let c = if tc == ReprTag::Int { pc as i64 as f64 } else { f64::from_bits(pc) };
                        Value16::number(a * b + c)
                    } else {
                        let a = pa as i64; let b = pb as i64; let c = pc as i64;
                        if let Some(prod) = a.checked_mul(b) {
                            if let Some(sum) = prod.checked_add(c) { Value16::int(sum) } else { crate::vm::bigint_arith::int_mul(od, mv).and_then(|p| crate::vm::bigint_arith::int_add(p, av)).map_err(|code| compile_codes::runtime_error(code.to_string()))? }
                        } else { crate::vm::bigint_arith::int_mul(od, mv).and_then(|p| crate::vm::bigint_arith::int_add(p, av)).map_err(|code| compile_codes::runtime_error(code.to_string()))? }
                    }
                } else {
                    crate::vm::bigint_arith::int_mul(od, mv)
                        .and_then(|p| crate::vm::bigint_arith::int_add(p, av))
                        .map_err(|code| compile_codes::runtime_error(code.to_string()))?
                };
                Ok(PackedResult::Advance)

            }
        _ => unreachable!(),
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod b2_tests {
    use crate::vm::VM;
    use crate::vm::types::decode_packed;
    use crate::vm::dense_ops::D_NUM_MUL_ADD_ASSIGN;
    use hudhudscript_bytecode::packed_instruction::pack;
    use hudhudscript_bytecode::{Instruction, Value16, Bytecode};

    fn run(dst: u8, mul: u8, add: u8, dv: Value16, mv: Value16, av: Value16) -> (Value16, Result<(), String>) {
        let mut v = VM::new();
        v.registers[dst as usize] = dv;
        v.registers[mul as usize] = mv;
        v.registers[add as usize] = av;
        let bc = Bytecode::default();
        let r = crate::vm::dispatch_num_arith::dispatch_num_arithmetic(
            &mut v, D_NUM_MUL_ADD_ASSIGN, dst, ((mul as u16) << 8) | (add as u16), &bc, 0
        );
        (v.registers[dst as usize], r.map(|_| ()).map_err(|e| format!("{}", e)))
    }

    fn expected(dv: Value16, mv: Value16, av: Value16) -> Result<Value16, String> {
        crate::vm::bigint_arith::int_mul(dv, mv)
            .and_then(|p| crate::vm::bigint_arith::int_add(p, av))
            .map_err(|e| e.to_string())
    }

    #[test] fn pack_rt() { let i=Instruction::NumMulAddAssign{dst:2,mul:3,add:4}; let p=pack(&i).unwrap(); let(o,a1,a2)=decode_packed(p); assert_eq!(o,139); assert_eq!(a1,2); assert_eq!(a2>>8&0xFF,3); assert_eq!(a2&0xFF,4); }
    #[test] fn maxr() { assert_eq!(Instruction::NumMulAddAssign{dst:10,mul:5,add:7}.max_register(),10); }
    #[test] fn iii() { let(v,_)=run(2,3,4,Value16::int(3),Value16::int(2),Value16::int(1)); assert_eq!(v, Value16::int(7)); }
    #[test] fn nni() { let(v,_)=run(2,3,4,Value16::number(3.0),Value16::number(2.0),Value16::int(1)); assert_eq!(v.as_number(),Some(7.0)); }
    #[test] fn nin() { let(v,_)=run(2,3,4,Value16::number(3.0),Value16::int(2),Value16::number(1.0)); assert_eq!(v.as_number(),Some(7.0)); }
    #[test] fn inn() { let(v,_)=run(2,3,4,Value16::int(3),Value16::number(2.0),Value16::number(1.0)); assert_eq!(v.as_number(),Some(7.0)); }
    #[test] fn a1() { let(v,_)=run(2,2,4,Value16::number(5.0),Value16::number(5.0),Value16::number(3.0)); assert_eq!(v.as_number(),Some(28.0)); }
    #[test] fn a2() { let(v,_)=run(2,3,2,Value16::number(5.0),Value16::number(2.0),Value16::number(5.0)); assert_eq!(v.as_number(),Some(15.0)); }
    #[test] fn bigint() { let bi=Value16::bigint(num_bigint::BigInt::from(1u128<<64)); let(v,_)=run(2,3,4,bi,Value16::int(2),Value16::int(1)); assert_eq!(expected(bi,Value16::int(2),Value16::int(1)).unwrap().as_bigint().unwrap().to_string(), v.as_bigint().unwrap().to_string()); }
    #[test] fn overflow() { let big=Value16::int(1i64<<62); let(v,_)=run(2,3,4,big,Value16::int(2),Value16::int(1)); assert!(v.is_bigint()); }
    #[test] fn bi_num_rej() { let bi=Value16::bigint(num_bigint::BigInt::from(1u128<<64)); let(_,r)=run(2,3,4,bi,Value16::number(2.0),Value16::int(1)); assert!(r.is_err()); }
    #[test] fn str_rej() { let(_,r)=run(2,3,4,Value16::string("x"),Value16::number(2.0),Value16::number(1.0)); assert!(r.is_err()); }
}
