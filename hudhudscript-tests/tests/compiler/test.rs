//! Compiler tests for enhanced features

use hudhudscript_compiler::{Compiler, VM};
use hudhudscript_parser::parse;

#[test]
fn test_compile_while_loop() {
    let source = r#"
        let x = 0;
        while (x < 5) {
            x = x + 1;
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_const() {
    let source = r#"
        const PI = 3.14;
        let area = PI * 2;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_assignment() {
    let source = r#"
        let x = 10;
        x = 20;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_return() {
    let source = r#"
        let x = 10;
        return x;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_unary_not() {
    let source = r#"
        let x = true;
        let y = !x;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_unary_neg() {
    let source = r#"
        let x = 10;
        let y = -x;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_array() {
    let source = r#"
        let arr = [1, 2, 3];
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_object() {
    let source = r#"
        let obj = { x: 10, y: 20 };
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_array_index() {
    let source = r#"
        let arr = [1, 2, 3];
        let x = arr[0];
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_object_member() {
    let source = r#"
        let obj = { x: 10 };
        let val = obj.x;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_logical_and() {
    let source = r#"
        let x = true && false;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_compile_logical_or() {
    let source = r#"
        let x = true || false;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&statements).expect("Failed to compile");

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

// ===========================================================================
// Issue #141 — User-defined functions and new built-ins
// ===========================================================================

// ---------------------------------------------------------------------------
// User-defined function compilation and execution
// ---------------------------------------------------------------------------

#[test]
fn test_user_defined_function_no_args() {
    let stmts = parse(
        r#"
        function greet() { return 42; }
        let result = greet();
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("result").and_then(|v| v.as_number()) == Some(42.0));
}

#[test]
fn test_user_defined_function_with_args() {
    let stmts = parse(
        r#"
        function add(a, b) { return a + b; }
        let result = add(3, 4);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("result").and_then(|v| v.as_number()) == Some(7.0));
}

#[test]
fn test_user_defined_function_with_local_var() {
    let stmts = parse(
        r#"
        function square(n) { let sq = n * n; return sq; }
        let result = square(9);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("result").and_then(|v| v.as_number()) == Some(81.0));
}

#[test]
fn test_function_returns_string() {
    let stmts = parse(
        r#"
        function echo(s) { return s; }
        let result = echo("hello");
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(
        vm.get_variable("result")
            .and_then(|v| v.as_str())
            .as_deref()
            == Some("hello")
    );
}

#[test]
fn test_function_with_conditional_return() {
    let stmts = parse(
        r#"
        function abs_val(x) {
            if (x < 0) { return x * -1; } else { return x; }
        }
        let pos = abs_val(-5);
        let same = abs_val(3);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("pos").and_then(|v| v.as_number()) == Some(5.0));
    assert!(vm.get_variable("same").and_then(|v| v.as_number()) == Some(3.0));
}

#[test]
fn test_function_with_while_loop() {
    let stmts = parse(
        r#"
        function sum_to(limit) {
            let i = 1;
            let acc = 0;
            while (i <= limit) { acc = acc + i; i = i + 1; }
            return acc;
        }
        let result = sum_to(5);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    // 1+2+3+4+5 = 15
    assert!(vm.get_variable("result").and_then(|v| v.as_number()) == Some(15.0));
}

#[test]
fn test_function_stored_as_variable() {
    let stmts = parse(r#"function noop() { return 0; }"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(
        vm.get_variable("noop")
            .map(|v| v.as_function_data().is_some())
            == Some(true)
    );
}

#[test]
fn test_multiple_function_calls() {
    let stmts = parse(
        r#"
        function double(x) { return x * 2; }
        function triple(x) { return x * 3; }
        let a = double(5);
        let b = triple(5);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("a").and_then(|v| v.as_number()) == Some(10.0));
    assert!(vm.get_variable("b").and_then(|v| v.as_number()) == Some(15.0));
}

#[test]
fn test_function_result_in_expression() {
    let stmts = parse(
        r#"
        function inc(x) { return x + 1; }
        let result = inc(10) + inc(20);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    // 11 + 21 = 32
    assert!(vm.get_variable("result").and_then(|v| v.as_number()) == Some(32.0));
}

// ---------------------------------------------------------------------------
// Built-in: len()
// ---------------------------------------------------------------------------

#[test]
fn test_builtin_len_string() {
    let stmts = parse(r#"let n = len("hello");"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(5.0));
}

#[test]
fn test_builtin_len_array() {
    let stmts = parse(r#"let n = len([1, 2, 3, 4]);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(4.0));
}

#[test]
fn test_builtin_len_object() {
    let stmts = parse(r#"let n = len({ a: 1, b: 2, c: 3 });"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(3.0));
}

// ---------------------------------------------------------------------------
// Built-in: type()
// ---------------------------------------------------------------------------

#[test]
fn test_builtin_type_number() {
    let stmts = parse(r#"let t = type(42);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("t").and_then(|v| v.as_str()).as_deref() == Some("number"));
}

#[test]
fn test_builtin_type_string() {
    let stmts = parse(r#"let t = type("hello");"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("t").and_then(|v| v.as_str()).as_deref() == Some("string"));
}

#[test]
fn test_builtin_type_boolean() {
    let stmts = parse(r#"let t = type(true);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("t").and_then(|v| v.as_str()).as_deref() == Some("boolean"));
}

#[test]
fn test_builtin_type_null() {
    let stmts = parse(r#"let t = type(null);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("t").and_then(|v| v.as_str()).as_deref() == Some("null"));
}

#[test]
fn test_builtin_type_array() {
    let stmts = parse(r#"let t = type([1, 2]);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("t").and_then(|v| v.as_str()).as_deref() == Some("array"));
}

#[test]
fn test_builtin_type_object() {
    let stmts = parse(r#"let t = type({ a: 1 });"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("t").and_then(|v| v.as_str()).as_deref() == Some("object"));
}

#[test]
fn test_builtin_type_function() {
    let stmts = parse(
        r#"
        function f() { return 1; }
        let t = type(f);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("t").and_then(|v| v.as_str()).as_deref() == Some("function"));
}

// ---------------------------------------------------------------------------
// Built-in: toString()
// ---------------------------------------------------------------------------

#[test]
fn test_builtin_tostring_number() {
    let stmts = parse(r#"let s = toString(42);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("s").and_then(|v| v.as_str()).as_deref() == Some("42"));
}

#[test]
fn test_builtin_tostring_boolean() {
    let stmts = parse(r#"let s = toString(true);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("s").and_then(|v| v.as_str()).as_deref() == Some("true"));
}

#[test]
fn test_builtin_tostring_null() {
    let stmts = parse(r#"let s = toString(null);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("s").and_then(|v| v.as_str()).as_deref() == Some("null"));
}

// ---------------------------------------------------------------------------
// Built-in: toNumber() / toBoolean()
// ---------------------------------------------------------------------------

#[test]
fn test_builtin_tonumber_string() {
    let stmts = parse(r#"let n = toNumber("3.14");"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!((vm.get_variable("n").and_then(|v| v.as_number()).unwrap() - 3.14).abs() < 1e-9);
}

#[test]
fn test_builtin_toboolean_zero_is_false() {
    let stmts = parse(r#"let b = toBoolean(0);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert_eq!(vm.get_variable("b").and_then(|v| v.as_bool()), Some(false));
}

#[test]
fn test_builtin_toboolean_nonzero_is_true() {
    let stmts = parse(r#"let b = toBoolean(1);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert_eq!(vm.get_variable("b").and_then(|v| v.as_bool()), Some(true));
}

// ---------------------------------------------------------------------------
// Built-in: math helpers
// ---------------------------------------------------------------------------

#[test]
fn test_builtin_sqrt() {
    let stmts = parse(r#"let n = sqrt(16);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!((vm.get_variable("n").and_then(|v| v.as_number()).unwrap() - 4.0).abs() < 1e-9);
}

#[test]
fn test_builtin_abs() {
    let stmts = parse(r#"let n = abs(-7);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(7.0));
}

#[test]
fn test_builtin_floor() {
    let stmts = parse(r#"let n = floor(3.9);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(3.0));
}

#[test]
fn test_builtin_ceil() {
    let stmts = parse(r#"let n = ceil(3.1);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(4.0));
}

#[test]
fn test_builtin_round() {
    let stmts = parse(r#"let n = round(3.5);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(4.0));
}

#[test]
fn test_builtin_min() {
    let stmts = parse(r#"let n = min(10, 3);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(3.0));
}

#[test]
fn test_builtin_max() {
    let stmts = parse(r#"let n = max(10, 3);"#).expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    assert!(vm.get_variable("n").and_then(|v| v.as_number()) == Some(10.0));
}

// ---------------------------------------------------------------------------
// Built-in: keys() / values()
// ---------------------------------------------------------------------------

#[test]
fn test_builtin_keys() {
    let stmts = parse(
        r#"
        let obj = { b: 2, a: 1 };
        let k = keys(obj);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    if let Some(arr) = vm.get_variable("k").and_then(|v| v.as_array()) {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_str().as_deref(), Some("a"));
        assert_eq!(arr[1].as_str().as_deref(), Some("b"));
    } else {
        panic!(
            "keys() should return an array, got {:?}",
            vm.get_variable("k")
        );
    }
}

#[test]
fn test_builtin_values() {
    let stmts = parse(
        r#"
        let obj = { a: 10, b: 20 };
        let v = values(obj);
    "#,
    )
    .expect("parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("execute");
    if let Some(arr) = vm.get_variable("v").and_then(|v| v.as_array()) {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_number(), Some(10.0));
        assert_eq!(arr[1].as_number(), Some(20.0));
    } else {
        panic!("values() should return an array");
    }
}

// ── WI-1.2: needs_async detection ──────────────────────────────

#[test]
fn test_needs_async_sync_script() {
    let source = "var x = 5; var y = x + 3;";
    let stmts = parse(source).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&stmts).unwrap();
    assert!(!bc.needs_async, "pure sync script should not need async");
}

#[test]
fn test_needs_async_with_spawn() {
    let source = "agent X { } spawn X;";
    let stmts = parse(source).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&stmts).unwrap();
    assert!(bc.needs_async, "script with spawn should need async");
}

#[test]
fn test_needs_async_default_true() {
    let bc = hudhudscript_bytecode::Bytecode::new();
    assert!(bc.needs_async, "default should be true (safe)");
}
