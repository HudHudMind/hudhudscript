//! Tests for hudhudscript-mir `lower` — AŞAMA-0 untyped lowering lane.

use hudhudscript_mir::lower::{lower_function, LowerError};
use hudhudscript_types::{HirBinOp, HirExpr, HirFunction, HirParam, HirStmt};
use hudhudscript_types::Type;

fn main_print_2_3() -> HirFunction {
    HirFunction {
        name: "main".into(),
        params: vec![],
        return_type: Type::Null,
        body: vec![HirStmt::Expr(HirExpr::Call {
            callee: "print".into(),
            args: vec![HirExpr::Binary {
                op: HirBinOp::Add,
                lhs: Box::new(HirExpr::IntLit(2)),
                rhs: Box::new(HirExpr::IntLit(3)),
                ty: Type::Number,
            }],
            ty: Type::Null,
        })],
    }
}

#[test]
fn lowers_print_2_plus_3() {
    let mir = lower_function(&main_print_2_3()).expect("must lower");
    let text = hudhudscript_mir::print::render_function(&mir);
    assert!(text.contains("v0 = const.i64 2"), "{text}");
    assert!(text.contains("v1 = const.i64 3"), "{text}");
    assert!(text.contains("v2 = i64.add v0, v1"), "{text}");
    assert!(text.contains("native hudhud_print(v2) -> unit"), "{text}");
    assert!(text.contains("gc.safepoint"), "{text}");
}

#[test]
fn rejects_unsupported_cleanly() {
    let hir = HirFunction {
        name: "f".into(),
        params: vec![HirParam { name: "a".into(), ty: Type::Number }],
        return_type: Type::Number,
        body: vec![HirStmt::Return(Some(HirExpr::Local {
            name: "a".into(),
            ty: Type::Number,
        }))],
    };
    match lower_function(&hir) {
        Err(LowerError::Unsupported { function, reason }) => {
            assert_eq!(function, "f");
            assert!(reason.contains("locals"), "{reason}");
        }
        Ok(_) => panic!("locals must be rejected in AŞAMA-0"),
    }
}
