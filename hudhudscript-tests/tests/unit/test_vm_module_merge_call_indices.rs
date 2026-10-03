//! Module merge call-index rebasing tests: merged call payloads must resolve
//! to the canonical function index in the parent bytecode.

use hudhudscript_vm::vm::execute::module_merge::merge_module_bytecode;
use hudhudscript_vm::vm::VM;
use hudhudscript_bytecode::{Bytecode, CallPayload, FunctionChunk, Instruction, SymId};
use std::sync::Arc;

fn chunk(instructions: Vec<Instruction>) -> Arc<FunctionChunk> {
    Arc::new(FunctionChunk {
        params: vec![],
        source_positions: vec![None; instructions.len()],
        instructions,
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
    })
}

fn symbol(name: &str) -> SymId {
    SymId(hudhudscript_bytecode::interner::intern(name).0)
}

fn call_payload(name: &str, function_idx: u32) -> CallPayload {
    CallPayload {
        sym: symbol(name),
        arg_count: 0,
        function_idx,
        builtin_method_idx: u32::MAX,
    }
}

fn return_int_chunk(const_idx: u16) -> Arc<FunctionChunk> {
    chunk(vec![
        Instruction::LoadIntConst { dst: 0, const_idx },
        Instruction::Return { src: 0 },
    ])
}

fn call_then_return_chunk(payload_idx: u16) -> Arc<FunctionChunk> {
    chunk(vec![
        Instruction::Call {
            dst: 0,
            payload_idx,
            first_arg: 1,
            arg_count: 0,
        },
        Instruction::Return { src: 0 },
    ])
}

fn add_functions(bytecode: &Bytecode, prefix: &str, count: usize) {
    for index in 0..count {
        bytecode.add_function(
            format!("{}_{}", prefix, index),
            chunk(vec![Instruction::Return { src: 0 }]),
        );
    }
}

fn assert_payload_target(bytecode: &Bytecode, payload_index: usize, expected_name: &str) {
    let payload = bytecode
        .call_payloads
        .get(payload_index)
        .expect("merged call payload must exist");
    let expected_index = bytecode
        .get_function_idx(expected_name)
        .expect("expected target function must exist");
    assert_eq!(payload.function_idx, expected_index);
    assert_eq!(
        bytecode
            .function_name_at(payload.function_idx)
            .expect("resolved function index must be canonical"),
        expected_name
    );
    assert_eq!(
        hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(
            payload.sym.0
        ),),
        expected_name
    );
}

#[test]
fn module_action_rebases_direct_function_index() {
    let mut target = Bytecode::default();
    add_functions(&target, "parent", 9);
    target.call_payloads.push(call_payload("parent_8", 8));

    let mut source = Bytecode::default();
    add_functions(&source, "module", 8);
    source.add_function(
        "invoke_agent".to_string(),
        chunk(vec![Instruction::Return { src: 0 }]),
    );
    source.call_payloads.push(call_payload("invoke_agent", 8));
    source.action_registry.borrow_mut().insert(
        "IterationAgentProcess.execute".to_string(),
        call_then_return_chunk(0),
    );

    merge_module_bytecode(&source, &target).expect("module merge must succeed");

    let merged_action = target
        .action_registry
        .borrow()
        .get("IterationAgentProcess.execute")
        .cloned()
        .expect("module action must be copied");
    match &merged_action.instructions[0] {
        Instruction::Call { payload_idx, .. } => assert_eq!(*payload_idx, 1),
        instruction => panic!("expected merged Call, got {instruction:?}"),
    }
    assert_payload_target(&target, 1, "invoke_agent");
    assert_ne!(target.call_payloads[1].function_idx, 8);
}

#[test]
fn merged_call_payload_symbol_matches_target_index() {
    let target = Bytecode::default();
    target.add_function(
        "existing".to_string(),
        chunk(vec![Instruction::Return { src: 0 }]),
    );

    let mut source = Bytecode::default();
    source.add_function(
        "later".to_string(),
        chunk(vec![Instruction::Return { src: 0 }]),
    );
    source.add_function(
        "invoke_agent".to_string(),
        chunk(vec![Instruction::Return { src: 0 }]),
    );
    source.call_payloads.push(call_payload("invoke_agent", 1));

    merge_module_bytecode(&source, &target).expect("module merge must succeed");

    assert_payload_target(&target, 0, "invoke_agent");
    assert_eq!(target.call_payloads[0].function_idx, 2);
}

#[test]
fn module_calls_to_earlier_and_later_functions() {
    let target = Bytecode::default();
    add_functions(&target, "parent", 3);

    let mut source = Bytecode::default();
    add_functions(&source, "module", 10);
    source.call_payloads.push(call_payload("module_0", 0));
    source.call_payloads.push(call_payload("module_9", 9));

    merge_module_bytecode(&source, &target).expect("module merge must succeed");

    assert_payload_target(&target, 0, "module_0");
    assert_payload_target(&target, 1, "module_9");
    assert_eq!(target.call_payloads[0].function_idx, 3);
    assert_eq!(target.call_payloads[1].function_idx, 12);
}

#[test]
fn module_action_does_not_reenter_parent_loop() {
    let mut target = Bytecode::default();
    target.int_constants.push(-8);
    for index in 0..8 {
        target.add_function(
            format!("parent_{index}"),
            chunk(vec![Instruction::Return { src: 0 }]),
        );
    }
    target.add_function("parent_loop".to_string(), return_int_chunk(0));

    let mut source = Bytecode::default();
    source.int_constants.push(42);
    add_functions(&source, "module", 8);
    source.add_function("invoke_agent".to_string(), return_int_chunk(0));
    source.call_payloads.push(call_payload("invoke_agent", 8));
    source.action_registry.borrow_mut().insert(
        "IterationAgentProcess.execute".to_string(),
        call_then_return_chunk(0),
    );

    merge_module_bytecode(&source, &target).expect("module merge must succeed");
    assert_payload_target(&target, 0, "invoke_agent");
    assert_eq!(target.get_function_idx("parent_loop"), Some(8));
    assert_eq!(target.get_function_idx("invoke_agent"), Some(17));

    let action = Arc::clone(
        target
            .action_registry
            .borrow()
            .get("IterationAgentProcess.execute")
            .expect("merged action must exist"),
    );
    let mut vm = VM::new();
    let result = vm
        .call_chunk(&action, &[], &[], &target, symbol("execute"))
        .expect("merged action execution must succeed");

    assert_eq!(result.as_int(), Some(42));
}
