//! Tests for MirFunctionBuilder.

use crate::builder::{BinOp, MirFunctionBuilder};
use crate::mir::{MirFunction, MirType, ValueId};
use crate::verify::verify_function;

fn build_add() -> MirFunction {
    let mut b = MirFunctionBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);
    let e = b.entry();
    let a = b.const_i64(e, 0);
    let c = b.const_i64(e, 0);
    let sum = b.bin(e, BinOp::Add, MirType::I64, a, c);
    b.ret(e, sum);
    b.finish()
}

#[test]
fn builds_and_verifies_add() {
    let f = build_add();
    assert_eq!(f.blocks.len(), 1);
    assert_eq!(f.name.as_ref(), "add");
    assert_eq!(f.return_ty, MirType::I64);
    verify_function(&f).expect("add must verify");
}

#[test]
fn fresh_values_increase() {
    let mut b = MirFunctionBuilder::new("f", vec![], MirType::Unit);
    let e = b.entry();
    let v0 = b.const_i64(e, 1);
    let v1 = b.const_i64(e, 2);
    let v2 = b.bin(e, BinOp::Add, MirType::I64, v0, v1);
    assert_eq!(v0, ValueId(0));
    assert_eq!(v1, ValueId(1));
    assert_eq!(v2, ValueId(2));
}

#[test]
#[should_panic(expected = "already terminated")]
fn double_termination_panics() {
    let mut b = MirFunctionBuilder::new("f", vec![], MirType::I64);
    let e = b.entry();
    let z = b.const_i64(e, 0);
    b.ret(e, z);
    b.ret(e, z);
}
