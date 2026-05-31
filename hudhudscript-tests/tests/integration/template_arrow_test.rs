// Tests for template strings and arrow functions

use hudhudscript_ast::{Expr, Stmt};
use hudhudscript_parser::parse;

#[test]
fn test_template_string_simple() {
    let source = r#"
        var name = "Ali";
        var greeting = `Hello, ${name}!`;
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse template string: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 2);

    // Check second statement is a var declaration with template string
    // var now produces VarDecl (#744), not Stmt::Let
    match &stmts[1] {
        Stmt::VarDecl(decl) => {
            if let Some(ref value) = decl.initializer {
                assert!(matches!(value, Expr::TemplateString { .. }));
            } else {
                panic!("Expected initializer");
            }
        }
        Stmt::Let { value, .. } => {
            assert!(matches!(value, Expr::TemplateString { .. }));
        }
        _ => panic!(
            "Expected VarDecl or Let statement, got {:?}",
            std::mem::discriminant(&stmts[1])
        ),
    }
}

#[test]
fn test_template_string_multiple_interpolations() {
    let source = r#"
        var x = 10;
        var y = 20;
        var msg = `x=${x}, y=${y}`;
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_arrow_function_expression() {
    let source = r#"
        var add = (x, y) => x + y;
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse arrow function: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 1);

    // var now produces VarDecl (#744)
    let value = match &stmts[0] {
        Stmt::VarDecl(decl) => decl.initializer.as_ref().expect("Expected initializer"),
        Stmt::Let { value, .. } => value,
        _ => panic!("Expected VarDecl or Let"),
    };
    if let Expr::ArrowFunction { params, .. } = value {
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "x");
        assert_eq!(params[1], "y");
    } else {
        panic!("Expected ArrowFunction expression");
    }
}

#[test]
fn test_arrow_function_block() {
    let source = r#"
        var greet = (name) => {
            var msg = `Hello, ${name}!`;
            return msg;
        };
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse arrow function with block: {:?}",
        result.err()
    );
}

#[test]
fn test_arrow_function_single_param() {
    let source = r#"
        var square = (x) => x * x;
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse single param arrow function: {:?}",
        result.err()
    );
}

#[test]
fn test_arrow_function_no_params() {
    let source = r#"
        var getFortyTwo = () => 42;
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse no-param arrow function: {:?}",
        result.err()
    );
}

#[test]
fn test_arrow_function_call() {
    let source = r#"
        var add = (x, y) => x + y;
        var result = add(5, 3);
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse arrow function call: {:?}",
        result.err()
    );
}

#[test]
fn test_template_in_arrow() {
    let source = r#"
        var greet = (name) => `Hello, ${name}!`;
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse template in arrow function: {:?}",
        result.err()
    );
}
