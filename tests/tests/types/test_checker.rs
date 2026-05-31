//! External tests for hudhudscript-types — TypeChecker, SymbolTable, SymbolInfo.

use hudhudscript_ast::Span;
use hudhudscript_ast::{
    ArrowFunctionBody, BinaryOp, CatchClause, Expr, Literal, OwnershipMode, Position, Stmt,
    SwitchCase, TypeAnnotation, UnaryOp, VarDecl,
};
use hudhudscript_types::{Ownership, SymbolInfo, SymbolTable, Type, TypeChecker};

// ── helpers ──────────────────────────────────────────────────────────────────

fn dummy_span() -> Span {
    let pos = Position::new(1, 1, 0);
    Span::new(pos, pos)
}

fn num_expr(n: f64) -> Expr {
    Expr::Literal(Literal::Number(n, false), dummy_span())
}

fn str_expr(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.to_string()), dummy_span())
}

fn ident(name: &str) -> Expr {
    Expr::Identifier(name.to_string(), dummy_span())
}

// ── basic type checker tests ──────────────────────────────────────────────────

#[test]
fn test_checker_let_defines_variable() {
    let mut checker = TypeChecker::new();
    // `let x = 42` should succeed and define x as Number
    let stmt = Stmt::Let {
        name: "x".to_string(),
        value: num_expr(42.0),
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
    // Now x should be accessible
    let expr = ident("x");
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Number);
}

#[test]
fn test_checker_const_defines_variable() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::Const {
        name: "PI".to_string(),
        value: num_expr(3.14),
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
    let ty = checker.check_expr(&ident("PI")).unwrap();
    assert_eq!(ty, Type::Number);
}

#[test]
fn test_checker_block_scope_isolation() {
    let mut checker = TypeChecker::new();
    // Define x inside a block; it should not be visible outside
    let block = Stmt::Block {
        statements: vec![Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        }],
        span: dummy_span(),
    };
    checker.check_stmt(&block).unwrap();
    // x must NOT be defined in the outer scope now
    assert!(checker.check_expr(&ident("x")).is_err());
}

#[test]
fn test_checker_for_loop_variable_in_scope() {
    let mut checker = TypeChecker::new();

    // for (item in [1, 2, 3]) { }
    let stmt = Stmt::For {
        variable: "item".to_string(),
        iterable: Expr::Array {
            elements: vec![num_expr(1.0), num_expr(2.0)],
            span: dummy_span(),
        },
        body: Box::new(Stmt::Block {
            statements: vec![Stmt::Expr(ident("item"))],
            span: dummy_span(),
        }),
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
    // item must NOT leak outside the loop
    assert!(checker.check_expr(&ident("item")).is_err());
}

#[test]
fn test_checker_string_concat_add() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Binary {
        left: Box::new(str_expr("hello")),
        op: BinaryOp::Add,
        right: Box::new(str_expr(" world")),
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::String);
}

#[test]
fn test_checker_function_stmt_registers_name() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::Function {
        name: "greet".to_string(),
        params: vec!["name".to_string()],
        body: vec![],
        is_async: false,
        is_generator: false,
        type_params: Vec::new(),
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
    // greet should now be visible as a Function type
    let ty = checker.check_expr(&ident("greet")).unwrap();
    assert!(matches!(ty, Type::Function { .. }));
}

#[test]
fn test_checker_try_catch_binds_param() {
    let mut checker = TypeChecker::new();
    // try {} catch (e) { e; }
    let stmt = Stmt::Try {
        try_block: Box::new(Stmt::Block {
            statements: vec![],
            span: dummy_span(),
        }),
        catch_clause: Some(CatchClause {
            param: "e".to_string(),
            body: Box::new(Stmt::Block {
                statements: vec![Stmt::Expr(ident("e"))],
                span: dummy_span(),
            }),
            span: dummy_span(),
        }),
        finally_block: None,
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
    // e must NOT leak after the catch block
    assert!(checker.check_expr(&ident("e")).is_err());
}

#[test]
fn test_checker_switch_stmt() {
    let mut checker = TypeChecker::new();
    // Put x in scope first
    checker
        .check_stmt(&Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        })
        .unwrap();

    let stmt = Stmt::Switch {
        value: ident("x"),
        cases: vec![SwitchCase {
            value: num_expr(1.0),
            body: vec![],
            span: dummy_span(),
        }],
        default: None,
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
}

#[test]
fn test_checker_throw_stmt() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::Throw {
        value: str_expr("oops"),
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
}

// ── gradual typing tests (Issue #450) ────────────────────────────────────────

#[test]
fn test_gradual_typing_vardecl_matching_type() {
    // var x: number = 42 → OK
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "x".to_string(),
        is_const: false,
        type_annotation: Some(TypeAnnotation::Number),
        initializer: Some(num_expr(42.0)),
        ownership: OwnershipMode::Owned,
        span: dummy_span(),
    });
    assert!(checker.check_stmt(&stmt).is_ok());
}

#[test]
fn test_gradual_typing_vardecl_mismatched_type() {
    // var x: number = "hello" → type error
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "x".to_string(),
        is_const: false,
        type_annotation: Some(TypeAnnotation::Number),
        initializer: Some(str_expr("hello")),
        ownership: OwnershipMode::Owned,
        span: dummy_span(),
    });
    let result = checker.check_stmt(&stmt);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.code,
        hudhudscript_errors::ErrorCode::TypeTypeMismatchInDecl
    );
    assert_eq!(err.context_get("expected"), Some("Number"));
    assert_eq!(err.context_get("found"), Some("String"));
    assert_eq!(err.context_get("variable"), Some("x"));
}

#[test]
fn test_gradual_typing_no_annotation_no_error() {
    // var x = "hello" → OK (no annotation, gradual)
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "x".to_string(),
        is_const: false,
        type_annotation: None,
        initializer: Some(str_expr("hello")),
        ownership: OwnershipMode::Owned,
        span: dummy_span(),
    });
    assert!(checker.check_stmt(&stmt).is_ok());
}

#[test]
fn test_gradual_typing_string_annotation_ok() {
    // var s: string = "hello" → OK
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "s".to_string(),
        is_const: false,
        type_annotation: Some(TypeAnnotation::String),
        initializer: Some(str_expr("hello")),
        ownership: OwnershipMode::Owned,
        span: dummy_span(),
    });
    assert!(checker.check_stmt(&stmt).is_ok());
}

#[test]
fn test_gradual_typing_boolean_mismatch() {
    // const flag: boolean = 42 → type error
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "flag".to_string(),
        is_const: true,
        type_annotation: Some(TypeAnnotation::Boolean),
        initializer: Some(num_expr(42.0)),
        ownership: OwnershipMode::Owned,
        span: dummy_span(),
    });
    let result = checker.check_stmt(&stmt);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.code,
        hudhudscript_errors::ErrorCode::TypeTypeMismatchInDecl
    );
    assert_eq!(err.context_get("variable"), Some("flag"));
}

#[test]
fn test_gradual_typing_any_annotation_accepts_all() {
    // var x: any = "hello" → OK (Any accepts everything)
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "x".to_string(),
        is_const: false,
        type_annotation: Some(TypeAnnotation::Any),
        initializer: Some(str_expr("hello")),
        ownership: OwnershipMode::Owned,
        span: dummy_span(),
    });
    assert!(checker.check_stmt(&stmt).is_ok());
}

#[test]
fn test_check_program_success() {
    let mut checker = TypeChecker::new();
    let stmts = vec![
        Stmt::Let {
            name: "a".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        },
        Stmt::Let {
            name: "b".to_string(),
            value: str_expr("hello"),
            span: dummy_span(),
        },
    ];
    assert!(checker.check_program(&stmts).is_ok());
    assert!(checker.errors().is_empty());
}

#[test]
fn test_check_program_reports_error() {
    let mut checker = TypeChecker::new();
    let stmts = vec![Stmt::VarDecl(VarDecl {
        name: "x".to_string(),
        is_const: false,
        type_annotation: Some(TypeAnnotation::Number),
        initializer: Some(str_expr("oops")),
        ownership: OwnershipMode::Owned,
        span: dummy_span(),
    })];
    assert!(checker.check_program(&stmts).is_err());
    assert_eq!(checker.errors().len(), 1);
}

// ── checker: assignment ───────────────────────────────────────────────────────

#[test]
fn test_checker_assignment_to_const_rejected() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Const {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        })
        .unwrap();
    let assign = Stmt::Assignment {
        target: ident("x"),
        value: num_expr(2.0),
        span: dummy_span(),
    };
    let result = checker.check_stmt(&assign);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.code,
        hudhudscript_errors::ErrorCode::TypeInvalidOperator
    );
    assert_eq!(err.context_get("op"), Some("assignment"));
}

#[test]
fn test_checker_assignment_to_let_ok() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        })
        .unwrap();
    let assign = Stmt::Assignment {
        target: ident("x"),
        value: num_expr(2.0),
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&assign).is_ok());
}

#[test]
fn test_checker_assignment_type_mismatch() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        })
        .unwrap();
    let assign = Stmt::Assignment {
        target: ident("x"),
        value: str_expr("hello"),
        span: dummy_span(),
    };
    let result = checker.check_stmt(&assign);
    assert!(result.is_err());
    assert!(result.unwrap_err().code == hudhudscript_errors::ErrorCode::TypeMismatch);
}

#[test]
fn test_checker_assignment_undefined_variable() {
    let mut checker = TypeChecker::new();
    let assign = Stmt::Assignment {
        target: ident("undefined_var"),
        value: num_expr(1.0),
        span: dummy_span(),
    };
    let result = checker.check_stmt(&assign);
    assert!(result.is_err());
    assert!(result.unwrap_err().code == hudhudscript_errors::ErrorCode::TypeUndefinedVariable);
}

// ── checker: return statement ─────────────────────────────────────────────────

#[test]
fn test_checker_return_with_value() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::Return {
        value: Some(num_expr(42.0)),
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
}

#[test]
fn test_checker_return_without_value() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::Return {
        value: None,
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
}

// ── checker: while loop ───────────────────────────────────────────────────────

#[test]
fn test_checker_while_loop() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::While {
        condition: Expr::Literal(Literal::Boolean(true), dummy_span()),
        body: Box::new(Stmt::Block {
            statements: vec![],
            span: dummy_span(),
        }),
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
}

// ── checker: expression types ─────────────────────────────────────────────────

#[test]
fn test_checker_empty_array() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Array {
        elements: vec![],
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Array(Box::new(Type::Any)));
}

#[test]
fn test_checker_mixed_array() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Array {
        elements: vec![num_expr(1.0), str_expr("hello")],
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    // Mixed types should result in Array<Any>
    assert_eq!(ty, Type::Array(Box::new(Type::Any)));
}

#[test]
fn test_checker_homogeneous_array() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Array {
        elements: vec![num_expr(1.0), num_expr(2.0)],
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Array(Box::new(Type::Number)));
}

#[test]
fn test_checker_object_expression() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Object {
        properties: vec![("x".to_string(), num_expr(1.0))],
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    match ty {
        Type::Object(props) => {
            assert_eq!(props.get("x"), Some(&Type::Number));
        }
        other => panic!("Expected Object, got: {:?}", other),
    }
}

#[test]
fn test_checker_template_string() {
    let mut checker = TypeChecker::new();
    let expr = Expr::TemplateString {
        parts: vec![],
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::String);
}

#[test]
fn test_checker_this_expr() {
    let mut checker = TypeChecker::new();
    let expr = Expr::This(dummy_span());
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Any);
}

#[test]
fn test_checker_yield_expr() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Yield {
        value: Some(Box::new(num_expr(42.0))),
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Any);
}

#[test]
fn test_checker_yield_no_value() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Yield {
        value: None,
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Any);
}

#[test]
fn test_checker_spread_expr() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Let {
            name: "arr".to_string(),
            value: Expr::Array {
                elements: vec![num_expr(1.0)],
                span: dummy_span(),
            },
            span: dummy_span(),
        })
        .unwrap();
    let expr = Expr::Spread {
        expr: Box::new(ident("arr")),
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Array(Box::new(Type::Number)));
}

// ── checker: unary ops ────────────────────────────────────────────────────────

#[test]
fn test_checker_unary_not_on_boolean() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(Expr::Literal(Literal::Boolean(true), dummy_span())),
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Boolean);
}

#[test]
fn test_checker_unary_neg_on_number() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(num_expr(5.0)),
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Number);
}

#[test]
fn test_checker_unary_plus_on_number() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Unary {
        op: UnaryOp::Plus,
        expr: Box::new(num_expr(5.0)),
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Number);
}

#[test]
fn test_checker_unary_not_on_string_fails() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(str_expr("hello")),
        span: dummy_span(),
    };
    assert!(checker.check_expr(&expr).is_err());
}

#[test]
fn test_checker_unary_neg_on_string_fails() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(str_expr("hello")),
        span: dummy_span(),
    };
    assert!(checker.check_expr(&expr).is_err());
}

// ── checker: binary ops ───────────────────────────────────────────────────────

#[test]
fn test_checker_comparison_ops_return_boolean() {
    let mut checker = TypeChecker::new();
    for op in [
        BinaryOp::Eq,
        BinaryOp::Ne,
        BinaryOp::Lt,
        BinaryOp::Le,
        BinaryOp::Gt,
        BinaryOp::Ge,
    ] {
        let expr = Expr::Binary {
            left: Box::new(num_expr(1.0)),
            op,
            right: Box::new(num_expr(2.0)),
            span: dummy_span(),
        };
        let ty = checker.check_expr(&expr).unwrap();
        assert_eq!(ty, Type::Boolean);
    }
}

#[test]
fn test_checker_logical_and_or() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal(Literal::Boolean(true), dummy_span())),
        op: BinaryOp::And,
        right: Box::new(Expr::Literal(Literal::Boolean(false), dummy_span())),
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Boolean);
}

#[test]
fn test_checker_sub_numbers() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Binary {
        left: Box::new(num_expr(5.0)),
        op: BinaryOp::Sub,
        right: Box::new(num_expr(3.0)),
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    assert_eq!(ty, Type::Number);
}

#[test]
fn test_checker_add_incompatible_types_fails() {
    let mut checker = TypeChecker::new();
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal(Literal::Boolean(true), dummy_span())),
        op: BinaryOp::Add,
        right: Box::new(num_expr(1.0)),
        span: dummy_span(),
    };
    assert!(checker.check_expr(&expr).is_err());
}

// ── checker: await ────────────────────────────────────────────────────────────

#[test]
fn test_checker_await_non_promise_fails() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        })
        .unwrap();
    let expr = Expr::Await {
        expr: Box::new(ident("x")),
        span: dummy_span(),
    };
    let result = checker.check_expr(&expr);
    assert!(result.is_err());
    assert!(result.unwrap_err().code == hudhudscript_errors::ErrorCode::TypeInvalidAwait);
}

// ── checker: index ────────────────────────────────────────────────────────────

#[test]
fn test_checker_index_non_array_fails() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        })
        .unwrap();
    let expr = Expr::Index {
        object: Box::new(ident("x")),
        index: Box::new(num_expr(0.0)),
        span: dummy_span(),
    };
    let result = checker.check_expr(&expr);
    assert!(result.is_err());
    assert!(result.unwrap_err().code == hudhudscript_errors::ErrorCode::TypeInvalidIndex);
}

// ── checker: member access ────────────────────────────────────────────────────

#[test]
fn test_checker_member_access_on_non_object_fails() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        })
        .unwrap();
    let expr = Expr::Member {
        object: Box::new(ident("x")),
        property: "foo".to_string(),
        span: dummy_span(),
    };
    let result = checker.check_expr(&expr);
    assert!(result.is_err());
    assert!(result.unwrap_err().code == hudhudscript_errors::ErrorCode::TypeInvalidMember);
}

// ── checker: VarDecl with ownership modes ────────────────────────────────────

#[test]
fn test_checker_vardecl_borrowed_ownership() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "r".to_string(),
        is_const: false,
        type_annotation: None,
        initializer: Some(num_expr(1.0)),
        ownership: OwnershipMode::Borrowed,
        span: dummy_span(),
    });
    assert!(checker.check_stmt(&stmt).is_ok());
}

#[test]
fn test_checker_vardecl_mut_borrowed_ownership() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "r".to_string(),
        is_const: false,
        type_annotation: None,
        initializer: Some(num_expr(1.0)),
        ownership: OwnershipMode::MutBorrowed,
        span: dummy_span(),
    });
    assert!(checker.check_stmt(&stmt).is_ok());
}

#[test]
fn test_checker_vardecl_no_initializer() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::VarDecl(VarDecl {
        name: "x".to_string(),
        is_const: false,
        type_annotation: Some(TypeAnnotation::Number),
        initializer: None,
        ownership: OwnershipMode::Owned,
        span: dummy_span(),
    });
    assert!(checker.check_stmt(&stmt).is_ok());
}

// ── checker: duplicate variable ───────────────────────────────────────────────

#[test]
fn test_checker_duplicate_let_rejected() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        })
        .unwrap();
    let result = checker.check_stmt(&Stmt::Let {
        name: "x".to_string(),
        value: num_expr(2.0),
        span: dummy_span(),
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().code == hudhudscript_errors::ErrorCode::TypeDuplicateVariable);
}

// ── checker: call non-function ────────────────────────────────────────────────

#[test]
fn test_checker_call_non_function_fails() {
    let mut checker = TypeChecker::new();
    checker
        .check_stmt(&Stmt::Let {
            name: "x".to_string(),
            value: num_expr(42.0),
            span: dummy_span(),
        })
        .unwrap();
    let expr = Expr::Call {
        callee: Box::new(ident("x")),
        args: vec![],
        span: dummy_span(),
    };
    let result = checker.check_expr(&expr);
    assert!(result.is_err());
    assert!(result.unwrap_err().code == hudhudscript_errors::ErrorCode::TypeInvalidOperator);
}

// ── SymbolTable coverage ──────────────────────────────────────────────────────

#[test]
fn test_symbol_table_exit_scope_at_root() {
    let mut table = SymbolTable::new();
    // Exiting at root should not panic
    table.exit_scope();
    // Still should be able to define
    assert!(table.define("x".to_string(), Type::Number).is_ok());
}

#[test]
fn test_symbol_table_nested_scopes() {
    let mut table = SymbolTable::new();
    table.define("outer".to_string(), Type::Number).unwrap();
    table.enter_scope();
    table.define("inner".to_string(), Type::String).unwrap();
    assert!(table.lookup("outer").is_some());
    assert!(table.lookup("inner").is_some());
    table.exit_scope();
    assert!(table.lookup("outer").is_some());
    assert!(table.lookup("inner").is_none());
}

// ── SymbolInfo coverage ───────────────────────────────────────────────────────

#[test]
fn test_symbol_info_new() {
    let info = SymbolInfo::new(Type::Number, true, Ownership::SharedRef);
    assert_eq!(info.ty, Type::Number);
    assert!(info.mutable);
    assert_eq!(info.ownership, Ownership::SharedRef);
}

// ── TypeChecker Default impl ──────────────────────────────────────────────────

#[test]
fn test_type_checker_default() {
    let checker = TypeChecker::default();
    assert!(checker.errors().is_empty());
}

// ── SymbolTable Default impl ──────────────────────────────────────────────────

#[test]
fn test_symbol_table_default() {
    let table = SymbolTable::default();
    assert!(table.lookup("anything").is_none());
}

// ── checker: arrow function ───────────────────────────────────────────────────

#[test]
fn test_checker_arrow_function_expression_body() {
    let mut checker = TypeChecker::new();
    let expr = Expr::ArrowFunction {
        params: vec!["x".to_string()],
        body: ArrowFunctionBody::Expression(Box::new(num_expr(42.0))),
        is_async: false,
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    match ty {
        Type::Function {
            params,
            return_type,
        } => {
            assert_eq!(params.len(), 1);
            assert_eq!(*return_type, Type::Number);
        }
        other => panic!("Expected Function, got: {:?}", other),
    }
}

#[test]
fn test_checker_arrow_function_block_body() {
    let mut checker = TypeChecker::new();
    let expr = Expr::ArrowFunction {
        params: vec![],
        body: ArrowFunctionBody::Block(vec![]),
        is_async: false,
        span: dummy_span(),
    };
    let ty = checker.check_expr(&expr).unwrap();
    match ty {
        Type::Function { return_type, .. } => {
            assert_eq!(*return_type, Type::Any);
        }
        other => panic!("Expected Function, got: {:?}", other),
    }
}

// ── checker: if with non-boolean condition ────────────────────────────────────

#[test]
fn test_checker_if_non_boolean_condition_fails() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::If {
        condition: num_expr(42.0),
        then_branch: Box::new(Stmt::Block {
            statements: vec![],
            span: dummy_span(),
        }),
        else_branch: None,
        span: dummy_span(),
    };
    let result = checker.check_stmt(&stmt);
    assert!(result.is_err());
    assert!(result.unwrap_err().code == hudhudscript_errors::ErrorCode::TypeMismatch);
}

// ── checker: export ───────────────────────────────────────────────────────────

#[test]
fn test_checker_export_stmt() {
    let mut checker = TypeChecker::new();
    let stmt = Stmt::Export {
        item: Box::new(Stmt::Let {
            name: "x".to_string(),
            value: num_expr(1.0),
            span: dummy_span(),
        }),
        source: None,
        span: dummy_span(),
    };
    assert!(checker.check_stmt(&stmt).is_ok());
}

// ── checker: no-op statements ─────────────────────────────────────────────────

#[test]
fn test_checker_noop_statements() {
    let mut checker = TypeChecker::new();
    assert!(checker
        .check_stmt(&Stmt::Break { span: dummy_span() })
        .is_ok());
    assert!(checker
        .check_stmt(&Stmt::Continue { span: dummy_span() })
        .is_ok());
}

// ── function compatibility contravariance ─────────────────────────────────────

#[test]
fn test_compat_function_contravariant_params() {
    // A function accepting Any should be compatible where Number is expected
    let f_any_param = Type::Function {
        params: vec![Type::Any],
        return_type: Box::new(Type::Number),
    };
    let f_num_param = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::Number),
    };
    // Contravariant: f(Any) is compatible with f(Number) because
    // Any param is more permissive
    assert!(f_any_param.is_compatible_with(&f_num_param));
}
