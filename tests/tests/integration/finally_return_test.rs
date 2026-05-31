use hudhud_script_tests::vm_interpreter::runtime_codes;
use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn eval_function(
    source: &str,
) -> Result<Value16, hudhud_script_tests::vm_interpreter::RuntimeError> {
    let statements = parse(source).map_err(|e| {
        hudhud_script_tests::vm_interpreter::runtime_codes::custom(format!("Parse error: {:?}", e))
    })?;
    let mut interpreter = Interpreter::new();
    interpreter.execute(&statements)
}

#[test]
fn finally_return_overrides_try_return() {
    // function f() { try { return 1; } finally { return 2; } }
    let source = r#"
        function f() {
            try { return 1; }
            finally { return 2; }
        }
        f()
    "#;
    let result = eval_function(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(
        result.unwrap(),
        Value16::number(2.0),
        "finally return should override try return"
    );
}

#[test]
fn finally_return_overrides_catch_return() {
    // function f() { try { throw "err"; } catch(e) { return 1; } finally { return 2; } }
    let source = r#"
        function f() {
            try { throw "err"; }
            catch(e) { return 1; }
            finally { return 2; }
        }
        f()
    "#;
    let result = eval_function(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(
        result.unwrap(),
        Value16::number(2.0),
        "finally return should override catch return"
    );
}

#[test]
fn finally_without_return_preserves_try_return() {
    // function f() { try { return 1; } finally { let x = 2; } }
    let source = r#"
        function f() {
            try { return 1; }
            finally { let x = 2; }
        }
        f()
    "#;
    let result = eval_function(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(
        result.unwrap(),
        Value16::number(1.0),
        "try return should be preserved when finally has no return"
    );
}

#[test]
fn finally_return_preserves_try_error() {
    // function f() { try { throw "err"; } finally { return 2; } }
    let source = r#"
        function f() {
            try { throw "err"; }
            finally { return 2; }
        }
        f()
    "#;
    let result = eval_function(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(
        result.unwrap(),
        Value16::number(2.0),
        "finally return should override thrown error"
    );
}

#[test]
fn finally_throw_overrides_try_return() {
    // function f() { try { return 1; } finally { throw "err2"; } }
    let source = r#"
        function f() {
            try { return 1; }
            finally { throw "err2"; }
        }
        f()
    "#;
    let result = eval_function(source);
    assert!(result.is_err(), "Should throw from finally");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("err2"), "Should throw 'err2' from finally");
}
