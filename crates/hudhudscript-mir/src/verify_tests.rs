use super::builder::{BinOp, MirFunctionBuilder};
use super::mir::{CmpOp, MirFunction, MirInst, MirTerminator, MirType, ValueId};
use super::verify::{verify_function, VerifyError, VerifyErrorKind};

fn ok_add() -> MirFunction {
    let mut b = MirFunctionBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);
    let e = b.entry();
    let x = b.const_i64(e, 2);
    let y = b.const_i64(e, 3);
    let s = b.bin(e, BinOp::Add, MirType::I64, x, y);
    b.ret(e, s);
    b.finish()
}

#[test]
fn accepts_valid_function() {
    assert!(verify_function(&ok_add()).is_ok());
}

#[test]
fn rejects_missing_terminator() {
    let mut b = MirFunctionBuilder::new("f", vec![], MirType::I64);
    let e = b.entry();
    let x = b.const_i64(e, 1);
    b.bin(e, BinOp::Add, MirType::I64, x, x); // not terminated
    let f = b.finish();
    let blocks = f.blocks.clone();
    let _ = blocks;
    match verify_function(&f) {
        Err(VerifyError { kind: VerifyErrorKind::MissingTerminator { .. }, .. }) => {}
        other => panic!("expected MissingTerminator, got {other:?}"),
    }
}

#[test]
fn rejects_unknown_value_use() {
    let mut b = MirFunctionBuilder::new("f", vec![], MirType::I64);
    let e = b.entry();
    let ghost = ValueId(99);
    let x = b.const_i64(e, 1);
    let mut f = b.finish();
    f.blocks[0].insts.push(MirInst::Add {
        dst: ValueId(2),
        ty: MirType::I64,
        lhs: x,
        rhs: ghost,
    });
    match verify_function(&f) {
        Err(VerifyError { kind: VerifyErrorKind::UnknownValue { value: 99 }, .. }) => {}
        other => panic!("expected UnknownValue, got {other:?}"),
    }
}

#[test]
fn rejects_unknown_branch_target() {
    let mut b = MirFunctionBuilder::new("f", vec![], MirType::I64);
    let e = b.entry();
    let x = b.const_i64(e, 1);
    b.ret(e, x);
    let mut f = b.finish();
    f.blocks[0].terminator = Some(MirTerminator::Branch { target: crate::mir::BlockId(7), args: Vec::new() });
    match verify_function(&f) {
        Err(VerifyError { kind: VerifyErrorKind::UnknownBlock { target: 7 }, .. }) => {}
        other => panic!("expected UnknownBlock, got {other:?}"),
    }
}
