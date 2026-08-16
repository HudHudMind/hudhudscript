//! G06C-B acceptance tests for promise callback continuations.

use crate::vm::VM;
use hudhudscript_bytecode::{PromiseState16, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn run_source(source: &str) -> VM {
    let ast = parse(source).expect("test source must parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("test source must compile");
    let mut vm = VM::new();
    VM::reset_driver_entry_count_for_test();
    vm.execute(&bytecode).expect("test source must execute");
    assert_eq!(
        VM::driver_entry_count_for_test(),
        1,
        "promise callbacks must stay on the canonical native driver"
    );
    vm
}

fn resolved_value(vm: &VM, name: &str) -> Value16 {
    let value = vm
        .get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name));
    match value.as_promise_state() {
        Some(PromiseState16::Resolved(inner)) => **inner,
        other => panic!("{} must be a resolved promise, got {:?}", name, other),
    }
}

#[test]
fn promise_then_callback_uses_trampoline() {
    let vm = run_source(
        r#"
let p = Promise.resolve(5)
let r = p.then((v) => { return v + 1 })
"#,
    );
    assert_eq!(resolved_value(&vm, "r").as_number(), Some(6.0));
}

#[test]
fn promise_catch_callback_uses_trampoline() {
    let vm = run_source(
        r#"
let p = Promise.reject("boom")
let r = p.catch((message) => { return message + "-handled" })
"#,
    );
    assert_eq!(
        resolved_value(&vm, "r").as_string(),
        Some("boom-handled".to_string())
    );
}
