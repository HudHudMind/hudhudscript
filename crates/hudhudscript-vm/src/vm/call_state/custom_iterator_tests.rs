//! G06C-A acceptance tests for custom iterator continuations.

use crate::vm::call_state::{CustomIteratorSequence, VmContinuation};
use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{gc, FunctionChunk, Instruction, SymId, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use rustc_hash::FxHashMap;
use std::sync::Arc;

const RANGE_SCRIPT: &str = r#"
class Range {
    constructor(limit) { this.i = 0; this.limit = limit }
    next() {
        if (this.i >= this.limit) { return null }
        let v = this.i
        this.i = this.i + 1
        return v
    }
}
let out = []
for (let x in new Range(3)) { out.push(x) }
"#;

fn compile_source(source: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = parse(source).expect("test source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("test source must compile")
}

fn run_source(source: &str) -> VM {
    let bytecode = compile_source(source);
    let mut vm = VM::new();
    VM::reset_driver_entry_count_for_test();
    vm.execute(&bytecode).expect("test source must execute");
    vm
}

fn collected(vm: &VM) -> Vec<f64> {
    let value = vm.get_variable_owned("out").expect("out must be published");
    let array = value.as_array().expect("out must be an array");
    array
        .iter()
        .map(|item| item.as_number().expect("out must contain numbers"))
        .collect()
}

#[test]
fn custom_iterator_next_uses_trampoline() {
    let vm = run_source(RANGE_SCRIPT);
    assert_eq!(collected(&vm), vec![0.0, 1.0, 2.0]);
    assert_eq!(
        VM::driver_entry_count_for_test(),
        1,
        "custom iterator next() calls must stay on the canonical native driver"
    );
}

#[test]
fn custom_iterator_stops_on_null_and_preserves_order() {
    let vm = run_source(RANGE_SCRIPT);
    assert_eq!(collected(&vm), vec![0.0, 1.0, 2.0]);
}

#[test]
fn custom_iterator_limit_returns_clean_runtime_error() {
    let source = r#"
class Forever {
    constructor() { this.i = 0 }
    next() {
        this.i = this.i + 1
        return this.i
    }
}
let out = []
for (let x in new Forever()) { out.push(x) }
"#;
    let bytecode = compile_source(source);
    let mut vm = VM::new();
    vm.max_builtin_iter = 4;
    VM::reset_driver_entry_count_for_test();
    let result: CompileResult<()> = vm.execute(&bytecode).map(|_| ());
    let error = result.expect_err("infinite iterator must hit the iteration limit");
    assert!(
        error.message.contains("maximum iteration limit"),
        "unexpected error: {}",
        error.message
    );
}

fn return_chunk() -> Arc<FunctionChunk> {
    Arc::new(FunctionChunk {
        params: vec![],
        instructions: vec![Instruction::Return { src: 0 }],
        constants: vec![],
        captures: vec![],
        capture_sym_ids: vec![],
        capture_slots: vec![],
        is_async: false,
        is_generator: false,
        local_count: 0,
        local_names: vec![],
        capture_cells: vec![],
        max_register: 0,
        sym_to_slot: std::sync::OnceLock::new(),
        param_slots: Box::new([]),
        is_plain_function: true,
        source_positions: vec![None],
    })
}

#[test]
fn custom_iterator_state_survives_gc() {
    let mut vm = VM::new();
    let receiver = Value16::string("gc-iterator-receiver-dynamic-value");
    let element = Value16::string("gc-iterator-element-dynamic-value");
    let state = CustomIteratorSequence {
        receiver,
        elements: vec![element],
        variable_name: "x".to_string(),
        limit: 100,
        dst: 255,
        origin_ip: 0,
        chunk: return_chunk(),
        func_sym: SymId(0),
        captures: FxHashMap::default(),
        class_sym: None,
        write_back: false,
    };
    vm.vm_continuations
        .push(VmContinuation::CustomIteratorSequence(state));

    vm.mark_from_roots();

    assert!(gc::is_marked(receiver), "receiver root was not marked");
    assert!(gc::is_marked(element), "element root was not marked");
}
