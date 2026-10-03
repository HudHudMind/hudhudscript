//! Typed HIR shape tests — HirModule/HirFunction/HirExpr structure, `ty()`
//! source-type reporting, and HirBinOp Display.

use hudhudscript_types::{HirBinOp, HirExpr, HirFunction, HirModule, HirStmt, Type};

// ── example_add ───────────────────────────────────────────────────

#[test]
fn example_add_shape() {
    let f = HirFunction::example_add();
    assert_eq!(f.name, "add");
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0].name, "a");
    assert_eq!(f.params[0].ty, Type::Number);
    assert_eq!(f.return_type, Type::Number);
    assert_eq!(f.body.len(), 1);
    match &f.body[0] {
        HirStmt::Return(Some(HirExpr::Binary { op, lhs, rhs, ty })) => {
            assert_eq!(*op, HirBinOp::Add);
            assert!(matches!(lhs.as_ref(), HirExpr::Local { name, .. } if name == "a"));
            assert!(matches!(rhs.as_ref(), HirExpr::Local { name, .. } if name == "b"));
            assert_eq!(*ty, Type::Number);
        }
        other => panic!("expected return of binary add, got {other:?}"),
    }
}

// ── expr ty ───────────────────────────────────────────────────────

#[test]
fn expr_ty_reports_source_type() {
    assert_eq!(HirExpr::IntLit(3).ty(), Type::Number);
    assert_eq!(HirExpr::FloatLit(1.5).ty(), Type::Number);
    assert_eq!(HirExpr::BoolLit(true).ty(), Type::Boolean);
    assert_eq!(HirExpr::StringLit("x".into()).ty(), Type::String);
    assert_eq!(HirExpr::NullLit.ty(), Type::Null);
}

// ── module determinism ────────────────────────────────────────────

#[test]
fn module_holds_functions_deterministically() {
    let mut m = HirModule::default();
    let add = HirFunction::example_add();
    m.functions.insert("add".into(), add.clone());
    m.functions.insert("main".into(), HirFunction {
        name: "main".into(),
        params: vec![],
        return_type: Type::Null,
        body: vec![HirStmt::Expr(HirExpr::Call {
            callee: "print".into(),
            args: vec![HirExpr::IntLit(5)],
            ty: Type::Null,
        })],
    });
    let names: Vec<&str> = m.functions.keys().map(|s| s.as_str()).collect();
    assert_eq!(names, ["add", "main"]); // BTreeMap → alfabetik
    assert!(m.functions.contains_key("add"));
    assert_eq!(m.functions["main"].body.len(), 1);
}

// ── bin op display ────────────────────────────────────────────────

#[test]
fn bin_op_display() {
    assert_eq!(HirBinOp::Add.to_string(), "+");
    assert_eq!(HirBinOp::Le.to_string(), "<=");
    assert_eq!(HirBinOp::And.to_string(), "&&");
}
