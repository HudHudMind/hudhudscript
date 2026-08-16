use super::{assert_payload_target, call_payload, call_then_return_chunk, chunk};
use hudhudscript_bytecode::{Bytecode, Instruction};

use super::super::merge_module_bytecode;

#[test]
fn serialized_module_call_index_is_rebased() {
    let mut target = Bytecode::default();
    for index in 0..9 {
        target.add_function(
            format!("parent_{index}"),
            chunk(vec![Instruction::Return { src: index as u8 }]),
        );
    }
    target.call_payloads.push(call_payload("parent_8", 8));

    let mut source = Bytecode::default();
    for index in 0..8 {
        source.add_function(
            format!("module_{index}"),
            chunk(vec![Instruction::Return { src: index as u8 }]),
        );
    }
    source.add_function(
        "invoke_agent".to_string(),
        chunk(vec![Instruction::Return { src: 8 }]),
    );
    source.call_payloads.push(call_payload("invoke_agent", 8));
    source.action_registry.borrow_mut().insert(
        "IterationAgentProcess.execute".to_string(),
        call_then_return_chunk(0),
    );

    let bytes = source
        .to_bytes()
        .expect("module serialization must succeed");
    let restored = Bytecode::from_bytes(&bytes).expect("module deserialization must succeed");
    assert_eq!(restored.call_payloads[0].function_idx, 8);
    assert_eq!(
        restored
            .function_name_at(8)
            .expect("source index must retain its name"),
        "invoke_agent"
    );

    merge_module_bytecode(&restored, &target).expect("serialized module merge must succeed");

    assert_payload_target(&target, 1, "invoke_agent");
    assert_eq!(target.call_payloads[1].function_idx, 17);
    let action = target
        .action_registry
        .borrow()
        .get("IterationAgentProcess.execute")
        .cloned()
        .expect("serialized action must be merged");
    match &action.instructions[0] {
        Instruction::Call { payload_idx, .. } => assert_eq!(*payload_idx, 1),
        instruction => panic!("expected rebased Call, got {instruction:?}"),
    }
}

#[test]
fn bytecode_round_trip_preserves_function_index_name_pairs() {
    let original = Bytecode::default();
    let names = ["zeta", "alpha", "middle", "omega", "beta"];
    for (index, name) in names.iter().enumerate() {
        original.add_function(
            (*name).to_string(),
            chunk(vec![Instruction::Return { src: index as u8 }]),
        );
    }

    let bytes = original
        .to_bytes()
        .expect("bytecode serialization must succeed");
    let restored = Bytecode::from_bytes(&bytes).expect("bytecode deserialization must succeed");

    assert_eq!(restored.function_count(), names.len());
    for (index, expected_name) in names.iter().enumerate() {
        assert_eq!(
            restored
                .function_name_at(index as u32)
                .expect("restored function index must be valid"),
            *expected_name
        );
        assert_eq!(restored.get_function_idx(expected_name), Some(index as u32));
        let function = restored
            .get_function_by_index(index as u32)
            .expect("restored function chunk must exist");
        match function.instructions.as_slice() {
            [Instruction::Return { src }] => assert_eq!(*src, index as u8),
            instructions => panic!("unexpected restored instructions: {instructions:?}"),
        }
    }
}
