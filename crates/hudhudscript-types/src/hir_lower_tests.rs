//! hir_lower unit testleri (elle kurulan AST ile — parser bağımlılığı yok).

#[cfg(test)]
mod tests {
    use crate::hir::{HirBinOp, HirExpr, HirStmt};
    use crate::hir_lower::*;
    use crate::types::Type;
    use hudhudscript_ast::{BinaryOp, Expr, Literal, Stmt};
    // Not: hudhudscript-types şu an parser'a bağımlı değil (grafiği
    // daraltmak için). Testler ayrı entegrasyon testinde (mir-opt
    // parity_with_vm.rs) gerçek parse kullanır; burada AST'yi elle
    // kurarak aynı yolu doğrularız.

    fn ident(name: &str) -> Expr {
        Expr::Identifier(name.to_string(), hudhudscript_ast::Span::default())
    }

    fn int(v: i64) -> Expr {
        Expr::Literal(Literal::Int(v), hudhudscript_ast::Span::default())
    }

    fn binary(op: BinaryOp, l: Expr, r: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(l),
            op,
            right: Box::new(r),
            span: hudhudscript_ast::Span::default(),
        }
    }

    #[test]
    fn lowers_function_with_print_add() {
        let stmts = vec![Stmt::Function {
            name: "main".into(),
            params: vec![],
            body: vec![Stmt::Expr(Expr::Call {
                callee: Box::new(ident("print")),
                args: vec![binary(BinaryOp::Add, int(2), int(3))],
                span: hudhudscript_ast::Span::default(),
            })],
            is_async: false,
            is_generator: false,
            type_params: vec![],
            span: hudhudscript_ast::Span::default(),
        }];
        let module = lower_module(&stmts).expect("must lower");
        let main = module.functions.get("main").expect("main exists");
        assert!(main.params.is_empty());
        match &main.body[0] {
            HirStmt::Expr(HirExpr::Call { callee, args, .. }) => {
                assert_eq!(callee, "print");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    HirExpr::Binary { op, lhs, rhs, ty } => {
                        assert_eq!(*op, HirBinOp::Add);
                        assert!(matches!(lhs.as_ref(), HirExpr::IntLit(2)));
                        assert!(matches!(rhs.as_ref(), HirExpr::IntLit(3)));
                        assert_eq!(*ty, Type::Number);
                    }
                    other => panic!("expected binary add, got {other:?}"),
                }
            }
            other => panic!("expected print call, got {other:?}"),
        }
    }

    #[test]
    fn params_and_identifiers_are_any_typed() {
        let f = lower_function(
            "add",
            &["a".into(), "b".into()],
            &[Stmt::Return {
                value: Some(binary(BinaryOp::Add, ident("a"), ident("b"))),
                span: hudhudscript_ast::Span::default(),
            }],
        )
        .expect("must lower");
        assert_eq!(f.params[0].name, "a");
        assert_eq!(f.params[0].ty, Type::Any);
        match &f.body[0] {
            HirStmt::Return(Some(HirExpr::Binary { ty, .. })) => assert_eq!(*ty, Type::Any),
            other => panic!("expected typed return, got {other:?}"),
        }
    }

    #[test]
    fn return_type_inference() {
        let with = lower_function("f", &[], &[Stmt::Return {
            value: Some(int(1)),
            span: hudhudscript_ast::Span::default(),
        }])
        .expect("lower");
        assert_eq!(with.return_type, Type::Number);

        let bare = lower_function("g", &[], &[Stmt::Return {
            value: None,
            span: hudhudscript_ast::Span::default(),
        }])
        .expect("lower");
        assert_eq!(bare.return_type, Type::Any);
    }

    #[test]
    fn rejects_async_function() {
        let stmts = vec![Stmt::Function {
            name: "go".into(),
            params: vec![],
            body: vec![],
            is_async: true,
            is_generator: false,
            type_params: vec![],
            span: hudhudscript_ast::Span::default(),
        }];
        match lower_module(&stmts) {
            Err(HirLowerError::Unsupported { reason, .. }) => assert!(reason.contains("async")),
            Ok(_) => panic!("async must be rejected"),
        }
    }

    #[test]
    fn rejects_bigint_literal() {
        let expr = Expr::Literal(
            Literal::BigInt("170141183460469231731687303715884105728".into()),
            hudhudscript_ast::Span::default(),
        );
        match lower_expr(&expr) {
            Err(HirLowerError::Unsupported { item, .. }) => assert!(item.contains("BigInt")),
            Ok(_) => panic!("BigInt must be rejected"),
        }
    }


}
