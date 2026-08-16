use super::{
    add_functions, assert_payload_target, call_payload, call_then_return_chunk, chunk,
    return_int_chunk, symbol,
};
use crate::vm::VM;
use hudhudscript_bytecode::{Bytecode, Instruction};
use std::sync::Arc;

use super::super::merge_module_bytecode;

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
