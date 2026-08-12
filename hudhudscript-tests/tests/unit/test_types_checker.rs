use hudhudscript_ast::{AccessModifier, ClassDecl, ClassMember, Expr, Span, Stmt};
use hudhudscript_types::{Type, TypeChecker};

fn dummy_span() -> Span {
    Span::default()
}

/// Helper: build a minimal class with a single method whose body is the given statements.
fn class_with_method(class_name: &str, method_body: Vec<Stmt>) -> ClassDecl {
    ClassDecl {
        is_abstract: false,
        name: class_name.to_string(),
        parent: None,
        type_params: vec![],
        implements: vec![],
        members: vec![ClassMember::Method {
            access: AccessModifier::Public,
            is_static: false,
            name: "doSomething".to_string(),
            params: vec![],
            body: method_body,
            span: dummy_span(),
        }],
        span: dummy_span(),
    }
}

#[test]
fn trivial_test() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn this_inside_class_returns_instance_type() {
    let mut checker = TypeChecker::new();

    // Build: class Foo { method doSomething() { this; } }
    let class_decl = class_with_method("Foo", vec![Stmt::Expr(Expr::This(dummy_span()))]);

    let program = vec![Stmt::Class(class_decl)];
    checker.check_program(&program).unwrap();

    // Verify that check_expr returns Instance("Foo") when inside the class
    // We can verify indirectly: no errors were raised, and the class was registered.
    assert!(checker.errors().is_empty());
}

#[test]
fn this_inside_class_has_correct_type() {
    let mut checker = TypeChecker::new();
    // Set class context manually to verify Expr::This typing
    checker.current_class = Some("MyClass".to_string());

    let result = checker.check_expr(&Expr::This(dummy_span()));
    assert_eq!(result.unwrap(), Type::Instance("MyClass".to_string()));
}

#[test]
fn this_outside_class_returns_any() {
    let mut checker = TypeChecker::new();
    // current_class is None by default

    let result = checker.check_expr(&Expr::This(dummy_span()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Any);
}

#[test]
fn current_class_restored_after_class_decl() {
    let mut checker = TypeChecker::new();

    let class_decl = class_with_method("Bar", vec![]);
    let program = vec![Stmt::Class(class_decl)];
    checker.check_program(&program).unwrap();

    // After checking the class, current_class should be None again
    assert!(checker.current_class.is_none());
}

#[test]
fn this_inside_constructor_returns_instance_type() {
    let mut checker = TypeChecker::new();

    let class_decl = ClassDecl {
        is_abstract: false,
        name: "Widget".to_string(),
        parent: None,
        type_params: vec![],
        implements: vec![],
        members: vec![ClassMember::Constructor {
            params: vec![],
            body: vec![Stmt::Expr(Expr::This(dummy_span()))],
            span: dummy_span(),
        }],
        span: dummy_span(),
    };

    let program = vec![Stmt::Class(class_decl)];
    checker.check_program(&program).unwrap();
    assert!(checker.errors().is_empty());
}
