//! VM Continue instruction tests
//!
//! Tests for fixing the bug where Instruction::Continue was implemented as `break`
//! instead of jumping to the loop header.
//!
//! Test scripts use the language's block-form if (`if (cond) { stmt }`) — the
//! grammar requires a brace-delimited block after a condition, there is no
//! braceless single-statement `if (cond) stmt;` form. When these tests were
//! originally written they used the braceless form and had never parsed
//! successfully; the rewrite below keeps every case's intent intact but uses
//! the syntax the parser actually accepts, so the VM's Continue handler is
//! exercised end-to-end.

use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

/// Helper to run source code through parser → compiler → VM
fn run(source: &str) -> CompileResult<VM> {
    let stmts = parse(source).map_err(|e| {
        hudhudscript_bytecode::error::compile_codes::runtime_error(format!("Parse error: {:?}", e))
    })?;
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts)?;
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

#[test]
fn vm_continue_skips_iteration() {
    let source = r#"
let result = [];
for (i in [1, 2, 3]) {
    if (i == 2) { continue }
    result.push(i);
}
result;
"#;

    let vm = run(source).expect("Execution failed");

    // Check that result variable contains [1, 3]
    let result = vm
        .get_variable("result")
        .expect("result variable not found");
    let arr = result.as_array().expect("Expected array");
    assert_eq!(arr.len(), 2);
    let n1 = arr[0].as_number().expect("Expected number");
    let n2 = arr[1].as_number().expect("Expected number");
    assert!((n1 - 1.0).abs() < 1e-10);
    assert!((n2 - 3.0).abs() < 1e-10);
}

#[test]
fn vm_continue_in_nested_loops() {
    let source = r#"
let result = [];
for (x in [1, 2]) {
    for (y in [10, 20]) {
        if (y == 20) { continue } // Skip y=20 in inner loop
        result.push(x * y);
    }
}
result;
"#;

    let vm = run(source).expect("Execution failed");

    // Expected: x=1,y=10 → 10; x=1,y=20 skipped; x=2,y=10 → 20; x=2,y=20 skipped
    // So result should be [10, 20]
    let result = vm
        .get_variable("result")
        .expect("result variable not found");
    let arr = result.as_array().expect("Expected array");
    assert_eq!(arr.len(), 2);
    let n1 = arr[0].as_number().expect("Expected number");
    let n2 = arr[1].as_number().expect("Expected number");
    assert!((n1 - 10.0).abs() < 1e-10);
    assert!((n2 - 20.0).abs() < 1e-10);
}

#[test]
fn vm_continue_with_break() {
    let source = r#"
let result = [];
for (i in [1, 2, 3, 4, 5]) {
    if (i == 3) { continue }
    if (i == 4) { break }
    result.push(i);
}
result;
"#;

    let vm = run(source).expect("Execution failed");

    // Expected: i=1 → push, i=2 → push, i=3 → skip, i=4 → break
    // So result should be [1, 2]
    let result = vm
        .get_variable("result")
        .expect("result variable not found");
    let arr = result.as_array().expect("Expected array");
    assert_eq!(arr.len(), 2);
    let n1 = arr[0].as_number().expect("Expected number");
    let n2 = arr[1].as_number().expect("Expected number");
    assert!((n1 - 1.0).abs() < 1e-10);
    assert!((n2 - 2.0).abs() < 1e-10);
}

#[test]
fn vm_continue_on_first_iteration() {
    let source = r#"
let result = [];
for (i in [1, 2, 3]) {
    if (i == 1) { continue }
    result.push(i);
}
result;
"#;

    let vm = run(source).expect("Execution failed");

    // Expected: skip i=1, push i=2, i=3
    let result = vm
        .get_variable("result")
        .expect("result variable not found");
    let arr = result.as_array().expect("Expected array");
    assert_eq!(arr.len(), 2);
    let n1 = arr[0].as_number().expect("Expected number");
    let n2 = arr[1].as_number().expect("Expected number");
    assert!((n1 - 2.0).abs() < 1e-10);
    assert!((n2 - 3.0).abs() < 1e-10);
}

// Test that continue works with strings (iterating over characters)
#[test]
fn vm_continue_string_iteration() {
    let source = r#"
let result = [];
for (ch in "abc") {
    if (ch == "b") { continue }
    result.push(ch);
}
result;
"#;

    let vm = run(source).expect("Execution failed");

    let result = vm
        .get_variable("result")
        .expect("result variable not found");
    let arr = result.as_array().expect("Expected array");
    assert_eq!(arr.len(), 2);
    let s1 = arr[0].as_str().expect("Expected string");
    let s2 = arr[1].as_str().expect("Expected string");
    assert_eq!(s1, "a");
    assert_eq!(s2, "c");
}
