//! lower_typed unit testleri.

use super::*;
use crate::print::render_function;
use hudhudscript_types::{HirBinOp, HirExpr, HirFunction, HirParam, HirStmt, Type};

    fn add_hir() -> HirFunction {
        HirFunction {
            name: "add".into(),
            params: vec![
                HirParam { name: "a".into(), ty: Type::Any },
                HirParam { name: "b".into(), ty: Type::Any },
            ],
            return_type: Type::Number,
            body: vec![HirStmt::Return(Some(HirExpr::Binary {
                op: HirBinOp::Add,
                lhs: Box::new(HirExpr::Local { name: "a".into(), ty: Type::Any }),
                rhs: Box::new(HirExpr::Local { name: "b".into(), ty: Type::Any }),
                ty: Type::Any,
            }))],
        }
    }

    fn tys() -> HashMap<String, MirType> {
        HashMap::from([("a".to_string(), MirType::I64), ("b".to_string(), MirType::I64)])
    }

    #[test]
    fn lowers_typed_add_with_params() {
        let mir = lower_function_typed(&add_hir(), &tys()).expect("lower");
        let text = render_function(&mir);
        assert!(text.contains("fn @add (i64, i64) -> i64"), "{text}");
        assert!(text.contains("v0 = param0 : i64"), "{text}");
        assert!(text.contains("v1 = param1 : i64"), "{text}");
        assert!(text.contains("v2 = i64.add v0, v1"), "{text}");
        assert!(text.contains("return v2"), "{text}");
    }

    #[test]
    fn lowers_local_bindings() {
        use hudhudscript_types::HirBinOp;
        let hir = HirFunction {
            name: "compute".into(),
            params: vec![HirParam { name: "a".into(), ty: Type::Any }],
            return_type: Type::Number,
            body: vec![
                HirStmt::Let {
                    name: "x".into(),
                    ty: Type::Any,
                    value: HirExpr::Binary {
                        op: HirBinOp::Add,
                        lhs: Box::new(HirExpr::Local { name: "a".into(), ty: Type::Any }),
                        rhs: Box::new(HirExpr::IntLit(10)),
                        ty: Type::Number,
                    },
                },
                HirStmt::Let {
                    name: "y".into(),
                    ty: Type::Any,
                    value: HirExpr::Binary {
                        op: HirBinOp::Mul,
                        lhs: Box::new(HirExpr::Local { name: "x".into(), ty: Type::Any }),
                        rhs: Box::new(HirExpr::IntLit(2)),
                        ty: Type::Number,
                    },
                },
                HirStmt::Return(Some(HirExpr::Local { name: "y".into(), ty: Type::Any })),
            ],
        };
        let types = HashMap::from([("a".to_string(), MirType::I64)]);
        let mir = lower_function_typed(&hir, &types).expect("lower");
        let text = crate::print::render_function(&mir);
        // (a + 10) * 2 — SSA zinciri
        assert!(text.contains("v0 = param0 : i64"), "{text}");
        assert!(text.contains("v1 = const.i64 10"), "{text}");
        assert!(text.contains("v2 = i64.add v0, v1"), "{text}");
        assert!(text.contains("v3 = const.i64 2"), "{text}");
        assert!(text.contains("v4 = i64.mul v2, v3"), "{text}");
        assert!(text.contains("return v4"), "{text}");
    }

    #[test]
    fn lowers_assign_rebinding() {
        use hudhudscript_types::HirBinOp;
        let hir = HirFunction {
            name: "rebind".into(),
            params: vec![],
            return_type: Type::Number,
            body: vec![
                HirStmt::Let {
                    name: "x".into(),
                    ty: Type::Any,
                    value: HirExpr::IntLit(1),
                },
                HirStmt::Assign {
                    name: "x".into(),
                    value: HirExpr::Binary {
                        op: HirBinOp::Add,
                        lhs: Box::new(HirExpr::Local { name: "x".into(), ty: Type::Any }),
                        rhs: Box::new(HirExpr::IntLit(1)),
                        ty: Type::Number,
                    },
                },
                HirStmt::Return(Some(HirExpr::Local { name: "x".into(), ty: Type::Any })),
            ],
        };
        let mir = lower_function_typed(&hir, &HashMap::new()).expect("lower");
        let text = crate::print::render_function(&mir);
        // x=1; x=x+1; return x → v0=1, v1=1, v2=add(v0,v1), return v2
        assert!(text.contains("v2 = i64.add v0, v1"), "{text}");
        assert!(text.contains("return v2"), "{text}");
    }

    #[test]
    fn unbound_name_rejected() {
        let hir = HirFunction {
            name: "f".into(),
            params: vec![],
            return_type: Type::Number,
            body: vec![HirStmt::Return(Some(HirExpr::Local {
                name: "ghost".into(),
                ty: Type::Any,
            }))],
        };
        let err = lower_function_typed(&hir, &HashMap::new()).unwrap_err();
        assert!(err.to_string().contains("unbound name `ghost`"), "{err}");
    }

    #[test]
    fn missing_param_type_rejected() {
        let mut t = tys();
        t.remove("b");
        let err = lower_function_typed(&add_hir(), &t).unwrap_err();
        assert!(err.to_string().contains("param `b`"), "{err}");
    }

    #[test]
    fn non_numeric_param_rejected() {
        let t = HashMap::from([("a".to_string(), MirType::Bool), ("b".to_string(), MirType::I64)]);
        let err = lower_function_typed(&add_hir(), &t).unwrap_err();
        assert!(err.to_string().contains("i64 and f64 params"), "{err}");
    }
