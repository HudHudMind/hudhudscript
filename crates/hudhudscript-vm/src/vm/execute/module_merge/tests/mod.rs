mod call_indices;
mod pools;
mod serialization;

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
