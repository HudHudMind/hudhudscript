//! Regression tests for the compile-time inliner (P0 correctness).
//!
//! Validates constant pool remapping, checked register arithmetic,
//! 255 return register parity, and safe rejection when ineligible.
//!
//! These tests use `try_inline_plan` directly (a pure function) so no
//! CompileTarget is needed — constant remap tables are supplied explicitly.

use hudhudscript_bytecode::{FunctionChunk, Instruction, Value16};
use hudhudscript_compiler::optimizer::inline_compile::try_inline_plan;

fn make_chunk(
    params: Vec<&str>,
    instructions: Vec<Instruction>,
) -> FunctionChunk {
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
        param_slots: (0..params.len() as u16).collect::<Vec<_>>().into_boxed_slice(),
        is_plain_function: true,
    }
}

fn make_chunk_with_consts(
    params: Vec<&str>,
    instructions: Vec<Instruction>,
    constants: Vec<Value16>,
) -> FunctionChunk {
    let mut c = make_chunk(params, instructions);
    c.constants = constants;
    c
}

// =====================================================================
// 1. Constant pool collision — same index, different values
// =====================================================================

#[test]
fn const_remap_same_index_different_string() {
    // Callee has const_idx=0 → "hello". Caller already has const_idx=0 → "world"
    // (simulated by remap table mapping callee 0 → caller 1).
    let chunk = make_chunk_with_consts(
        vec!["x"],
        vec![
            Instruction::LoadConst { dst: 1, const_idx: 0 },
            Instruction::Return { src: 1 },
        ],
        vec![Value16::string("hello")],
    );
    let const_remap = vec![1u16]; // callee const_idx 0 → caller const_idx 1
    let plan = try_inline_plan(&chunk, 10, 1, 255, &const_remap, &[], &[]);
    assert!(plan.is_some(), "should inline with const remap");
    let instrs = plan.unwrap();
    let load = instrs.iter().find(|ci| matches!(ci, Instruction::LoadConst { .. }));
    assert!(load.is_some());
    if let Instruction::LoadConst { const_idx, .. } = load.unwrap() {
        assert_eq!(*const_idx, 1, "const_idx must be remapped to 1, not 0");
    }
    // Ensure no instruction retains the callee's original const_idx
    let has_0 = instrs.iter().any(|ci| matches!(ci,
        Instruction::LoadConst { const_idx: 0, .. }
    ));
    assert!(!has_0, "const_idx=0 must not appear (collision avoided)");
}

#[test]
fn const_remap_empty_constants_still_works() {
    // Callee with no constants — remap table is empty
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
        Instruction::Return { src: 1 },
    ]);
    let plan = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
    assert!(plan.is_some());
}

// =====================================================================
// 2. LoadIntConst / LoadNumConst use global pools — no remap needed
// =====================================================================

#[test]
fn load_int_const_preserved() {
    let chunk = make_chunk(vec![], vec![
        Instruction::LoadIntConst { dst: 0, const_idx: 7 },
        Instruction::Return { src: 0 },
    ]);
    let plan = try_inline_plan(&chunk, 10, 0, 255, &[], &[], &[]);
    assert!(plan.is_some());
    let instrs = plan.unwrap();
    let has_load = instrs.iter().any(|ci| matches!(ci,
        Instruction::LoadIntConst { const_idx: 7, dst: 10 } // dst remapped
    ));
    assert!(has_load, "LoadIntConst const_idx=7 must be preserved");
}

#[test]
fn load_num_const_preserved() {
    let chunk = make_chunk(vec![], vec![
        Instruction::LoadNumConst { dst: 0, const_idx: 3 },
        Instruction::Return { src: 0 },
    ]);
    let plan = try_inline_plan(&chunk, 5, 0, 255, &[], &[], &[]);
    assert!(plan.is_some());
    let instrs = plan.unwrap();
    let has_load = instrs.iter().any(|ci| matches!(ci,
        Instruction::LoadNumConst { const_idx: 3, .. }
    ));
    assert!(has_load, "LoadNumConst const_idx=3 must be preserved");
}

// =====================================================================
// 3. Register overflow detection — wrapping is NOT allowed
// =====================================================================

#[test]
fn register_overflow_base() {
    // first_arg=250, arg_count=10 → base exceeds u8
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
        Instruction::Return { src: 1 },
    ]);
    let plan = try_inline_plan(&chunk, 250, 10, 255, &[], &[], &[]);
    assert!(plan.is_none(), "base overflow must reject inlining");
}

#[test]
fn register_overflow_param_map() {
    // first_arg=254, param reg 0 → 254 (OK)
    // callee reg 1 (non-param) → base(255) + 0 = 255 (OK — 255 is special)
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::Move { dst: 1, src: 0 },
        Instruction::Return { src: 1 },
    ]);
    let plan = try_inline_plan(&chunk, 254, 1, 10, &[], &[], &[]);
    assert!(plan.is_some(), "255 as non-param reg is valid");
}

#[test]
fn register_overflow_beyond_254() {
    // first_arg=254, arg_count=2 → base overflows u8
    let chunk = make_chunk(vec!["a", "b"], vec![
        Instruction::IntAdd { dst: 2, src1: 0, src2: 1 },
        Instruction::Return { src: 2 },
    ]);
    let plan = try_inline_plan(&chunk, 254, 2, 255, &[], &[], &[]);
    assert!(plan.is_none(), "overflow must reject");
}

#[test]
fn register_overflow_callee_high_reg() {
    // callee has reg 200, first_arg=0, argc=1 → base=1, offset=199 → 200 < 255 OK
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::Move { dst: 200, src: 0 },
        Instruction::Return { src: 200 },
    ]);
    let plan = try_inline_plan(&chunk, 0, 1, 255, &[], &[], &[]);
    assert!(plan.is_some(), "high callee reg within range should work");
}

#[test]
fn register_overflow_callee_reg_too_high() {
    // callee has reg 200, first_arg=100, argc=1 → base=101, offset=199 → 300 overflows
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::Move { dst: 200, src: 0 },
        Instruction::Return { src: 200 },
    ]);
    let plan = try_inline_plan(&chunk, 100, 1, 255, &[], &[], &[]);
    assert!(plan.is_none(), "high callee reg overflow must reject");
}

// =====================================================================
// 4. dst=255 vs normal dst parity
// =====================================================================

#[test]
fn dst_255_return_becomes_move_to_255() {
    // compile_complex.rs always passes dst=255
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::Return { src: 0 },
    ]);
    // first_arg=10, argc=1 → ret_src map: 0→10. dst=255. 10≠255 → Move{255,10}
    let plan = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
    assert!(plan.is_some());
    let instrs = plan.unwrap();
    assert_eq!(instrs.len(), 1);
    assert!(matches!(instrs[0], Instruction::Move { dst: 255, src: 10 }));
}

#[test]
fn normal_dst_return_becomes_move_to_dst() {
    // compile_reg.rs uses actual dst
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::Return { src: 0 },
    ]);
    let plan = try_inline_plan(&chunk, 10, 1, 5, &[], &[], &[]);
    assert!(plan.is_some());
    let instrs = plan.unwrap();
    assert_eq!(instrs.len(), 1);
    assert!(matches!(instrs[0], Instruction::Move { dst: 5, src: 10 }));
}

#[test]
fn return_self_move_elided() {
    // When dst == mapped_src, no Move emitted
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::Return { src: 0 },
    ]);
    let plan = try_inline_plan(&chunk, 5, 1, 5, &[], &[], &[]);
    assert!(plan.is_some());
    assert!(plan.unwrap().is_empty(), "Move{{dst=5,src=5}} elided");
}

// =====================================================================
// 5. Inlining rejection → normal Call semantics preserved
// =====================================================================

#[test]
fn body_too_large_rejects() {
    let mut body: Vec<Instruction> = (0..16)
        .map(|i| Instruction::Move { dst: i as u8, src: (i + 1) as u8 })
        .collect();
    body[15] = Instruction::Return { src: 15 };
    let chunk = make_chunk(vec![], body);
    assert!(try_inline_plan(&chunk, 10, 0, 255, &[], &[], &[]).is_none());
}

#[test]
fn fused_return_rejects() {
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
        Instruction::IntAddReturn { src1: 0, src2: 1 },
    ]);
    assert!(try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]).is_none());
}

#[test]
fn return_const_rejects() {
    let chunk = make_chunk_with_consts(
        vec![],
        vec![Instruction::ReturnConst { const_idx: 0 }],
        vec![Value16::int(42)],
    );
    assert!(try_inline_plan(&chunk, 10, 0, 255, &[0], &[], &[]).is_none());
}

#[test]
fn loop_body_rejects() {
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::LoopBegin(0),
        Instruction::Return { src: 0 },
    ]);
    assert!(try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]).is_none());
}

#[test]
fn empty_body_rejects() {
    let chunk = make_chunk(vec![], vec![]);
    assert!(try_inline_plan(&chunk, 10, 0, 255, &[], &[], &[]).is_none());
}

// =====================================================================
// 6. Source position & instruction count parity
// =====================================================================

#[test]
fn instruction_count_parity() {
    // 3 instructions in callee → 3 inlined (Return becomes Move)
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
        Instruction::IntMulI { dst: 1, src: 1, imm: 2 },
        Instruction::Return { src: 1 },
    ]);
    let plan = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
    assert!(plan.is_some());
    let instrs = plan.unwrap();
    // 2 arithmetic + 1 Move (Return→Move, dst≠src) = 3
    assert_eq!(instrs.len(), 3, "instruction count must stay consistent");
}

#[test]
fn multi_param_callee_remap() {
    // 3 parameters + 1 temp
    let chunk = make_chunk(vec!["a", "b", "c"], vec![
        Instruction::IntAdd { dst: 3, src1: 0, src2: 1 },
        Instruction::IntAdd { dst: 3, src1: 3, src2: 2 },
        Instruction::Return { src: 3 },
    ]);
    // first_arg=20, argc=3 → params: 0→20, 1→21, 2→22, base=23, reg3→23+0=23
    let plan = try_inline_plan(&chunk, 20, 3, 255, &[], &[], &[]);
    assert!(plan.is_some());
    let instrs = plan.unwrap();
    assert_eq!(instrs.len(), 3);
    // First IntAdd: dst(3)→23, src1(0)→20, src2(1)→21
    assert!(matches!(instrs[0], Instruction::IntAdd { dst: 23, src1: 20, src2: 21 }));
    // Second IntAdd: dst(3)→23, src1(3)→23, src2(2)→22
    assert!(matches!(instrs[1], Instruction::IntAdd { dst: 23, src1: 23, src2: 22 }));
}

// =====================================================================
// 7. Unsupported instruction rejection (mutation-free)
// =====================================================================

#[test]
fn unsupported_instr_rejects_no_partial_mutation() {
    // The inliner must reject atomically — no partial instruction list emitted.
    // This tests that an unsupported instruction mid-body causes clean rejection.
    let chunk = make_chunk(vec!["x"], vec![
        Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
        Instruction::Throw { src: 0 },       // unsupported – side effect
        Instruction::Return { src: 1 },
    ]);
    // Throw is in the side-effect list → rejected before remap
    let plan = try_inline_plan(&chunk, 10, 1, 255, &[], &[], &[]);
    assert!(plan.is_none(), "unsupported mid-body must reject cleanly");
}
