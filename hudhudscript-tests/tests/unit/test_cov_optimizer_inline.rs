//! Coverage tests for `optimizer/inline.rs` — `inline_small_functions` (P7).
//!
//! The pass is DEFERRED: `entry.rs` keeps its pipeline call commented out, so
//! the honest test surface is direct invocation of the public function (same
//! pattern `test_compiler_inline.rs` uses for `try_inline_plan`).
//!
//! Expected values are hand-traced from the source invariants:
//!   - remap_reg(r): r < arg_count -> first_arg + r; r == 255 -> 255;
//!     otherwise base + (r - arg_count), base = first_arg + arg_count.
//!   - Callee `Return { src }` becomes `Move { dst, src: remap(src) }` when
//!     `dst != ret_src` (pre-remap comparison), else it is deleted.
//!   - Constants, immediates and jump offsets are never register-remapped.

use std::collections::HashMap;
use std::sync::Arc;

use hudhudscript_bytecode::{CallPayload, FunctionChunk, Instruction, SymId};
use hudhudscript_compiler::optimizer::inline_small_functions;

// ── helpers ───────────────────────────────────────────────────────

/// Minimal `FunctionChunk` — only `instructions` matters to the inliner.
fn make_chunk(params: &[&str], instructions: Vec<Instruction>) -> FunctionChunk {
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

/// One-entry symbol table: function name -> chunk.
fn one_func(name: &str, chunk: FunctionChunk) -> HashMap<String, Arc<FunctionChunk>> {
    let mut m = HashMap::new();
    m.insert(name.to_string(), Arc::new(chunk));
    m
}

/// Unresolved direct-call payload (sentinel indices, pre-link shape).
fn direct_payload(name: &str, arg_count: u8) -> CallPayload {
    CallPayload {
        sym: SymId::from(name),
        arg_count,
        function_idx: u32::MAX,
        builtin_method_idx: u32::MAX,
    }
}

fn count_calls(instrs: &[Instruction]) -> usize {
    instrs.iter().filter(|i| matches!(i, Instruction::Call { .. })).count()
}

/// `add1(x) = x + 1` — the canonical inlinable body (2 instructions, pure).
fn add1_body() -> Vec<Instruction> {
    vec![
        Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
        Instruction::Return { src: 1 },
    ]
}

// ── basic inlining ────────────────────────────────────────────────

#[test]
fn single_small_call_is_inlined() {
    // Caller: r0 = c0; r2 = c1 (arg window); r3 = add1(r2).
    let mut instructions = vec![
        Instruction::LoadIntConst { dst: 0, const_idx: 0 },
        Instruction::LoadIntConst { dst: 2, const_idx: 1 },
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 },
    ];
    let funcs = one_func("cov_add1", make_chunk(&["x"], add1_body()));
    let payloads = vec![direct_payload("cov_add1", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    // The single Call is replaced by the 2-instruction remapped body.
    assert_eq!(instructions.len(), 4, "2-instr callee replaces 1 Call");
    assert_eq!(count_calls(&instructions), 0, "Call must be gone");
    // base = first_arg + arg_count = 3: param r0 -> r2, local r1 -> r3.
    assert!(matches!(&instructions[2], Instruction::IntAddI { dst: 3, src: 2, imm: 1 }));
    // Return { src: 1 } -> Move { dst: 3, src: remap(1) = 3 }; dst != ret_src
    // is compared PRE-remap (3 != 1), hence the Move form.
    assert!(matches!(&instructions[3], Instruction::Move { dst: 3, src: 3 }));
    // Instructions before the call site are untouched.
    assert!(matches!(&instructions[0], Instruction::LoadIntConst { dst: 0, const_idx: 0 }));
}

#[test]
fn two_params_map_onto_arg_window() {
    // add2(a, b) = a + b: local dst r2, params in r0/r1.
    let body = vec![
        Instruction::IntAdd { dst: 2, src1: 0, src2: 1 },
        Instruction::Return { src: 2 },
    ];
    let mut instructions = vec![
        Instruction::LoadIntConst { dst: 0, const_idx: 0 },
        Instruction::LoadIntConst { dst: 3, const_idx: 1 },
        Instruction::LoadIntConst { dst: 4, const_idx: 2 },
        Instruction::Call { dst: 5, payload_idx: 0, first_arg: 3, arg_count: 2 },
    ];
    let funcs = one_func("cov_add2", make_chunk(&["a", "b"], body));
    let payloads = vec![direct_payload("cov_add2", 2)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 5, "4-instr caller - 1 Call + 2 body");
    assert_eq!(count_calls(&instructions), 0);
    // base = 3 + 2 = 5: r0 -> r3, r1 -> r4, local r2 -> r5.
    assert!(matches!(&instructions[3], Instruction::IntAdd { dst: 5, src1: 3, src2: 4 }));
    assert!(matches!(&instructions[4], Instruction::Move { dst: 5, src: 5 }));
}

#[test]
fn both_call_sites_inlined_independently() {
    let mut instructions = vec![
        Instruction::LoadIntConst { dst: 0, const_idx: 0 },
        Instruction::LoadIntConst { dst: 2, const_idx: 1 },
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 },
        Instruction::LoadIntConst { dst: 5, const_idx: 2 },
        Instruction::Call { dst: 6, payload_idx: 0, first_arg: 5, arg_count: 1 },
    ];
    let funcs = one_func("cov_add1m", make_chunk(&["x"], add1_body()));
    let payloads = vec![direct_payload("cov_add1m", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 7, "5 - 2 Calls + 2x2 body instrs");
    assert_eq!(count_calls(&instructions), 0);
    // First site: base = 3, param r0 -> r2, local r1 -> r3.
    assert!(matches!(&instructions[2], Instruction::IntAddI { dst: 3, src: 2, imm: 1 }));
    assert!(matches!(&instructions[3], Instruction::Move { dst: 3, src: 3 }));
    // Second site: base = 6, param r0 -> r5, local r1 -> r6.
    assert!(matches!(&instructions[5], Instruction::IntAddI { dst: 6, src: 5, imm: 1 }));
    assert!(matches!(&instructions[6], Instruction::Move { dst: 6, src: 6 }));
}

#[test]
fn zero_arg_call_moves_result_to_dst() {
    // k() = 7 — single local r0, no parameters.
    let body = vec![
        Instruction::LoadIntConst { dst: 0, const_idx: 0 },
        Instruction::Return { src: 0 },
    ];
    let mut instructions = vec![Instruction::Call {
        dst: 2, payload_idx: 0, first_arg: 0, arg_count: 0,
    }];
    let funcs = one_func("cov_k", make_chunk(&[], body));
    let payloads = vec![direct_payload("cov_k", 0)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 2);
    assert_eq!(count_calls(&instructions), 0);
    // base = 0: local r0 stays at r0; Return becomes a real Move r2 <- r0,
    // i.e. the call's dst register receives the computed value.
    assert!(matches!(&instructions[0], Instruction::LoadIntConst { dst: 0, const_idx: 0 }));
    assert!(matches!(&instructions[1], Instruction::Move { dst: 2, src: 0 }));
}

#[test]
fn dst_equal_to_ret_src_deletes_return() {
    // identity(x): Move r1 <- r0; Return r1 — called with dst == 1.
    let body = vec![
        Instruction::Move { dst: 1, src: 0 },
        Instruction::Return { src: 1 },
    ];
    let mut instructions = vec![
        Instruction::LoadIntConst { dst: 2, const_idx: 1 },
        Instruction::Call { dst: 1, payload_idx: 0, first_arg: 2, arg_count: 1 },
    ];
    let funcs = one_func("cov_ident", make_chunk(&["x"], body));
    let payloads = vec![direct_payload("cov_ident", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    // dst == ret_src (pre-remap, 1 == 1) -> the Return slot is removed, so
    // the spliced body is 1 instruction, not 2.
    assert_eq!(instructions.len(), 2);
    assert_eq!(count_calls(&instructions), 0);
    assert!(!instructions.iter().any(|i| matches!(i, Instruction::Return { .. })));
    // The surviving Move is still register-remapped (base = 3).
    assert!(matches!(&instructions[1], Instruction::Move { dst: 3, src: 2 }));
}

// ── remap invariants ──────────────────────────────────────────────

#[test]
fn guard_register_255_and_jump_offset_survive_remap() {
    // guard(x): const -> r1; if r255 jump +7; return r1.
    let body = vec![
        Instruction::LoadIntConst { dst: 1, const_idx: 3 },
        Instruction::JumpIfFalse { src: 255, offset: 7 },
        Instruction::Return { src: 1 },
    ];
    let mut instructions = vec![Instruction::Call {
        dst: 2, payload_idx: 0, first_arg: 0, arg_count: 1,
    }];
    let funcs = one_func("cov_guard", make_chunk(&["x"], body));
    let payloads = vec![direct_payload("cov_guard", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 3);
    assert_eq!(count_calls(&instructions), 0);
    // base = 1: local r1 stays r1; reg 255 is exempt from remapping.
    assert!(matches!(&instructions[0], Instruction::LoadIntConst { dst: 1, const_idx: 3 }));
    assert!(matches!(&instructions[1], Instruction::JumpIfFalse { src: 255, offset: 7 }));
    // dst(2) != ret_src(1): a real Move carrying the value to the dst reg.
    assert!(matches!(&instructions[2], Instruction::Move { dst: 2, src: 1 }));
}

#[test]
fn immediates_and_const_indices_are_not_remaped() {
    // scale(x): r1 = const#9; r2 = r1 * -3; return r2.
    let body = vec![
        Instruction::LoadIntConst { dst: 1, const_idx: 9 },
        Instruction::IntMulI { dst: 2, src: 1, imm: -3 },
        Instruction::Return { src: 2 },
    ];
    let mut instructions = vec![Instruction::Call {
        dst: 4, payload_idx: 0, first_arg: 1, arg_count: 1,
    }];
    let funcs = one_func("cov_scale", make_chunk(&["x"], body));
    let payloads = vec![direct_payload("cov_scale", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 3);
    // base = 2: r1 -> r2, r2 -> r3; const_idx 9 and imm -3 pass through.
    assert!(matches!(&instructions[0], Instruction::LoadIntConst { dst: 2, const_idx: 9 }));
    assert!(matches!(&instructions[1], Instruction::IntMulI { dst: 3, src: 2, imm: -3 }));
    assert!(matches!(&instructions[2], Instruction::Move { dst: 4, src: 3 }));
}

#[test]
fn num_and_cmp_remap_arms() {
    // f2(a, b): r2 = a * b; r3 = (r2 == 5); return r3 — exercises the
    // NumMul and IntCmpI remap arms plus their untouched op/imm fields.
    let body = vec![
        Instruction::NumMul { dst: 2, src1: 0, src2: 1 },
        Instruction::IntCmpI { dst: 3, src: 2, imm: 5, op: 1 },
        Instruction::Return { src: 3 },
    ];
    let mut instructions = vec![Instruction::Call {
        dst: 6, payload_idx: 0, first_arg: 2, arg_count: 2,
    }];
    let funcs = one_func("cov_f2", make_chunk(&["a", "b"], body));
    let payloads = vec![direct_payload("cov_f2", 2)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 3);
    // base = 4: r0 -> r2, r1 -> r3, r2 -> r4, r3 -> r5.
    assert!(matches!(&instructions[0], Instruction::NumMul { dst: 4, src1: 2, src2: 3 }));
    assert!(matches!(&instructions[1], Instruction::IntCmpI { dst: 5, src: 4, imm: 5, op: 1 }));
    assert!(matches!(&instructions[2], Instruction::Move { dst: 6, src: 5 }));
}
