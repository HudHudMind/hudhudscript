//! Tests for hudhudscript-mir-opt `const_fold` and the optimize pipeline.

use hudhudscript_mir::builder::{BinOp, MirFunctionBuilder};
use hudhudscript_mir::{MirFunction, MirInst, MirType};
use hudhudscript_mir_opt::const_fold::const_fold;

// ── const_fold ────────────────────────────────────────────────────

fn build(ty: MirType, op: BinOp, a: i64, rhs: i64) -> MirFunction {
    let mut b = MirFunctionBuilder::new("f", vec![], MirType::I64);
    let e = b.entry();
    let x = b.const_i64(e, a);
    let y = b.const_i64(e, rhs);
    let s = b.bin(e, op, ty, x, y);
    b.ret(e, s);
    let f = b.finish();
    hudhudscript_mir::verify_function(&f).expect("built fn must verify");
    f
}

#[test]
fn folds_add_sub_mul() {
    for (op, a, b, expect) in [
        (BinOp::Add, 2, 3, 5i64),
        (BinOp::Add, -7, 7, 0),
        (BinOp::Sub, 10, 4, 6),
        (BinOp::Mul, 6, 7, 42),
    ] {
        let f = build(MirType::I64, op, a, b);
        let (folded_f, n) = const_fold(&f);
        assert_eq!(n, 1, "{op:?}");
        hudhudscript_mir::verify_function(&folded_f).expect("folded must verify");
        match &folded_f.blocks[0].insts[2] {
            MirInst::ConstInt { value, .. } => assert_eq!(*value, expect),
            other => panic!("{op:?} not folded: {other:?}"),
        }
    }
}

#[test]
fn never_folds_overflow() {
    let f = build(MirType::I64, BinOp::Add, i64::MAX, 1);
    let (folded_f, n) = const_fold(&f);
    assert_eq!(n, 0);
    assert!(matches!(folded_f.blocks[0].insts[2], MirInst::Add { .. }));
    hudhudscript_mir::verify_function(&folded_f).expect("must still verify");
}

#[test]
fn never_folds_div_rem() {
    let (f_div, n1) = const_fold(&build(MirType::I64, BinOp::Div, 10, 2));
    let (f_rem, n2) = const_fold(&build(MirType::I64, BinOp::Rem, 10, 3));
    assert_eq!(n1, 0);
    assert_eq!(n2, 0);
    assert!(matches!(f_div.blocks[0].insts[2], MirInst::Div { .. }));
    assert!(matches!(f_rem.blocks[0].insts[2], MirInst::Rem { .. }));
}

#[test]
fn folds_f64_add() {
    let mut b = MirFunctionBuilder::new("f", vec![], MirType::F64);
    let e = b.entry();
    let x = b.const_f64(e, 1.5);
    let y = b.const_f64(e, 2.25);
    b.bin(e, BinOp::Add, MirType::F64, x, y);
    let f = b.finish();
    let (folded_f, n) = const_fold(&f);
    assert_eq!(n, 1);
    match &folded_f.blocks[0].insts[2] {
        MirInst::ConstFloat { bits, .. } => assert_eq!(f64::from_bits(*bits), 3.75),
        other => panic!("f64 add not folded: {other:?}"),
    }
}

// ── optimize pipeline (§18 contracts) ─────────────────────────────

fn mk() -> MirFunctionBuilder {
    MirFunctionBuilder::new("t", vec![], MirType::I64)
}

#[test]
fn optimize_collapses_const_chain() {
    // v0=v(0); v1=v0+1; v2=v1+1; ret v2 → tek sabit 2
    let mut b = mk();
    let e = b.entry();
    let z = b.const_i64(e, 0);
    let o = b.const_i64(e, 1);
    let a1 = b.bin(e, hudhudscript_mir::builder::BinOp::Add, MirType::I64, z, o);
    let a2 = b.bin(e, hudhudscript_mir::builder::BinOp::Add, MirType::I64, a1, o);
    b.ret(e, a2);
    let f = b.finish();
    let (opt, _) = hudhudscript_mir_opt::optimize(&f);
    // 3 saf tanımdan (z,o,a1) a1 ve z,o ölü olmalı; kalan: 2 sabit + ret
    let inst_count: usize = opt.blocks.iter().map(|bl| bl.insts.len()).sum();
    assert!(inst_count <= 2, "zincir katlanamadi: {inst_count} inst");
}

#[test]
fn overflow_never_folded() {
    // §18: i64::MAX + 1 katlanamaz (runtime overflow lane)
    let mut b = mk();
    let e = b.entry();
    let mx = b.const_i64(e, i64::MAX);
    let o = b.const_i64(e, 1);
    let a = b.bin(e, hudhudscript_mir::builder::BinOp::Add, MirType::I64, mx, o);
    b.ret(e, a);
    let f = b.finish();
    let (opt, _) = const_fold(&f);
    let still_add = opt.blocks.iter().any(|bl| bl.insts.iter()
        .any(|i| matches!(i, MirInst::Add { .. })));
    assert!(still_add, "§18 ihlali: overflow katlandi!");
}

#[test]
fn div_zero_never_folded() {
    let mut b = mk();
    let e = b.entry();
    let n = b.const_i64(e, 10);
    let z = b.const_i64(e, 0);
    let d = b.bin(e, hudhudscript_mir::builder::BinOp::Div, MirType::I64, n, z);
    b.ret(e, d);
    let f = b.finish();
    let (opt, _) = const_fold(&f);
    let still_div = opt.blocks.iter().any(|bl| bl.insts.iter()
        .any(|i| matches!(i, MirInst::Div { .. })));
    assert!(still_div, "§18 ihlali: sifira bolme katlandi!");
}
