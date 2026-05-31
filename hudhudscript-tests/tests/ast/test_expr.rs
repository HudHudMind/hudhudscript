use hudhudscript_ast::*;

// ============================================================================
// Position
// ============================================================================

#[test]
fn position_new_fields() {
    let pos = Position::new(5, 10, 42);
    assert_eq!(pos.line, 5);
    assert_eq!(pos.column, 10);
    assert_eq!(pos.offset, 42);
}

#[test]
fn position_start() {
    let pos = Position::start();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 0);
}

#[test]
fn position_default_equals_start() {
    assert_eq!(Position::default(), Position::start());
}

#[test]
fn position_equality() {
    let a = Position::new(3, 7, 42);
    let b = Position::new(3, 7, 42);
    let c = Position::new(4, 7, 42);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn position_clone() {
    let a = Position::new(10, 20, 100);
    let b = a;
    assert_eq!(a, b);
}

// ============================================================================
// Span
// ============================================================================

#[test]
fn span_new_fields() {
    let start = Position::new(1, 1, 0);
    let end = Position::new(1, 5, 4);
    let span = Span::new(start, end);
    assert_eq!(span.start, start);
    assert_eq!(span.end, end);
}

#[test]
fn span_start_factory() {
    let span = Span::start();
    assert_eq!(span.start, Position::start());
    assert_eq!(span.end, Position::start());
}

#[test]
fn span_default_equals_start() {
    assert_eq!(Span::default(), Span::start());
}

#[test]
fn span_merge_disjoint() {
    let s1 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let s2 = Span::new(Position::new(1, 10, 9), Position::new(1, 15, 14));
    let merged = s1.merge(s2);
    assert_eq!(merged.start.offset, 0);
    assert_eq!(merged.end.offset, 14);
}

#[test]
fn span_merge_reversed_order() {
    let s1 = Span::new(Position::new(2, 5, 20), Position::new(2, 10, 25));
    let s2 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let merged = s1.merge(s2);
    assert_eq!(merged.start.offset, 0);
    assert_eq!(merged.end.offset, 25);
}

#[test]
fn span_merge_overlapping() {
    let s1 = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let s2 = Span::new(Position::new(1, 5, 4), Position::new(1, 10, 9));
    let merged = s1.merge(s2);
    assert_eq!(merged.start.offset, 0);
    assert_eq!(merged.end.offset, 19);
}

#[test]
fn span_equality() {
    let s1 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let s2 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    assert_eq!(s1, s2);
}

// ============================================================================
// Literal
// ============================================================================

fn mk_span() -> Span {
    Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9))
}

#[test]
fn literal_string() {
    let lit = Literal::String("hello".to_string());
    assert_eq!(lit, Literal::String("hello".to_string()));
}

#[test]
fn literal_number() {
    let lit = Literal::Number(42.0, false);
    assert_eq!(lit, Literal::Number(42.0, false));
}

#[test]
fn literal_boolean_true() {
    let lit = Literal::Boolean(true);
    assert_eq!(lit, Literal::Boolean(true));
}

#[test]
fn literal_boolean_false() {
    let lit = Literal::Boolean(false);
    assert_eq!(lit, Literal::Boolean(false));
}

#[test]
fn literal_null() {
    assert_eq!(Literal::Null, Literal::Null);
}

#[test]
fn literal_inequality() {
    assert_ne!(Literal::Number(1.0, false), Literal::Number(2.0, false));
    assert_ne!(Literal::String("a".into()), Literal::Null);
}

// ============================================================================
// BinaryOp
// ============================================================================

#[test]
fn binary_op_all_variants_equality() {
    let ops = [
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Mod,
        BinaryOp::Eq,
        BinaryOp::Ne,
        BinaryOp::Lt,
        BinaryOp::Le,
        BinaryOp::Gt,
        BinaryOp::Ge,
        BinaryOp::And,
        BinaryOp::Or,
    ];
    for op in &ops {
        assert_eq!(*op, *op);
    }
}

#[test]
fn binary_op_inequality() {
    assert_ne!(BinaryOp::Add, BinaryOp::Sub);
    assert_ne!(BinaryOp::Eq, BinaryOp::Ne);
    assert_ne!(BinaryOp::And, BinaryOp::Or);
}

// ============================================================================
// UnaryOp
// ============================================================================

#[test]
fn unary_op_all_variants_equality() {
    assert_eq!(UnaryOp::Not, UnaryOp::Not);
    assert_eq!(UnaryOp::Neg, UnaryOp::Neg);
    assert_eq!(UnaryOp::Plus, UnaryOp::Plus);
}

#[test]
fn unary_op_inequality() {
    assert_ne!(UnaryOp::Not, UnaryOp::Neg);
    assert_ne!(UnaryOp::Not, UnaryOp::Plus);
    assert_ne!(UnaryOp::Neg, UnaryOp::Plus);
}

// ============================================================================
// Expr variants — construction + span()
// ============================================================================

#[test]
fn expr_literal_span() {
    let span = mk_span();
    let expr = Expr::Literal(Literal::Number(42.0, false), span);
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_identifier_span() {
    let span = mk_span();
    let expr = Expr::Identifier("x".to_string(), span);
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_binary_span() {
    let span = mk_span();
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal(Literal::Number(1.0, false), span)),
        op: BinaryOp::Add,
        right: Box::new(Expr::Literal(Literal::Number(2.0, false), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_unary_span() {
    let span = mk_span();
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(Expr::Literal(Literal::Number(1.0, false), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_call_span() {
    let span = mk_span();
    let expr = Expr::Call {
        callee: Box::new(Expr::Identifier("f".to_string(), span)),
        args: vec![Expr::Literal(Literal::Number(1.0, false), span)],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_member_span() {
    let span = mk_span();
    let expr = Expr::Member {
        object: Box::new(Expr::Identifier("obj".to_string(), span)),
        property: "field".to_string(),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_index_span() {
    let span = mk_span();
    let expr = Expr::Index {
        object: Box::new(Expr::Identifier("arr".to_string(), span)),
        index: Box::new(Expr::Literal(Literal::Number(0.0, false), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_array_span() {
    let span = mk_span();
    let expr = Expr::Array {
        elements: vec![Expr::Literal(Literal::Number(1.0, false), span)],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_object_span() {
    let span = mk_span();
    let expr = Expr::Object {
        properties: vec![("x".to_string(), Expr::Literal(Literal::Number(1.0, false), span))],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_template_string_span() {
    let span = mk_span();
    let expr = Expr::TemplateString {
        parts: vec![TemplateStringPart::Text("hello".to_string())],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_arrow_function_span() {
    let span = mk_span();
    let expr = Expr::ArrowFunction {
        params: vec!["x".to_string()],
        body: ArrowFunctionBody::Expression(Box::new(Expr::Identifier("x".to_string(), span))),
        is_async: false,
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_arrow_function_async() {
    let span = mk_span();
    let expr = Expr::ArrowFunction {
        params: vec!["x".to_string()],
        body: ArrowFunctionBody::Block(vec![]),
        is_async: true,
        span,
    };
    assert_eq!(expr.span(), span);
    if let Expr::ArrowFunction { is_async, .. } = &expr {
        assert!(*is_async);
    }
}

#[test]
fn expr_await_span() {
    let span = mk_span();
    let expr = Expr::Await {
        expr: Box::new(Expr::Identifier("p".to_string(), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_new_span() {
    let span = mk_span();
    let expr = Expr::New {
        class_name: "Foo".to_string(),
        args: vec![],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_this_span() {
    let span = mk_span();
    let expr = Expr::This(span);
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_spread_span() {
    let span = mk_span();
    let expr = Expr::Spread {
        expr: Box::new(Expr::Identifier("arr".to_string(), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_yield_with_value_span() {
    let span = mk_span();
    let expr = Expr::Yield {
        value: Some(Box::new(Expr::Literal(Literal::Number(1.0, false), span))),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn expr_yield_without_value_span() {
    let span = mk_span();
    let expr = Expr::Yield { value: None, span };
    assert_eq!(expr.span(), span);
}

// ============================================================================
// TemplateStringPart
// ============================================================================

#[test]
fn template_string_part_text() {
    let part = TemplateStringPart::Text("hello".to_string());
    assert_eq!(part, TemplateStringPart::Text("hello".to_string()));
}

#[test]
fn template_string_part_interpolation() {
    let span = mk_span();
    let part = TemplateStringPart::Interpolation(Box::new(Expr::Identifier("x".to_string(), span)));
    match part {
        TemplateStringPart::Interpolation(expr) => {
            assert_eq!(expr.span(), span);
        }
        _ => panic!("Expected Interpolation"),
    }
}

// ============================================================================
// ArrowFunctionBody
// ============================================================================

#[test]
fn arrow_body_expression() {
    let span = mk_span();
    let body = ArrowFunctionBody::Expression(Box::new(Expr::Literal(Literal::Number(1.0, false), span)));
    match body {
        ArrowFunctionBody::Expression(e) => assert_eq!(e.span(), span),
        _ => panic!("Expected Expression body"),
    }
}

#[test]
fn arrow_body_block() {
    let body = ArrowFunctionBody::Block(vec![]);
    match body {
        ArrowFunctionBody::Block(stmts) => assert!(stmts.is_empty()),
        _ => panic!("Expected Block body"),
    }
}

// ============================================================================
// Stmt variants — construction + span()
// ============================================================================

#[test]
fn stmt_expr_span() {
    let span = mk_span();
    let stmt = Stmt::Expr(Expr::Literal(Literal::Number(42.0, false), span));
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_let_span() {
    let span = mk_span();
    let stmt = Stmt::Let {
        name: "x".to_string(),
        value: Expr::Literal(Literal::Number(1.0, false), span),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_const_span() {
    let span = mk_span();
    let stmt = Stmt::Const {
        name: "PI".to_string(),
        value: Expr::Literal(Literal::Number(3.14, true), span),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_assignment_span() {
    let span = mk_span();
    let stmt = Stmt::Assignment {
        target: Expr::Identifier("x".to_string(), span),
        value: Expr::Literal(Literal::Number(42.0, false), span),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_if_span() {
    let span = mk_span();
    let stmt = Stmt::If {
        condition: Expr::Literal(Literal::Boolean(true), span),
        then_branch: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        else_branch: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_if_with_else() {
    let span = mk_span();
    let stmt = Stmt::If {
        condition: Expr::Literal(Literal::Boolean(true), span),
        then_branch: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        else_branch: Some(Box::new(Stmt::Block {
            statements: vec![],
            span,
        })),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_while_span() {
    let span = mk_span();
    let stmt = Stmt::While {
        condition: Expr::Literal(Literal::Boolean(true), span),
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_for_span() {
    let span = mk_span();
    let stmt = Stmt::For {
        variable: "i".to_string(),
        iterable: Expr::Array {
            elements: vec![],
            span,
        },
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_for_c_style_span() {
    let span = mk_span();
    let stmt = Stmt::ForCStyle {
        init: None,
        condition: None,
        update: None,
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_for_range_span() {
    let span = mk_span();
    let stmt = Stmt::ForRange {
        start: Expr::Literal(Literal::Number(0.0, false), span),
        stop: Expr::Literal(Literal::Number(10.0, false), span),
        step: None,
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_block_span() {
    let span = mk_span();
    let stmt = Stmt::Block {
        statements: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_return_with_value_span() {
    let span = mk_span();
    let stmt = Stmt::Return {
        value: Some(Expr::Literal(Literal::Number(0.0, false), span)),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_return_without_value_span() {
    let span = mk_span();
    let stmt = Stmt::Return { value: None, span };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_break_span() {
    let span = mk_span();
    let stmt = Stmt::Break { span };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_continue_span() {
    let span = mk_span();
    let stmt = Stmt::Continue { span };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_throw_span() {
    let span = mk_span();
    let stmt = Stmt::Throw {
        value: Expr::Literal(Literal::String("error".into()), span),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_function_span() {
    let span = mk_span();
    let stmt = Stmt::Function {
        name: "foo".to_string(),
        params: vec!["x".to_string()],
        body: vec![],
        is_async: false,
        is_generator: false,
        type_params: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_function_async_generator() {
    let span = mk_span();
    let stmt = Stmt::Function {
        name: "gen".to_string(),
        params: vec![],
        body: vec![],
        is_async: true,
        is_generator: true,
        type_params: vec![],
        span,
    };
    if let Stmt::Function {
        is_async,
        is_generator,
        ..
    } = &stmt
    {
        assert!(*is_async);
        assert!(*is_generator);
    }
}

#[test]
fn stmt_import_span() {
    let span = mk_span();
    let stmt = Stmt::Import {
        path: "module".to_string(),
        imports: ImportKind::Named(vec!["foo".to_string()]),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_export_span() {
    let span = mk_span();
    let stmt = Stmt::Export {
        item: Box::new(Stmt::Let {
            name: "x".to_string(),
            value: Expr::Literal(Literal::Number(1.0, false), span),
            span,
        }),
        source: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_switch_span() {
    let span = mk_span();
    let stmt = Stmt::Switch {
        value: Expr::Identifier("x".to_string(), span),
        cases: vec![],
        default: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_try_span() {
    let span = mk_span();
    let stmt = Stmt::Try {
        try_block: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        catch_clause: None,
        finally_block: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_trait_span() {
    let span = mk_span();
    let stmt = Stmt::Trait {
        name: "Serializable".to_string(),
        type_params: vec![],
        methods: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_destructure_span() {
    let span = mk_span();
    let stmt = Stmt::Destructure {
        pattern: Pattern::Identifier("x".to_string()),
        value: Expr::Literal(Literal::Number(1.0, false), span),
        is_const: false,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_class_span() {
    let span = mk_span();
    let stmt = Stmt::Class(ClassDecl {
        is_abstract: false,
        name: "Foo".to_string(),
        parent: None,
        type_params: vec![],
        implements: vec![],
        members: vec![],
        span,
    });
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_match_span() {
    let span = mk_span();
    let stmt = Stmt::Match {
        value: Expr::Identifier("x".to_string(), span),
        arms: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_enum_decl_span() {
    let span = mk_span();
    let stmt = Stmt::EnumDecl {
        name: "Color".to_string(),
        variants: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_spawn_span() {
    let span = mk_span();
    let stmt = Stmt::Spawn {
        subject_name: "Player".to_string(),
        args: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_send_span() {
    let span = mk_span();
    let stmt = Stmt::Send {
        message: Box::new(Expr::Literal(Literal::String("msg".into()), span)),
        target: Box::new(Expr::Identifier("target".into(), span)),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn stmt_receive_span() {
    let span = mk_span();
    let stmt = Stmt::Receive {
        variable: "msg".to_string(),
        source: Box::new(Expr::Identifier("src".into(), span)),
        span,
    };
    assert_eq!(stmt.span(), span);
}

// ============================================================================
// Decl variants — construction + span()
// ============================================================================

#[test]
fn decl_agent_span() {
    let span = mk_span();
    let decl = Decl::Agent {
        name: "MyAgent".into(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn decl_action_span() {
    let span = mk_span();
    let decl = Decl::Action {
        name: "act".into(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn decl_tool_span() {
    let span = mk_span();
    let decl = Decl::Tool {
        name: "t".into(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn decl_resource_span() {
    let span = mk_span();
    let decl = Decl::Resource {
        name: "r".into(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn decl_import_span() {
    let span = mk_span();
    let decl = Decl::Import {
        module: "mod".into(),
        alias: None,
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn decl_import_with_alias() {
    let span = mk_span();
    let decl = Decl::Import {
        module: "mod".into(),
        alias: Some("m".into()),
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn decl_provider_span() {
    let span = mk_span();
    let decl = Decl::Provider {
        name: "openai".into(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn decl_store_span() {
    let span = mk_span();
    let decl = Decl::Store {
        name: "s".into(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn decl_role_span() {
    let span = mk_span();
    let decl = Decl::Role {
        name: "Fighter".into(),
        capabilities: vec![],
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

// ============================================================================
// Decl via Stmt::Decl — span delegation
// ============================================================================

#[test]
fn stmt_decl_agent_span() {
    let span = mk_span();
    let stmt = Stmt::Decl(Decl::Agent {
        name: "A".into(),
        fields: vec![],
        span,
    });
    assert_eq!(stmt.span(), span);
}

// ============================================================================
// ImportKind
// ============================================================================

#[test]
fn import_kind_named() {
    let ik = ImportKind::Named(vec!["foo".into(), "bar".into()]);
    match ik {
        ImportKind::Named(names) => assert_eq!(names.len(), 2),
        _ => panic!("Expected Named"),
    }
}

#[test]
fn import_kind_default() {
    let ik = ImportKind::Default("mod".into());
    match ik {
        ImportKind::Default(name) => assert_eq!(name, "mod"),
        _ => panic!("Expected Default"),
    }
}

#[test]
fn import_kind_wildcard() {
    let ik = ImportKind::Wildcard("all".into());
    match ik {
        ImportKind::Wildcard(alias) => assert_eq!(alias, "all"),
        _ => panic!("Expected Wildcard"),
    }
}

// ============================================================================
// TypeAnnotation
// ============================================================================

#[test]
fn type_annotation_all_simple_variants() {
    assert_eq!(TypeAnnotation::String, TypeAnnotation::String);
    assert_eq!(TypeAnnotation::Number, TypeAnnotation::Number);
    assert_eq!(TypeAnnotation::Boolean, TypeAnnotation::Boolean);
    assert_eq!(TypeAnnotation::Null, TypeAnnotation::Null);
    assert_eq!(TypeAnnotation::Any, TypeAnnotation::Any);
    assert_eq!(TypeAnnotation::Tool, TypeAnnotation::Tool);
    assert_eq!(TypeAnnotation::Resource, TypeAnnotation::Resource);
    assert_eq!(TypeAnnotation::Server, TypeAnnotation::Server);
}

#[test]
fn type_annotation_generic() {
    let t = TypeAnnotation::Generic("T".into());
    assert_eq!(t, TypeAnnotation::Generic("T".into()));
}

#[test]
fn type_annotation_array() {
    let t = TypeAnnotation::Array(Box::new(TypeAnnotation::Number));
    assert_eq!(t, TypeAnnotation::Array(Box::new(TypeAnnotation::Number)));
}

#[test]
fn type_annotation_union() {
    let t = TypeAnnotation::Union(vec![TypeAnnotation::String, TypeAnnotation::Number]);
    assert_eq!(
        t,
        TypeAnnotation::Union(vec![TypeAnnotation::String, TypeAnnotation::Number])
    );
}

#[test]
fn type_annotation_parameterized() {
    let t = TypeAnnotation::Parameterized {
        base: Box::new(TypeAnnotation::Generic("Stack".into())),
        args: vec![TypeAnnotation::Number],
    };
    match t {
        TypeAnnotation::Parameterized { base, args } => {
            assert_eq!(*base, TypeAnnotation::Generic("Stack".into()));
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Parameterized"),
    }
}

// ============================================================================
// OwnershipMode
// ============================================================================

#[test]
fn ownership_mode_default_is_owned() {
    let mode: OwnershipMode = Default::default();
    assert_eq!(mode, OwnershipMode::Owned);
}

#[test]
fn ownership_mode_variants() {
    assert_eq!(OwnershipMode::Owned, OwnershipMode::Owned);
    assert_eq!(OwnershipMode::Borrowed, OwnershipMode::Borrowed);
    assert_eq!(OwnershipMode::MutBorrowed, OwnershipMode::MutBorrowed);
    assert_ne!(OwnershipMode::Owned, OwnershipMode::Borrowed);
}

// ============================================================================
// AccessModifier
// ============================================================================

#[test]
fn access_modifier_default_is_private() {
    let am: AccessModifier = Default::default();
    assert_eq!(am, AccessModifier::Private);
}

#[test]
fn access_modifier_variants() {
    assert_eq!(AccessModifier::Public, AccessModifier::Public);
    assert_eq!(AccessModifier::Protected, AccessModifier::Protected);
    assert_ne!(AccessModifier::Public, AccessModifier::Private);
}

// ============================================================================
// Pattern
// ============================================================================

#[test]
fn pattern_identifier() {
    let p = Pattern::Identifier("x".into());
    assert_eq!(p, Pattern::Identifier("x".into()));
}

#[test]
fn pattern_identifier_default() {
    let span = mk_span();
    let p = Pattern::IdentifierDefault("port".into(), Expr::Literal(Literal::Number(8080.0, false), span));
    match p {
        Pattern::IdentifierDefault(name, _) => assert_eq!(name, "port"),
        _ => panic!("Expected IdentifierDefault"),
    }
}

#[test]
fn pattern_array_with_rest() {
    let p = Pattern::Array {
        elements: vec![Pattern::Identifier("a".into())],
        rest: Some(Box::new(Pattern::Identifier("rest".into()))),
    };
    match p {
        Pattern::Array { elements, rest } => {
            assert_eq!(elements.len(), 1);
            assert!(rest.is_some());
        }
        _ => panic!("Expected Array pattern"),
    }
}

#[test]
fn pattern_object_without_rest() {
    let p = Pattern::Object {
        properties: vec![("name".into(), Pattern::Identifier("name".into()))],
        rest: None,
    };
    match p {
        Pattern::Object { properties, rest } => {
            assert_eq!(properties.len(), 1);
            assert!(rest.is_none());
        }
        _ => panic!("Expected Object pattern"),
    }
}

// ============================================================================
// VarDecl
// ============================================================================

#[test]
fn var_decl_with_ownership() {
    let span = mk_span();
    let decl = VarDecl {
        name: "x".into(),
        is_const: true,
        type_annotation: Some(TypeAnnotation::Number),
        initializer: None,
        ownership: OwnershipMode::Borrowed,
        span,
    };
    assert_eq!(decl.name, "x");
    assert!(decl.is_const);
    assert_eq!(decl.ownership, OwnershipMode::Borrowed);
    assert!(decl.initializer.is_none());
}

#[test]
fn var_decl_default_ownership() {
    let span = mk_span();
    let decl = VarDecl {
        name: "y".into(),
        is_const: false,
        type_annotation: None,
        initializer: Some(Expr::Literal(Literal::Number(1.0, false), span)),
        ownership: Default::default(),
        span,
    };
    assert_eq!(decl.ownership, OwnershipMode::Owned);
}

// ============================================================================
// ClassDecl / ClassMember
// ============================================================================

#[test]
fn class_decl_with_parent_and_implements() {
    let span = mk_span();
    let cls = ClassDecl {
        is_abstract: false,
        name: "Dog".into(),
        parent: Some("Animal".into()),
        type_params: vec![],
        implements: vec!["Pet".into()],
        members: vec![],
        span,
    };
    assert_eq!(cls.name, "Dog");
    assert_eq!(cls.parent.unwrap(), "Animal");
    assert_eq!(cls.implements, vec!["Pet".to_string()]);
}

#[test]
fn class_member_field() {
    let span = mk_span();
    let m = ClassMember::Field {
        access: AccessModifier::Public,
        is_static: true,
        name: "count".into(),
        initializer: Some(Expr::Literal(Literal::Number(0.0, false), span)),
        span,
    };
    match m {
        ClassMember::Field {
            access,
            is_static,
            name,
            ..
        } => {
            assert_eq!(access, AccessModifier::Public);
            assert!(is_static);
            assert_eq!(name, "count");
        }
        _ => panic!("Expected Field"),
    }
}

#[test]
fn class_member_method() {
    let span = mk_span();
    let m = ClassMember::Method {
        access: AccessModifier::Private,
        is_static: false,
        name: "greet".into(),
        params: vec![],
        body: vec![],
        span,
    };
    match m {
        ClassMember::Method { name, .. } => assert_eq!(name, "greet"),
        _ => panic!("Expected Method"),
    }
}

#[test]
fn class_member_constructor() {
    let span = mk_span();
    let m = ClassMember::Constructor {
        params: vec![],
        body: vec![],
        span,
    };
    match m {
        ClassMember::Constructor { params, body, .. } => {
            assert!(params.is_empty());
            assert!(body.is_empty());
        }
        _ => panic!("Expected Constructor"),
    }
}

// ============================================================================
// GenericParam / TraitMethodSig / Param
// ============================================================================

#[test]
fn generic_param_with_constraint() {
    let span = mk_span();
    let gp = GenericParam {
        name: "T".into(),
        constraint: Some("Comparable".into()),
        span,
    };
    assert_eq!(gp.name, "T");
    assert_eq!(gp.constraint.unwrap(), "Comparable");
}

#[test]
fn generic_param_without_constraint() {
    let span = mk_span();
    let gp = GenericParam {
        name: "U".into(),
        constraint: None,
        span,
    };
    assert!(gp.constraint.is_none());
}

#[test]
fn trait_method_sig() {
    let span = mk_span();
    let sig = TraitMethodSig {
        name: "serialize".into(),
        params: vec![],
        return_type: Some(TypeAnnotation::String),
        span,
    };
    assert_eq!(sig.name, "serialize");
    assert_eq!(sig.return_type.unwrap(), TypeAnnotation::String);
}

#[test]
fn param_with_type_annotation() {
    let span = mk_span();
    let p = Param {
        name: "x".into(),
        type_annotation: Some(TypeAnnotation::Number),
        span,
    };
    assert_eq!(p.name, "x");
    assert_eq!(p.type_annotation.unwrap(), TypeAnnotation::Number);
}

// ============================================================================
// TransportType / AuthType / AuthConfig
// ============================================================================

#[test]
fn transport_type_variants() {
    assert_eq!(TransportType::Stdio, TransportType::Stdio);
    assert_eq!(TransportType::SSE, TransportType::SSE);
    assert_ne!(TransportType::Stdio, TransportType::SSE);
}

#[test]
fn auth_type_variants() {
    assert_eq!(AuthType::Bearer, AuthType::Bearer);
    assert_eq!(AuthType::Basic, AuthType::Basic);
    assert_eq!(AuthType::ApiKey, AuthType::ApiKey);
    assert_ne!(AuthType::Bearer, AuthType::Basic);
}

// ============================================================================
// MatchPattern / MatchArm / SwitchCase / EnumVariant / CatchClause
// ============================================================================

#[test]
fn match_pattern_wildcard() {
    let p = MatchPattern::Wildcard;
    assert_eq!(p, MatchPattern::Wildcard);
}

#[test]
fn match_pattern_literal() {
    let p = MatchPattern::Literal(Literal::Number(42.0, false));
    match p {
        MatchPattern::Literal(Literal::Number(n, _)) => assert_eq!(n, 42.0),
        _ => panic!("Expected Literal pattern"),
    }
}

#[test]
fn match_pattern_identifier() {
    let p = MatchPattern::Identifier("x".into());
    assert_eq!(p, MatchPattern::Identifier("x".into()));
}

#[test]
fn match_pattern_enum_variant() {
    let p = MatchPattern::EnumVariant {
        enum_name: "Shape".into(),
        variant: "Circle".into(),
        binding: Some("r".into()),
    };
    match p {
        MatchPattern::EnumVariant {
            enum_name,
            variant,
            binding,
        } => {
            assert_eq!(enum_name, "Shape");
            assert_eq!(variant, "Circle");
            assert_eq!(binding.unwrap(), "r");
        }
        _ => panic!("Expected EnumVariant"),
    }
}

#[test]
fn match_arm_construction() {
    let span = mk_span();
    let arm = MatchArm {
        pattern: MatchPattern::Wildcard,
        guard: None,
        body: vec![],
        span,
    };
    assert_eq!(arm.span, span);
}

#[test]
fn switch_case_construction() {
    let span = mk_span();
    let case = SwitchCase {
        value: Expr::Literal(Literal::Number(1.0, false), span),
        body: vec![],
        span,
    };
    assert_eq!(case.span, span);
}

#[test]
fn enum_variant_construction() {
    let span = mk_span();
    let v = EnumVariant {
        name: "Circle".into(),
        fields: vec!["radius".into()],
        span,
    };
    assert_eq!(v.name, "Circle");
    assert_eq!(v.fields.len(), 1);
}

#[test]
fn catch_clause_construction() {
    let span = mk_span();
    let cc = CatchClause {
        param: "e".into(),
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(cc.param, "e");
}

// ============================================================================
// Decorator
// ============================================================================

#[test]
fn decorator_construction() {
    let span = mk_span();
    let d = Decorator {
        name: "ai".into(),
        params: vec![],
        span,
    };
    assert_eq!(d.name, "ai");
    assert!(d.params.is_empty());
}

// ============================================================================
// From span.rs inline tests
// ============================================================================

#[test]
fn test_position_creation() {
    let pos = Position::new(5, 10, 42);
    assert_eq!(pos.line, 5);
    assert_eq!(pos.column, 10);
    assert_eq!(pos.offset, 42);
}

#[test]
fn test_position_start() {
    let pos = Position::start();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 0);
}

#[test]
fn test_span_creation() {
    let start = Position::new(1, 1, 0);
    let end = Position::new(1, 5, 4);
    let span = Span::new(start, end);
    assert_eq!(span.start, start);
    assert_eq!(span.end, end);
}

#[test]
fn test_span_merge() {
    let span1 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let span2 = Span::new(Position::new(1, 10, 9), Position::new(1, 15, 14));
    let merged = span1.merge(span2);
    assert_eq!(merged.start.offset, 0);
    assert_eq!(merged.end.offset, 14);
}

#[test]
fn test_position_default() {
    let pos = Position::default();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
    assert_eq!(pos.offset, 0);
}

#[test]
fn test_span_default() {
    let span = Span::default();
    assert_eq!(span.start, Position::start());
    assert_eq!(span.end, Position::start());
}

#[test]
fn test_span_start() {
    let span = Span::start();
    assert_eq!(span.start.line, 1);
    assert_eq!(span.start.column, 1);
    assert_eq!(span.start.offset, 0);
    assert_eq!(span.end.line, 1);
    assert_eq!(span.end.column, 1);
    assert_eq!(span.end.offset, 0);
}

#[test]
fn test_span_merge_reversed_order() {
    // When span2 starts before span1
    let span1 = Span::new(Position::new(2, 5, 20), Position::new(2, 10, 25));
    let span2 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let merged = span1.merge(span2);
    assert_eq!(merged.start.offset, 0);
    assert_eq!(merged.end.offset, 25);
}

#[test]
fn test_span_merge_overlapping() {
    // When span2's end is before span1's end
    let span1 = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let span2 = Span::new(Position::new(1, 5, 4), Position::new(1, 10, 9));
    let merged = span1.merge(span2);
    assert_eq!(merged.start.offset, 0);
    assert_eq!(merged.end.offset, 19);
}

#[test]
fn test_position_equality() {
    let p1 = Position::new(3, 7, 42);
    let p2 = Position::new(3, 7, 42);
    assert_eq!(p1, p2);
}

#[test]
fn test_span_equality() {
    let s1 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let s2 = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    assert_eq!(s1, s2);
}

// ============================================================================
// From lib.rs inline tests
// ============================================================================

#[test]
fn test_span_tracking() {
    let start = Position::new(1, 1, 0);
    let end = Position::new(1, 10, 9);
    let span = Span::new(start, end);

    assert_eq!(span.start.line, 1);
    assert_eq!(span.start.column, 1);
    assert_eq!(span.end.column, 10);
}

#[test]
fn test_full_ast_creation() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Literal(Literal::Number(42.0, false), span);
    let stmt = Stmt::Expr(expr);
    assert_eq!(stmt.span(), span);
}

// ============================================================================
// From expr.rs inline tests
// ============================================================================

#[test]
fn test_literal_creation() {
    let lit = Literal::Number(42.0, false);
    assert_eq!(lit, Literal::Number(42.0, false));

    let lit = Literal::String("hello".to_string());
    assert_eq!(lit, Literal::String("hello".to_string()));

    let lit = Literal::Boolean(true);
    assert_eq!(lit, Literal::Boolean(true));

    let lit = Literal::Null;
    assert_eq!(lit, Literal::Null);
}

#[test]
fn test_binary_op() {
    let ops = vec![
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Eq,
        BinaryOp::Ne,
        BinaryOp::Lt,
        BinaryOp::And,
        BinaryOp::Or,
    ];

    for op in ops {
        assert_eq!(op, op);
    }
}

#[test]
fn test_expr_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let expr = Expr::Literal(Literal::Number(42.0, false), span);
    assert_eq!(expr.span(), span);
}

#[test]
fn test_binary_expr() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let left = Box::new(Expr::Literal(Literal::Number(1.0, false), span));
    let right = Box::new(Expr::Literal(Literal::Number(2.0, false), span));

    let expr = Expr::Binary {
        left,
        op: BinaryOp::Add,
        right,
        span,
    };

    assert_eq!(expr.span(), span);
}

#[test]
fn test_identifier_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let expr = Expr::Identifier("x".to_string(), span);
    assert_eq!(expr.span(), span);
}

#[test]
fn test_unary_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(Expr::Literal(Literal::Number(1.0, false), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_call_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Call {
        callee: Box::new(Expr::Identifier("f".to_string(), span)),
        args: vec![],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_member_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Member {
        object: Box::new(Expr::Identifier("obj".to_string(), span)),
        property: "field".to_string(),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_index_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Index {
        object: Box::new(Expr::Identifier("arr".to_string(), span)),
        index: Box::new(Expr::Literal(Literal::Number(0.0, false), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_array_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Array {
        elements: vec![],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_object_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Object {
        properties: vec![],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_template_string_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::TemplateString {
        parts: vec![TemplateStringPart::Text("hello".to_string())],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_arrow_function_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::ArrowFunction {
        params: vec!["x".to_string()],
        body: ArrowFunctionBody::Expression(Box::new(Expr::Identifier("x".to_string(), span))),
        is_async: false,
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_await_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Await {
        expr: Box::new(Expr::Identifier("p".to_string(), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_new_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::New {
        class_name: "Foo".to_string(),
        args: vec![],
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_this_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 4, 3));
    let expr = Expr::This(span);
    assert_eq!(expr.span(), span);
}

#[test]
fn test_spread_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Spread {
        expr: Box::new(Expr::Identifier("arr".to_string(), span)),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_yield_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Yield {
        value: Some(Box::new(Expr::Literal(Literal::Number(1.0, false), span))),
        span,
    };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_yield_without_value_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let expr = Expr::Yield { value: None, span };
    assert_eq!(expr.span(), span);
}

#[test]
fn test_unary_ops_equality() {
    assert_eq!(UnaryOp::Not, UnaryOp::Not);
    assert_eq!(UnaryOp::Neg, UnaryOp::Neg);
    assert_eq!(UnaryOp::Plus, UnaryOp::Plus);
    assert_ne!(UnaryOp::Not, UnaryOp::Neg);
}

#[test]
fn test_binary_ops_remaining() {
    let ops = vec![BinaryOp::Mod, BinaryOp::Le, BinaryOp::Gt, BinaryOp::Ge];
    for op in &ops {
        assert_eq!(*op, *op);
    }
}

// ============================================================================
// From decl.rs inline tests
// ============================================================================

#[test]
fn test_type_annotation() {
    let ty = TypeAnnotation::String;
    assert_eq!(ty, TypeAnnotation::String);

    let ty = TypeAnnotation::Generic("T".to_string());
    assert_eq!(ty, TypeAnnotation::Generic("T".to_string()));
}

#[test]
fn test_param_creation() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let param = Param {
        name: "x".to_string(),
        type_annotation: Some(TypeAnnotation::Number),
        span,
    };
    assert_eq!(param.name, "x");
}

#[test]
fn test_import_kind() {
    let import = ImportKind::Named(vec!["foo".to_string(), "bar".to_string()]);
    match import {
        ImportKind::Named(names) => assert_eq!(names.len(), 2),
        other => unreachable!("Expected Named import, got: {:?}", other),
    }
}

#[test]
fn test_import_kind_default() {
    let import = ImportKind::Default("mod".to_string());
    match import {
        ImportKind::Default(name) => assert_eq!(name, "mod"),
        other => unreachable!("Expected Default import, got: {:?}", other),
    }
}

#[test]
fn test_import_kind_wildcard() {
    let import = ImportKind::Wildcard("everything".to_string());
    match import {
        ImportKind::Wildcard(alias) => assert_eq!(alias, "everything"),
        other => unreachable!("Expected Wildcard import, got: {:?}", other),
    }
}

#[test]
fn test_type_annotation_number() {
    assert_eq!(TypeAnnotation::Number, TypeAnnotation::Number);
}

#[test]
fn test_type_annotation_boolean() {
    assert_eq!(TypeAnnotation::Boolean, TypeAnnotation::Boolean);
}

#[test]
fn test_type_annotation_null() {
    assert_eq!(TypeAnnotation::Null, TypeAnnotation::Null);
}

#[test]
fn test_type_annotation_any() {
    assert_eq!(TypeAnnotation::Any, TypeAnnotation::Any);
}

#[test]
fn test_type_annotation_tool() {
    assert_eq!(TypeAnnotation::Tool, TypeAnnotation::Tool);
}

#[test]
fn test_type_annotation_resource() {
    assert_eq!(TypeAnnotation::Resource, TypeAnnotation::Resource);
}

#[test]
fn test_type_annotation_server() {
    assert_eq!(TypeAnnotation::Server, TypeAnnotation::Server);
}

#[test]
fn test_type_annotation_array() {
    let arr = TypeAnnotation::Array(Box::new(TypeAnnotation::Number));
    assert_eq!(arr, TypeAnnotation::Array(Box::new(TypeAnnotation::Number)));
}

#[test]
fn test_type_annotation_union() {
    let u = TypeAnnotation::Union(vec![TypeAnnotation::String, TypeAnnotation::Number]);
    assert_eq!(
        u,
        TypeAnnotation::Union(vec![TypeAnnotation::String, TypeAnnotation::Number])
    );
}

#[test]
fn test_type_annotation_parameterized() {
    let p = TypeAnnotation::Parameterized {
        base: Box::new(TypeAnnotation::Generic("Stack".to_string())),
        args: vec![TypeAnnotation::Number],
    };
    match p {
        TypeAnnotation::Parameterized { base, args } => {
            assert_eq!(*base, TypeAnnotation::Generic("Stack".to_string()));
            assert_eq!(args.len(), 1);
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_ownership_mode_default() {
    let mode: OwnershipMode = Default::default();
    assert_eq!(mode, OwnershipMode::Owned);
}

#[test]
fn test_ownership_mode_variants() {
    assert_eq!(OwnershipMode::Owned, OwnershipMode::Owned);
    assert_eq!(OwnershipMode::Borrowed, OwnershipMode::Borrowed);
    assert_eq!(OwnershipMode::MutBorrowed, OwnershipMode::MutBorrowed);
    assert_ne!(OwnershipMode::Owned, OwnershipMode::Borrowed);
}

#[test]
fn test_access_modifier_default() {
    let am: AccessModifier = Default::default();
    assert_eq!(am, AccessModifier::Private);
}

#[test]
fn test_access_modifier_variants() {
    assert_eq!(AccessModifier::Public, AccessModifier::Public);
    assert_eq!(AccessModifier::Protected, AccessModifier::Protected);
    assert_ne!(AccessModifier::Public, AccessModifier::Private);
}

#[test]
fn test_var_decl_with_ownership() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let decl = VarDecl {
        name: "x".to_string(),
        is_const: true,
        type_annotation: Some(TypeAnnotation::Number),
        initializer: None,
        ownership: OwnershipMode::Borrowed,
        span,
    };
    assert_eq!(decl.name, "x");
    assert!(decl.is_const);
    assert_eq!(decl.ownership, OwnershipMode::Borrowed);
    assert!(decl.initializer.is_none());
}

#[test]
fn test_pattern_identifier() {
    let p = Pattern::Identifier("x".to_string());
    assert_eq!(p, Pattern::Identifier("x".to_string()));
}

#[test]
fn test_pattern_array() {
    let p = Pattern::Array {
        elements: vec![Pattern::Identifier("a".to_string())],
        rest: Some(Box::new(Pattern::Identifier("rest".to_string()))),
    };
    match p {
        Pattern::Array { elements, rest } => {
            assert_eq!(elements.len(), 1);
            assert!(rest.is_some());
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_pattern_object() {
    let p = Pattern::Object {
        properties: vec![("name".to_string(), Pattern::Identifier("name".to_string()))],
        rest: None,
    };
    match p {
        Pattern::Object { properties, rest } => {
            assert_eq!(properties.len(), 1);
            assert!(rest.is_none());
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_pattern_identifier_default() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let p = Pattern::IdentifierDefault(
        "port".to_string(),
        Expr::Literal(Literal::Number(8080.0, false), span),
    );
    match p {
        Pattern::IdentifierDefault(name, _) => assert_eq!(name, "port"),
        _ => unreachable!(),
    }
}

#[test]
fn test_transport_type_variants() {
    assert_eq!(TransportType::Stdio, TransportType::Stdio);
    assert_eq!(TransportType::SSE, TransportType::SSE);
    assert_ne!(TransportType::Stdio, TransportType::SSE);
}

#[test]
fn test_auth_type_variants() {
    assert_eq!(AuthType::Bearer, AuthType::Bearer);
    assert_eq!(AuthType::Basic, AuthType::Basic);
    assert_eq!(AuthType::ApiKey, AuthType::ApiKey);
    assert_ne!(AuthType::Bearer, AuthType::Basic);
}

#[test]
fn test_generic_param() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let gp = GenericParam {
        name: "T".to_string(),
        constraint: Some("Comparable".to_string()),
        span,
    };
    assert_eq!(gp.name, "T");
    assert_eq!(gp.constraint.unwrap(), "Comparable");
}

#[test]
fn test_class_member_field() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let m = ClassMember::Field {
        access: AccessModifier::Public,
        is_static: true,
        name: "count".to_string(),
        initializer: Some(Expr::Literal(Literal::Number(0.0, false), span)),
        span,
    };
    match m {
        ClassMember::Field {
            access,
            is_static,
            name,
            ..
        } => {
            assert_eq!(access, AccessModifier::Public);
            assert!(is_static);
            assert_eq!(name, "count");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_class_decl() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let cls = ClassDecl {
        is_abstract: false,
        name: "Dog".to_string(),
        parent: Some("Animal".to_string()),
        type_params: vec![],
        implements: vec!["Pet".to_string()],
        members: vec![],
        span,
    };
    assert_eq!(cls.name, "Dog");
    assert_eq!(cls.parent.unwrap(), "Animal");
    assert_eq!(cls.implements, vec!["Pet".to_string()]);
}

#[test]
fn test_trait_method_sig() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let sig = TraitMethodSig {
        name: "serialize".to_string(),
        params: vec![],
        return_type: Some(TypeAnnotation::String),
        span,
    };
    assert_eq!(sig.name, "serialize");
    assert_eq!(sig.return_type.unwrap(), TypeAnnotation::String);
}

// ============================================================================
// From stmt.rs inline tests
// ============================================================================

#[test]
fn test_stmt_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let expr = Expr::Literal(Literal::Number(42.0, false), span);
    let stmt = Stmt::Expr(expr);
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_var_decl() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let decl = VarDecl {
        name: "x".to_string(),
        is_const: false,
        type_annotation: None,
        initializer: Some(Expr::Literal(Literal::Number(42.0, false), span)),
        ownership: Default::default(),
        span,
    };
    let stmt = Stmt::VarDecl(decl);
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_if_stmt() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let condition = Expr::Literal(Literal::Number(1.0, false), span);
    let then_branch = Box::new(Stmt::Block {
        statements: vec![],
        span,
    });
    let stmt = Stmt::If {
        condition,
        then_branch,
        else_branch: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_let_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let stmt = Stmt::Let {
        name: "x".to_string(),
        value: Expr::Literal(Literal::Number(1.0, false), span),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_const_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let stmt = Stmt::Const {
        name: "PI".to_string(),
        value: Expr::Literal(Literal::Number(3.14, true), span),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_assignment_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let stmt = Stmt::Assignment {
        target: Expr::Identifier("x".to_string(), span),
        value: Expr::Literal(Literal::Number(42.0, false), span),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_while_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(3, 1, 30));
    let stmt = Stmt::While {
        condition: Expr::Literal(Literal::Boolean(true), span),
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_for_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(3, 1, 30));
    let stmt = Stmt::For {
        variable: "i".to_string(),
        iterable: Expr::Array {
            elements: vec![],
            span,
        },
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_return_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let stmt = Stmt::Return {
        value: Some(Expr::Literal(Literal::Number(0.0, false), span)),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_break_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 5, 4));
    let stmt = Stmt::Break { span };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_continue_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 8, 7));
    let stmt = Stmt::Continue { span };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_throw_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));
    let stmt = Stmt::Throw {
        value: Expr::Literal(Literal::String("err".to_string()), span),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_switch_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let stmt = Stmt::Switch {
        value: Expr::Literal(Literal::Number(1.0, false), span),
        cases: vec![],
        default: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_try_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let stmt = Stmt::Try {
        try_block: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        catch_clause: None,
        finally_block: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_import_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 30, 29));
    let stmt = Stmt::Import {
        path: "module".to_string(),
        imports: ImportKind::Named(vec!["foo".to_string()]),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_export_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let stmt = Stmt::Export {
        item: Box::new(Stmt::Let {
            name: "x".to_string(),
            value: Expr::Literal(Literal::Number(1.0, false), span),
            span,
        }),
        source: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_function_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(3, 1, 30));
    let stmt = Stmt::Function {
        name: "foo".to_string(),
        params: vec!["x".to_string()],
        body: vec![],
        is_async: false,
        is_generator: false,
        type_params: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_class_stmt_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let stmt = Stmt::Class(ClassDecl {
        is_abstract: false,
        name: "Foo".to_string(),
        parent: None,
        type_params: vec![],
        implements: vec![],
        members: vec![],
        span,
    });
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_match_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let stmt = Stmt::Match {
        value: Expr::Literal(Literal::Number(1.0, false), span),
        arms: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_enum_decl_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let stmt = Stmt::EnumDecl {
        name: "Shape".to_string(),
        variants: vec![EnumVariant {
            name: "Circle".to_string(),
            fields: vec!["radius".to_string()],
            span,
        }],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_for_c_style_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(3, 1, 30));
    let stmt = Stmt::ForCStyle {
        init: None,
        condition: None,
        update: None,
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_for_range_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(3, 1, 30));
    let stmt = Stmt::ForRange {
        start: Expr::Literal(Literal::Number(0.0, false), span),
        stop: Expr::Literal(Literal::Number(10.0, false), span),
        step: None,
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_trait_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let stmt = Stmt::Trait {
        name: "Serializable".to_string(),
        type_params: vec![],
        methods: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_destructure_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let stmt = Stmt::Destructure {
        pattern: Pattern::Identifier("x".to_string()),
        value: Expr::Literal(Literal::Number(1.0, false), span),
        is_const: false,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_spawn_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let stmt = Stmt::Spawn {
        subject_name: "Player".to_string(),
        args: vec![],
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_send_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let stmt = Stmt::Send {
        message: Box::new(Expr::Literal(Literal::String("hello".to_string()), span)),
        target: Box::new(Expr::Identifier("player".to_string(), span)),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_receive_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let stmt = Stmt::Receive {
        variable: "msg".to_string(),
        source: Box::new(Expr::Identifier("inbox".to_string(), span)),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_require_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let stmt = Stmt::Require {
        condition: Box::new(Expr::Literal(Literal::Boolean(true), span)),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_perform_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let stmt = Stmt::Perform {
        action: Box::new(Expr::Identifier("attack".to_string(), span)),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_remember_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 30, 29));
    let stmt = Stmt::Remember {
        content: Box::new(Expr::Literal(Literal::String("data".to_string()), span)),
        store_name: Some("mystore".to_string()),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_recall_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 30, 29));
    let stmt = Stmt::Recall {
        query: Box::new(Expr::Literal(Literal::String("query".to_string()), span)),
        store_name: None,
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_forget_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 30, 29));
    let stmt = Stmt::Forget {
        target: Box::new(Expr::Literal(Literal::String("id".to_string()), span)),
        store_name: Some("store".to_string()),
        span,
    };
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_decl_agent_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let decl = Decl::Agent {
        name: "MyAgent".to_string(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn test_decl_action_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let decl = Decl::Action {
        name: "act".to_string(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn test_decl_tool_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let decl = Decl::Tool {
        name: "tool".to_string(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn test_decl_resource_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let decl = Decl::Resource {
        name: "res".to_string(),
        fields: vec![],
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn test_decl_import_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 20, 19));
    let decl = Decl::Import {
        module: "mod".to_string(),
        alias: Some("m".to_string()),
        span,
    };
    assert_eq!(decl.span(), span);
}

#[test]
fn test_decl_spans_exhaustive() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9));

    // Test remaining Decl variants all return their span
    let decls: Vec<Decl> = vec![
        Decl::Constitution {
            name: "c".to_string(),
            description: None,
            laws: vec![],
            span,
        },
        Decl::Law {
            name: "l".to_string(),
            description: "d".to_string(),
            enforcement_level: "mandatory".to_string(),
            rules: vec![],
            span,
        },
        Decl::Council {
            name: "c".to_string(),
            constitution: "const".to_string(),
            members: vec![],
            rules: vec![],
            span,
        },
        Decl::Rule {
            name: "r".to_string(),
            conditions: vec![],
            actions: vec![],
            priority: 1,
            span,
        },
        Decl::Swarm {
            name: "s".to_string(),
            agents: vec![],
            strategy: "parallel".to_string(),
            span,
        },
        Decl::Provider {
            name: "p".to_string(),
            fields: vec![],
            span,
        },
        Decl::Protocol {
            name: "p".to_string(),
            execution: None,
            governance: None,
            timeout: None,
            session: vec![],
            span,
        },
        Decl::Governance {
            name: "g".to_string(),
            base_type: "democracy".to_string(),
            fields: vec![],
            span,
        },
        Decl::Role {
            name: "r".to_string(),
            capabilities: vec![],
            fields: vec![],
            span,
        },
        Decl::Store {
            name: "s".to_string(),
            fields: vec![],
            span,
        },
        Decl::Strategy {
            name: "s".to_string(),
            execution: None,
            governance: None,
            timeout: None,
            permissions: vec![],
            realm: None,
            session: vec![],
            span,
        },
        Decl::Community {
            name: "comm".to_string(),
            members: vec![],
            councils: vec![],
            culture: CultureDecl {
                values: vec![],
                norms: vec![],
                communication_style: "formal".to_string(),
                span,
            },
            span,
        },
        Decl::Subject {
            of_subject: None,
            ability_defs: vec![],
            name: "sub".to_string(),
            decorators: vec![],
            roles: vec![],
            states: vec![],
            capabilities: vec![],
            intents: vec![],
            uses: vec![],
            memory: vec![],
            perception: vec![],
            fields: vec![],
            span,
        },
        Decl::Relation {
            subject_a: "A".to_string(),
            subject_b: "B".to_string(),
            fields: vec![],
            span,
        },
        Decl::Effect {
            params: vec![],
            event_name: "Damage".to_string(),
            body: vec![],
            span,
        },
        Decl::Entity {
            name: "e".to_string(),
            fields: vec![],
            span,
        },
        Decl::StateMachine {
            name: "sm".to_string(),
            fields: vec![],
            span,
        },
        Decl::Event {
            name: "ev".to_string(),
            fields: vec![],
            span,
        },
        Decl::Contract {
            name: "c".to_string(),
            fields: vec![],
            span,
        },
        Decl::Treaty {
            name: "t".to_string(),
            fields: vec![],
            span,
        },
        Decl::Music {
            kind: "note".to_string(),
            name: "m".to_string(),
            fields: vec![],
            span,
        },
        Decl::UiApp {
            name: "app".to_string(),
            entry_screen: None,
            screens: vec![],
            components: vec![],
            span,
        },
        Decl::Deploy {
            name: "d".to_string(),
            targets: vec![],
            providers: vec![],
            fields: vec![],
            span,
        },
    ];
    for decl in decls {
        assert_eq!(decl.span(), span);
    }
}

#[test]
fn test_match_pattern_wildcard() {
    let p = MatchPattern::Wildcard;
    assert_eq!(p, MatchPattern::Wildcard);
}

#[test]
fn test_match_pattern_literal() {
    let p = MatchPattern::Literal(Literal::Number(42.0, false));
    assert_eq!(p, MatchPattern::Literal(Literal::Number(42.0, false)));
}

#[test]
fn test_match_pattern_identifier() {
    let p = MatchPattern::Identifier("x".to_string());
    assert_eq!(p, MatchPattern::Identifier("x".to_string()));
}

#[test]
fn test_match_pattern_enum_variant() {
    let p = MatchPattern::EnumVariant {
        enum_name: "Shape".to_string(),
        variant: "Circle".to_string(),
        binding: Some("r".to_string()),
    };
    match p {
        MatchPattern::EnumVariant {
            enum_name,
            variant,
            binding,
        } => {
            assert_eq!(enum_name, "Shape");
            assert_eq!(variant, "Circle");
            assert_eq!(binding.unwrap(), "r");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_stmt_decl_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let stmt = Stmt::Decl(Decl::Agent {
        name: "A".to_string(),
        fields: vec![],
        span,
    });
    assert_eq!(stmt.span(), span);
}

#[test]
fn test_mcp_server_span() {
    let span = Span::new(Position::new(1, 1, 0), Position::new(5, 1, 50));
    let stmt = Stmt::McpServer(McpServerDecl {
        name: "srv".to_string(),
        config: ServerConfig {
            transport: TransportType::Stdio,
            command: Some("cmd".to_string()),
            args: vec![],
            url: None,
            auth: None,
        },
        fields: vec![],
        tools: vec![],
        span,
    });
    assert_eq!(stmt.span(), span);
}
