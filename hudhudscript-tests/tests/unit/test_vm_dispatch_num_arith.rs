//! VM dispatch: packed `NumMulAddAssign` fast path (V2-B0 / b2) — packing,
//! register-range, Int/Number mixes, and BigInt overflow promotion of
//! `dispatch_num_arithmetic()`.

use hudhudscript_vm::vm::bigint_arith::{int_add, int_mul};
use hudhudscript_vm::vm::dense_ops::D_NUM_MUL_ADD_ASSIGN;
use hudhudscript_vm::vm::dispatch_num_arith::{dispatch_num_arithmetic, num_bigint};
use hudhudscript_vm::vm::types::decode_packed;
use hudhudscript_vm::vm::VM;
use hudhudscript_bytecode::packed_instruction::pack;
use hudhudscript_bytecode::{Bytecode, Instruction, Value16};

// ── helpers ──────────────────────────────────────────────────────────────────

fn run(dst: u8, mul: u8, add: u8, dv: Value16, mv: Value16, av: Value16) -> (Value16, Result<(), String>) {
    let mut v = VM::new();
    v.registers[dst as usize] = dv;
    v.registers[mul as usize] = mv;
    v.registers[add as usize] = av;
    let bc = Bytecode::default();
    let r = dispatch_num_arithmetic(
        &mut v, D_NUM_MUL_ADD_ASSIGN, dst, ((mul as u16) << 8) | (add as u16), &bc, 0
    );
    (v.registers[dst as usize], r.map(|_| ()).map_err(|e| format!("{}", e)))
}

fn expected(dv: Value16, mv: Value16, av: Value16) -> Result<Value16, String> {
    int_mul(dv, mv)
        .and_then(|p| int_add(p, av))
        .map_err(|e| e.to_string())
}

// ── packing / register range ─────────────────────────────────────────────────

#[test] fn pack_rt() { let i=Instruction::NumMulAddAssign{dst:2,mul:3,add:4}; let p=pack(&i).unwrap(); let(o,a1,a2)=decode_packed(p); assert_eq!(o,139); assert_eq!(a1,2); assert_eq!(a2>>8&0xFF,3); assert_eq!(a2&0xFF,4); }
#[test] fn maxr() { assert_eq!(Instruction::NumMulAddAssign{dst:10,mul:5,add:7}.max_register(),10); }

// ── Int/Number operand mixes ─────────────────────────────────────────────────

#[test] fn iii() { let(v,_)=run(2,3,4,Value16::int(3),Value16::int(2),Value16::int(1)); assert_eq!(v, Value16::int(7)); }
#[test] fn nni() { let(v,_)=run(2,3,4,Value16::number(3.0),Value16::number(2.0),Value16::int(1)); assert_eq!(v.as_number(),Some(7.0)); }
#[test] fn nin() { let(v,_)=run(2,3,4,Value16::number(3.0),Value16::int(2),Value16::number(1.0)); assert_eq!(v.as_number(),Some(7.0)); }
#[test] fn inn() { let(v,_)=run(2,3,4,Value16::int(3),Value16::number(2.0),Value16::number(1.0)); assert_eq!(v.as_number(),Some(7.0)); }
#[test] fn a1() { let(v,_)=run(2,2,4,Value16::number(5.0),Value16::number(5.0),Value16::number(3.0)); assert_eq!(v.as_number(),Some(28.0)); }
#[test] fn a2() { let(v,_)=run(2,3,2,Value16::number(5.0),Value16::number(2.0),Value16::number(5.0)); assert_eq!(v.as_number(),Some(15.0)); }

// ── BigInt / overflow / type rejects ─────────────────────────────────────────

#[test] fn bigint() { let bi=Value16::bigint(num_bigint::BigInt::from(1u128<<64)); let(v,_)=run(2,3,4,bi,Value16::int(2),Value16::int(1)); assert_eq!(expected(bi,Value16::int(2),Value16::int(1)).unwrap().as_bigint().unwrap().to_string(), v.as_bigint().unwrap().to_string()); }
#[test] fn overflow() { let big=Value16::int(1i64<<62); let(v,_)=run(2,3,4,big,Value16::int(2),Value16::int(1)); assert!(v.is_bigint()); }
#[test] fn bi_num_rej() { let bi=Value16::bigint(num_bigint::BigInt::from(1u128<<64)); let(_,r)=run(2,3,4,bi,Value16::number(2.0),Value16::int(1)); assert!(r.is_err()); }
#[test] fn str_rej() { let(_,r)=run(2,3,4,Value16::string("x"),Value16::number(2.0),Value16::number(1.0)); assert!(r.is_err()); }
