//! G06F deep-chain acceptance tests: 2500 user call levels must stay on a
//! single native driver without overflowing the OS stack, and the
//! max-call-depth guard must fire cleanly.

use crate::vm::VM;
use hudhudscript_bytecode::{Bytecode, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

const DEEP_SCRIPT: &str = r#"
class Deep {
    fn go(n) {
        if (n == 0) { return "sentinel-g06f" }
        return this.go(n - 1)
    }
}
let r = (new Deep()).go(2500)
"#;

fn compile_source(source: &str) -> Bytecode {
    let ast = parse(source).expect("test source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("test source must compile")
}

fn variable(vm: &VM, name: &str) -> Value16 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name))
}

#[test]
fn deep_agent_action_chain_does_not_overflow_native_stack() {
    let bytecode = compile_source(DEEP_SCRIPT);
    let mut vm = VM::new();
    vm.with_max_call_depth(3000);
    VM::reset_driver_entry_count_for_test();
    vm.execute(&bytecode).expect("deep chain must execute");

    assert_eq!(
        variable(&vm, "r").as_string(),
        Some("sentinel-g06f".to_string())
    );
    assert_eq!(
        VM::driver_entry_count_for_test(),
        1,
        "2500 user call levels must stay on one canonical native driver"
    );
}

#[test]
fn deep_agent_action_chain_honors_call_depth_limit() {
    let bytecode = compile_source(DEEP_SCRIPT);
    let mut vm = VM::new();
    vm.with_max_call_depth(512);
    VM::reset_driver_entry_count_for_test();
    let result = vm.execute(&bytecode);
    let error = result.expect_err("depth limit must fire");
    assert!(
        error.message.contains("Maximum call depth exceeded"),
        "unexpected error: {}",
        error.message
    );
}

#[test]
fn method_spread_preserves_receiver_and_argument_array() {
    let bytecode = compile_source(
        r#"
class Adder {
    fn combine(a, b, c) {
        return this.base + a + b + c
    }
    fn setup(v) {
        this.base = v
        return this
    }
}
let parts = [3, 4]
let r = (new Adder()).setup(10).combine(1, ...parts)
"#,
    );
    let mut vm = VM::new();
    VM::reset_driver_entry_count_for_test();
    vm.execute(&bytecode).expect("spread method must execute");
    assert_eq!(variable(&vm, "r").as_number(), Some(18.0));
}

#[test]
fn method_spread_callback_method_uses_trampoline() {
    let bytecode = compile_source(
        r#"
let values = [1, 2, 3]
let fns = [(item, index) => { return item + index }]
let mapped = values.map(...fns)
"#,
    );
    let mut vm = VM::new();
    VM::reset_driver_entry_count_for_test();
    vm.execute(&bytecode)
        .expect("spread callback method must execute");
    assert_eq!(
        VM::driver_entry_count_for_test(),
        1,
        "spread callback must stay on the canonical native driver"
    );
    let mapped = variable(&vm, "mapped");
    let array = mapped.as_array().expect("mapped must be an array");
    let numbers: Vec<f64> = array
        .iter()
        .map(|item| item.as_number().expect("numbers only"))
        .collect();
    assert_eq!(numbers, vec![1.0, 3.0, 5.0]);
}
