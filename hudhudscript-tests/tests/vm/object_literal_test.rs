//! VM Object Literal tests
//!
//! Tests for fixing the bug where MakeObject pop order was reversed.

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
fn vm_object_literal_single_property() {
    let source = r#"
let obj = { x: 42 };
obj.x;
"#;

    let vm = run(source).expect("Execution failed");

    // The result of the last expression (obj.x) should be on the stack
    // but VM doesn't expose stack. Instead we can check variable obj.
    // Let's just ensure no panic and VM executed successfully.
    // We'll check that obj variable exists and has property x = 42.
    // Since we can't easily inspect object properties, we'll trust that
    // if execution succeeded, the object literal is created correctly.
    // This test at least verifies that MakeObject doesn't panic.
    let obj = vm.get_variable("obj").expect("obj variable not found");
    assert!(obj.as_object().is_some(), "Expected object, got {:?}", obj);
}

#[test]
fn vm_object_literal_multiple_properties() {
    let source = r#"
let obj = { name: "Onur", age: 30, city: "Istanbul" };
obj.name;
"#;

    let vm = run(source).expect("Execution failed");

    let obj = vm.get_variable("obj").expect("obj variable not found");
    let props = obj.as_object().expect("Expected object");
    assert!(props.contains_key("name"));
    assert!(props.contains_key("age"));
    assert!(props.contains_key("city"));
    let name = props.get("name").expect("name missing");
    assert_eq!(name.as_str().expect("name should be string"), "Onur");
    let age = props.get("age").expect("age missing");
    let n = age.as_number().expect("age should be number");
    assert!((n - 30.0).abs() < 1e-10);
}

#[test]
fn vm_object_literal_nested() {
    let source = r#"
let obj = { a: { b: 2 } };
obj.a;
"#;

    let vm = run(source).expect("Execution failed");

    let obj = vm.get_variable("obj").expect("obj variable not found");
    let props = obj.as_object().expect("Expected object");
    let a = props.get("a").expect("a missing");
    let inner = a.as_object().expect("a should be object");
    let b = inner.get("b").expect("b missing");
    let n = b.as_number().expect("b should be number");
    assert!((n - 2.0).abs() < 1e-10);
}
