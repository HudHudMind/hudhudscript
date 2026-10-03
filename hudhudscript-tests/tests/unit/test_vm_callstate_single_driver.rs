//! G06A acceptance tests: nested user-code calls through agent actions,
//! instance methods, property functions and constructors must all run on
//! the single canonical native driver (`run_frame_loop` entered once).
//! Plus deferred-call frame scheduling invariants from the G06A shell.

use hudhudscript_vm::vm::call_state::{ReturnSink, VmCallRequest};
use hudhudscript_vm::vm::VM;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, Instruction, SymId, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::rustc_hash::FxHashMap;
use std::sync::{Arc, OnceLock};

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

fn returning_chunk(value: Value16) -> Arc<FunctionChunk> {
    Arc::new(FunctionChunk {
        params: vec![],
        instructions: vec![
            Instruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            Instruction::Return { src: 0 },
        ],
        constants: vec![value],
        captures: vec![],
        capture_sym_ids: vec![],
        capture_slots: vec![],
        is_async: false,
        is_generator: false,
        local_count: 0,
        local_names: vec![],
        capture_cells: vec![],
        max_register: 0,
        sym_to_slot: OnceLock::new(),
        param_slots: Box::new([]),
        is_plain_function: true,
        source_positions: vec![None, None],
    })
}

fn request(value: Value16, dst: u8) -> Box<VmCallRequest> {
    Box::new(VmCallRequest {
        chunk: returning_chunk(value),
        func_sym: SymId(hudhudscript_bytecode::interner::intern("deferred_test").0),
        args: vec![],
        captures: FxHashMap::default(),
        dst,
        origin_ip: 0,
        receiver_context: None,
        return_sink: ReturnSink::Register(dst),
        swallow_error: false,
    })
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

#[test]
fn deferred_call_pushes_frame_without_nested_driver() {
    let mut vm = VM::new();
    let bytecode = Bytecode::new();
    VM::reset_driver_entry_count_for_test();
    vm.schedule_vm_call(request(Value16::int(7), 4)).unwrap();

    let returned = vm.run_frame_loop(&bytecode, &[], 0).unwrap();

    assert!(returned);
    assert_eq!(VM::driver_entry_count_for_test(), 1);
    assert!(vm.frame_stack.is_empty());
    assert!(vm.pending_vm_call.is_none());
}

#[test]
fn deferred_call_result_reaches_destination_register() {
    let mut vm = VM::new();
    let bytecode = Bytecode::new();
    vm.schedule_vm_call(request(Value16::int(42), 17)).unwrap();

    vm.run_frame_loop(&bytecode, &[], 0).unwrap();

    assert_eq!(vm.registers[17].as_int(), Some(42));
}

#[test]
fn deferred_call_depth_limit_returns_runtime_error() {
    let mut vm = VM::new();
    let bytecode = Bytecode::new();
    vm.with_max_call_depth(0);
    vm.schedule_vm_call(request(Value16::int(1), 0)).unwrap();

    let error = vm.run_frame_loop(&bytecode, &[], 0).unwrap_err();

    assert!(error.to_string().contains("Maximum call depth exceeded"));
    assert!(vm.frame_stack.is_empty());
}
