//! Tests for hudhudscript-mir `print` — MIR text rendering.

use hudhudscript_mir::builder::{BinOp, MirFunctionBuilder};
use hudhudscript_mir::mir::MirType;
use hudhudscript_mir::print::render_function;
use hudhudscript_mir::types::RuntimeHelperId;

#[test]
fn renders_add_example() {
    let mut b = MirFunctionBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);
    let e = b.entry();
    let x = b.const_i64(e, 2);
    let y = b.const_i64(e, 3);
    let s = b.bin(e, BinOp::Add, MirType::I64, x, y);
    b.ret(e, s);
    let text = render_function(&b.finish());
    assert!(text.contains("fn @add (i64, i64) -> i64 {"), "header: {text}");
    assert!(text.contains("v0 = const.i64 2"), "{text}");
    assert!(text.contains("v2 = i64.add v0, v1"), "{text}");
    assert!(text.contains("return v2"), "{text}");
}

#[test]
fn renders_native_call_and_safepoint() {
    let mut b = MirFunctionBuilder::new("main", vec![], MirType::Unit);
    let e = b.entry();
    let five = b.const_i64(e, 5);
    b.call_native(e, MirType::Unit, RuntimeHelperId::Print, vec![five]);
    b.gc_safepoint(e);
    b.ret(e, five); // Unit dönüş henüz desteklenmiyor — Return hedefi
    let text = render_function(&b.finish());
    assert!(text.contains("native hudhud_print(v0) -> unit"), "{text}");
    assert!(text.contains("gc.safepoint"), "{text}");
}
