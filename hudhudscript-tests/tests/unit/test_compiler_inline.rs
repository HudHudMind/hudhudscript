//! Optimizer inline-compile tests — `try_inline_plan` eligibility checks.

use hudhudscript_compiler::optimizer::inline_compile::try_inline_plan;
use hudhudscript_bytecode::{FunctionChunk, Instruction};

// ── helpers ───────────────────────────────────────────────────────

fn make_chunk(_name: &str, params: Vec<&str>, instructions: Vec<Instruction>) -> FunctionChunk {
    FunctionChunk {
        params: params.iter().map(|s| s.to_string()).collect(),
        instructions,
        constants: vec![],
        captures: vec![],
        capture_sym_ids: vec![],
        capture_slots: vec![],
        is_async: false,
        is_generator: false,
        local_count: 2,
        local_names: params.iter().map(|s| s.to_string()).collect(),
        capture_cells: vec![],
        max_register: 2,
        sym_to_slot: std::sync::OnceLock::new(),
        source_positions: vec![],
        param_slots: (0..params.len() as u16)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        is_plain_function: true,
    }
}

// ── eligibility ───────────────────────────────────────────────────

#[test]
fn add1_is_inlinable() {
    let chunk = make_chunk(
        "add1",
        vec!["x"],
        vec![
            Instruction::IntAddI {
                dst: 1,
                src: 0,
                imm: 1,
            },
            Instruction::Return { src: 1 },
        ],
    );
    let result = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
    assert!(result.is_some(), "add1(x)=x+1 should be inlinable");
}

#[test]
fn recursive_not_inlinable() {
    let chunk = make_chunk(
        "recurse",
        vec!["x"],
        vec![
            Instruction::Call {
                dst: 1,
                payload_idx: 0,
                first_arg: 0,
                arg_count: 1,
            },
            Instruction::Return { src: 1 },
        ],
    );
    let result = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
    assert!(result.is_none(), "recursive function must NOT be inlinable");
}

#[test]
fn side_effect_not_inlinable() {
    let chunk = make_chunk(
        "s",
        vec!["x"],
        vec![
            Instruction::StoreGlobal { src: 0, sym: 0 },
            Instruction::Return { src: 0 },
        ],
    );
    let result = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
    assert!(result.is_none(), "side-effect must NOT be inlinable");
}
