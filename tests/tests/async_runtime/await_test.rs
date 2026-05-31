use hudhudscript_ast::{Expr, Stmt};
use hudhudscript_parser::parse;

#[test]
fn test_function_declaration() {
    let source = r#"
        function add(x, y) {
            return x + y;
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse function: {:?}",
        result.err()
    );

    let program = result.unwrap();
    assert_eq!(program.len(), 1);

    match &program[0] {
        Stmt::Function {
            name,
            params,
            is_async,
            ..
        } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "x");
            assert_eq!(params[1], "y");
            assert_eq!(*is_async, false);
        }
        _ => panic!("Expected Function statement"),
    }
}

#[test]
fn test_async_function_declaration() {
    let source = r#"
        async function fetchData(url) {
            let response = await fetch(url);
            return response;
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse async function: {:?}",
        result.err()
    );

    let program = result.unwrap();
    assert_eq!(program.len(), 1);

    match &program[0] {
        Stmt::Function {
            name,
            params,
            is_async,
            ..
        } => {
            assert_eq!(name, "fetchData");
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], "url");
            assert_eq!(*is_async, true);
        }
        _ => panic!("Expected Function statement"),
    }
}

#[test]
fn test_await_expression() {
    let source = r#"
        async function test() {
            let result = await somePromise;
            return result;
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse await expression: {:?}",
        result.err()
    );

    let program = result.unwrap();
    assert_eq!(program.len(), 1);

    match &program[0] {
        Stmt::Function { body, .. } => {
            // Check that body contains await expression
            assert!(!body.is_empty());

            // First statement should be let with await
            match &body[0] {
                Stmt::Let { value, .. } => {
                    match value {
                        Expr::Await { .. } => {
                            // Success!
                        }
                        _ => panic!(
                            "Expected Await expression in let statement, got: {:?}",
                            value
                        ),
                    }
                }
                _ => panic!("Expected Let statement, got: {:?}", &body[0]),
            }
        }
        _ => panic!("Expected Function statement"),
    }
}

#[test]
fn test_function_with_no_params() {
    let source = r#"
        function greet() {
            return "Hello";
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse function with no params: {:?}",
        result.err()
    );

    let program = result.unwrap();
    assert_eq!(program.len(), 1);

    match &program[0] {
        Stmt::Function { name, params, .. } => {
            assert_eq!(name, "greet");
            assert_eq!(params.len(), 0);
        }
        _ => panic!("Expected Function statement"),
    }
}

#[test]
fn test_nested_await() {
    let source = r#"
        async function test() {
            let x = await await promise;
            return x;
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse nested await: {:?}",
        result.err()
    );
}

#[test]
fn test_turkish_async_function() {
    let source = r#"
        eşzamansız işlev veriAl(url) {
            let yanıt = bekle fetch(url);
            return yanıt;
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Turkish async function: {:?}",
        result.err()
    );

    let program = result.unwrap();
    assert_eq!(program.len(), 1);

    match &program[0] {
        Stmt::Function { name, is_async, .. } => {
            assert_eq!(name, "veriAl");
            assert_eq!(*is_async, true);
        }
        _ => panic!("Expected Function statement"),
    }
}
