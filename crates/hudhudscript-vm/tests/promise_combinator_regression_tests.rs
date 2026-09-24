//! Regression tests for VM Promise combinators over async-function results.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn execute(source: &str) -> Result<VM, String> {
    let ast = parse(source).map_err(|error| format!("parse: {error}"))?;
    let bytecode = Compiler::new()
        .compile(&ast)
        .map_err(|error| format!("compile: {error}"))?;
    let mut vm = VM::new();
    vm.execute(&bytecode)
        .map_err(|error| format!("execute: {error}"))?;
    Ok(vm)
}

fn string_variable(vm: &VM, name: &str) -> String {
    vm.get_variable(name)
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| panic!("{name} must be a published string"))
}

#[test]
fn promise_all_resolves_async_functions_in_input_order() {
    let vm = execute(
        r#"
async function delayed(value, duration) {
    sleep(duration)
    return value
}
let slow = delayed("slow", 80)
let fast = delayed("fast", 10)
let values = await Promise.all([slow, fast])
let first = values[0]
let second = values[1]
"#,
    )
    .expect("Promise.all must resolve detached async-function promises");

    assert_eq!(string_variable(&vm, "first"), "slow");
    assert_eq!(string_variable(&vm, "second"), "fast");
}

#[test]
fn promise_all_combines_settled_and_detached_promises() {
    let vm = execute(
        r#"
async function later() {
    sleep(10)
    return "detached"
}
let settled = Promise.resolve("settled")
let values = await Promise.all([settled, later()])
let first = values[0]
let second = values[1]
"#,
    )
    .expect("Promise.all must support mixed resolver transports");

    assert_eq!(string_variable(&vm, "first"), "settled");
    assert_eq!(string_variable(&vm, "second"), "detached");
}

#[test]
fn promise_race_returns_first_detached_async_function() {
    let vm = execute(
        r#"
async function delayed(value, duration) {
    sleep(duration)
    return value
}
let winner = await Promise.race([
    delayed("slow", 120),
    delayed("fast", 10)
])
"#,
    )
    .expect("Promise.race must resolve detached async-function promises");

    assert_eq!(string_variable(&vm, "winner"), "fast");
}

#[test]
fn promise_all_propagates_detached_rejection() {
    let result = execute(
        r#"
async function failSoon() {
    sleep(10)
    throw "detached boom"
}
async function succeedLater() {
    sleep(80)
    return "late"
}
let values = await Promise.all([succeedLater(), failSoon()])
"#,
    );
    let error = match result {
        Ok(_) => panic!("Promise.all must reject when a detached async function rejects"),
        Err(error) => error,
    };

    assert!(
        error.contains("detached boom"),
        "the original async rejection must be preserved, got: {error}"
    );
}
