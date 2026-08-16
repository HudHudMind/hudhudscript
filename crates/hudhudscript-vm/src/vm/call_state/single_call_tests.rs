//! G06A acceptance tests: nested user-code calls through agent actions,
//! instance methods, property functions and constructors must all run on
//! the single canonical native driver (`run_frame_loop` entered once).

use crate::vm::VM;
use hudhudscript_bytecode::Bytecode;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn compile_source(source: &str) -> Bytecode {
    let ast = parse(source).expect("test source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("test source must compile")
}

fn run_and_get_int(source: &str, variable: &str) -> i64 {
    let bytecode = compile_source(source);
    let mut vm = VM::new();
    VM::reset_driver_entry_count_for_test();
    vm.execute(&bytecode).expect("test source must execute");
    let value = vm
        .get_variable(variable)
        .unwrap_or_else(|| panic!("{} must be published", variable));
    let result = value
        .as_int()
        .unwrap_or_else(|| panic!("{} must be an int", variable));
    assert_eq!(
        VM::driver_entry_count_for_test(),
        1,
        "nested user-code calls must not open a second native driver"
    );
    result
}

#[test]
fn nested_agent_actions_use_single_native_driver() {
    let result = run_and_get_int(
        r#"
agent Inner {
    action ping(x) {
        return x + 1
    }
}

agent Outer {
    action run(x) {
        return Inner.ping(x) + 1
    }
}

let result = Outer.run(40)
"#,
        "result",
    );
    assert_eq!(result, 42);
}

#[test]
fn nested_instance_methods_use_single_native_driver() {
    let result = run_and_get_int(
        r#"
class Counter {
    constructor(start) {
        this.value = start
    }
    fn inc(x) {
        return this.add(x) + 1
    }
    fn add(x) {
        return this.value + x
    }
}

let c = new Counter(10)
let result = c.inc(1)
"#,
        "result",
    );
    assert_eq!(result, 12);
}

#[test]
fn nested_property_functions_use_single_native_driver() {
    let result = run_and_get_int(
        r#"
let bonus = 100
let o = {
    base: 10,
    inner: (y) => { return this.base + y + bonus },
    outer: (y) => { return this.inner(y) + 1 }
}
let result = o.outer(1)
"#,
        "result",
    );
    assert_eq!(result, 112);
}

#[test]
fn constructor_call_uses_single_native_driver() {
    let result = run_and_get_int(
        r#"
class Point {
    constructor(x) {
        this.x = this.double(x)
    }
    fn double(v) {
        return v * 2
    }
}

let p = new Point(21)
let result = p.x
"#,
        "result",
    );
    assert_eq!(result, 42);
}
