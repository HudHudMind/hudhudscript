//! VM integration tests for Batch 5: I/O & Terminal (v0.4.38)

use hudhudscript_bytecode::{ObjMap, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn vm_run_and_get(code: &str, var: &str) -> (hudhudscript_vm::VM, hudhudscript_bytecode::Value16) {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("VM execution failed");
    let val = vm
        .get_variable(var)
        .cloned()
        .unwrap_or_else(|| panic!("variable \'{}\' not found", var));
    (vm, val)
}

fn assert_string(
    (_vm, val): (hudhudscript_vm::VM, hudhudscript_bytecode::Value16),
    expected: &str,
) {
    match val.as_str() {
        Some(s) => assert_eq!(s, expected, "Expected '{}', got '{}'", expected, s),
        other => panic!("Expected String(\"{}\"), got {:?}", expected, other),
    }
}

fn unwrap_string((_vm, val): (hudhudscript_vm::VM, hudhudscript_bytecode::Value16)) -> String {
    match val.as_str() {
        Some(s) => s.to_string(),
        other => panic!("Expected String, got {:?}", other),
    }
}

fn unwrap_object((_vm, val): (hudhudscript_vm::VM, hudhudscript_bytecode::Value16)) -> ObjMap {
    match val.as_object() {
        Some(obj) => obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        other => panic!("Expected Object, got {:?}", other),
    }
}

// ── Terminal module ──────────────────────────────────────────────

#[test]
fn test_vm_terminal_red() {
    let s = unwrap_string(vm_run_and_get(r#"var x = Terminal.red("error");"#, "x"));
    assert!(s.contains("\x1b[31m"));
    assert!(s.contains("error"));
    assert!(s.contains("\x1b[0m"));
}

#[test]
fn test_vm_terminal_green() {
    let s = unwrap_string(vm_run_and_get(r#"var x = Terminal.green("ok");"#, "x"));
    assert!(s.contains("\x1b[32m"));
}

#[test]
fn test_vm_terminal_bold() {
    let s = unwrap_string(vm_run_and_get(r#"var x = Terminal.bold("title");"#, "x"));
    assert!(s.contains("\x1b[1m"));
    assert!(s.contains("title"));
}

#[test]
fn test_vm_terminal_strip() {
    assert_string(
        vm_run_and_get(
            r#"var colored = Terminal.red("hello");
var x = Terminal.strip(colored);"#,
            "x",
        ),
        "hello",
    );
}

// ── style function ──────────────────────────────────────────────

#[test]
fn test_vm_style_with_color() {
    let s = unwrap_string(vm_run_and_get(
        r#"var x = style("msg", { color: "cyan" });"#,
        "x",
    ));
    assert!(s.contains("\x1b[")); // has ANSI
    assert!(s.contains("36")); // cyan
}

#[test]
fn test_vm_style_plain() {
    assert_string(vm_run_and_get(r#"var x = style("plain");"#, "x"), "plain");
}

// ── log module ──────────────────────────────────────────────────

#[test]
fn test_vm_log_info() {
    let obj = unwrap_object(vm_run_and_get(r#"var x = log.info("started");"#, "x"));
    match obj.get("level").and_then(|v| v.as_str()) {
        Some(s) => assert_eq!(s, "info"),
        other => panic!("Expected level='info', got {:?}", other),
    }
    match obj.get("message").and_then(|v| v.as_str()) {
        Some(s) => assert_eq!(s, "started"),
        other => panic!("Expected message='started', got {:?}", other),
    }
}

#[test]
fn test_vm_log_error() {
    let obj = unwrap_object(vm_run_and_get(r#"var x = log.error("crash");"#, "x"));
    match obj.get("level").and_then(|v| v.as_str()) {
        Some(s) => assert_eq!(s, "error"),
        other => panic!("Expected level='error', got {:?}", other),
    }
}

// ── exec module ─────────────────────────────────────────────────

#[test]
fn test_vm_exec_run_echo() {
    let obj = unwrap_object(vm_run_and_get(r#"var x = exec.run("echo vm_test");"#, "x"));
    match obj.get("success").and_then(|v| v.as_bool()) {
        Some(b) => assert!(b),
        other => panic!("Expected success=true, got {:?}", other),
    }
    match obj.get("stdout").and_then(|v| v.as_str()) {
        Some(s) => assert!(s.contains("vm_test")),
        other => panic!("Expected stdout containing 'vm_test', got {:?}", other),
    }
}

#[test]
fn test_vm_exec_output() {
    let s = unwrap_string(vm_run_and_get(
        r#"var x = exec.output("echo vm_output");"#,
        "x",
    ));
    assert!(s.contains("vm_output"));
}

#[test]
fn test_vm_exec_lines() {
    let tuple = vm_run_and_get(r#"var x = exec.lines("echo hello");"#, "x");
    match tuple.1.as_array() {
        Some(arr) => assert!(!arr.is_empty()),
        other => panic!("Expected array, got {:?}", other),
    }
}
