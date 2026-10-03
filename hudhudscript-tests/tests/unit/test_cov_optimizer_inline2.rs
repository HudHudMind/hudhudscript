//! Coverage tests for `optimizer/inline.rs` — rejection gates & scan behavior.
//!
//! Companion to `test_cov_optimizer_inline.rs`. These tests pin the pass's
//! REFUSAL paths (size window [2, 8], purity guards, payload/table lookups,
//! fused-return skip) and two deterministic characteristics of the splicer:
//!   - after an inline, the scan index advances to `i + body_len + 1`, so the
//!     instruction immediately following an inlined block is never examined;
//!   - a nested Call inside a callee has no remap arm and is cloned verbatim
//!     with its callee-frame register operands.
//!
//! All expected values are hand-traced from `inline.rs`; nothing is derived
//! at runtime from the pass itself.

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

fn one_func(name: &str, chunk: FunctionChunk) -> HashMap<String, Arc<FunctionChunk>> {
    let mut m = HashMap::new();
    m.insert(name.to_string(), Arc::new(chunk));
    m
}

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

/// `add1(x) = x + 1` — canonical inlinable body (2 instructions, pure).
fn add1_body() -> Vec<Instruction> {
    vec![
        Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
        Instruction::Return { src: 1 },
    ]
}

/// Canonical 3-instruction caller with one call to payload 0.
fn one_call_caller() -> Vec<Instruction> {
    vec![
        Instruction::LoadIntConst { dst: 0, const_idx: 0 },
        Instruction::LoadIntConst { dst: 2, const_idx: 1 },
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 },
    ]
}

// ── size window [2, 8] ────────────────────────────────────────────

#[test]
fn body_over_eight_instructions_is_rejected() {
    // 8 x IntAddI + Return = 9 instructions: one past the upper bound.
    let mut body: Vec<Instruction> = (1i16..=8)
        .map(|k| Instruction::IntAddI { dst: 1, src: 0, imm: k })
        .collect();
    body.push(Instruction::Return { src: 1 });

    let mut instructions = one_call_caller();
    let funcs = one_func("cov_big", make_chunk(&["x"], body));
    let payloads = vec![direct_payload("cov_big", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 3, "9-instr callee exceeds the size gate");
    assert_eq!(count_calls(&instructions), 1);
    assert!(matches!(
        &instructions[2],
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 }
    ));
}

#[test]
fn body_of_exactly_eight_is_inlined() {
    // 7 x IntAddI + Return = 8 instructions: the inclusive upper bound.
    let mut body: Vec<Instruction> = (0..7)
        .map(|_| Instruction::IntAddI { dst: 1, src: 0, imm: 7 })
        .collect();
    body.push(Instruction::Return { src: 1 });

    let mut instructions = one_call_caller();
    let funcs = one_func("cov_edge8", make_chunk(&["x"], body));
    let payloads = vec![direct_payload("cov_edge8", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 10, "3 - 1 Call + 8 body instructions");
    assert_eq!(count_calls(&instructions), 0);
    // base = 3: every IntAddI becomes r3 = r2 + 7; Return -> Move{3,3}.
    for idx in 2..=8 {
        assert!(
            matches!(&instructions[idx], Instruction::IntAddI { dst: 3, src: 2, imm: 7 }),
            "index {idx} must be the remapped IntAddI"
        );
    }
    assert!(matches!(&instructions[9], Instruction::Move { dst: 3, src: 3 }));
}

#[test]
fn single_instruction_body_is_rejected() {
    // Body of just [Return] is below the lower bound of the size window.
    let body = vec![Instruction::Return { src: 0 }];

    let mut instructions = vec![
        Instruction::LoadIntConst { dst: 2, const_idx: 1 },
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 },
    ];
    let funcs = one_func("cov_tiny", make_chunk(&["x"], body));
    let payloads = vec![direct_payload("cov_tiny", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 2, "1-instr callee is below the size gate");
    assert!(matches!(
        &instructions[1],
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 }
    ));
}

// ── purity guards ─────────────────────────────────────────────────

#[test]
fn impure_bodies_are_rejected() {
    // Each body is inside the [2, 8] window but trips a purity guard:
    // global write, throw, array write, loop, try, property write.
    let cases: Vec<(&str, Vec<Instruction>)> = vec![
        ("store_global", vec![
            Instruction::StoreGlobal { src: 0, sym: 42 },
            Instruction::Return { src: 0 },
        ]),
        ("throw", vec![
            Instruction::Throw { src: 0 },
            Instruction::Return { src: 0 },
        ]),
        ("index_assign", vec![
            Instruction::IndexAssign { obj: 1, idx: 2, val: 0 },
            Instruction::Return { src: 0 },
        ]),
        ("loop_begin", vec![
            Instruction::LoopBegin(3),
            Instruction::Return { src: 0 },
        ]),
        ("try_begin", vec![
            Instruction::TryBegin(2),
            Instruction::Return { src: 0 },
        ]),
        ("set_property", vec![
            Instruction::SetProperty { dst: 2, obj: 1, val: 0, prop_sym: 7 },
            Instruction::Return { src: 0 },
        ]),
    ];

    for (name, body) in cases {
        let fn_name = format!("cov_impure_{name}");
        let mut instructions = one_call_caller();
        let funcs = one_func(&fn_name, make_chunk(&["x"], body));
        let payloads = vec![direct_payload(&fn_name, 1)];

        inline_small_functions(&mut instructions, &funcs, &payloads);

        assert_eq!(instructions.len(), 3, "{name}: must NOT be inlined");
        assert_eq!(count_calls(&instructions), 1, "{name}: Call must survive");
        assert!(
            matches!(
                &instructions[2],
                Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 }
            ),
            "{name}: Call must survive verbatim"
        );
    }
}

#[test]
fn self_recursion_is_rejected() {
    // Body calls payload 0 — the very payload being inlined -> recursion.
    let body = vec![
        Instruction::IntSubI { dst: 1, src: 0, imm: 1 },
        Instruction::Call { dst: 2, payload_idx: 0, first_arg: 3, arg_count: 1 },
        Instruction::Return { src: 2 },
    ];

    let mut instructions = vec![Instruction::Call {
        dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1,
    }];
    let funcs = one_func("cov_rec", make_chunk(&["x"], body));
    let payloads = vec![direct_payload("cov_rec", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 1, "recursive callee must not be inlined");
    assert!(matches!(
        &instructions[0],
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 }
    ));
}

#[test]
fn fused_return_body_is_skipped() {
    // The return scan hits IntAddReturn before any plain Return, which
    // disqualifies the candidate (fused returns need special handling).
    let body = vec![
        Instruction::IntAddI { dst: 1, src: 0, imm: 1 },
        Instruction::IntAddReturn { src1: 1, src2: 0 },
    ];

    let mut instructions = one_call_caller();
    let funcs = one_func("cov_fused", make_chunk(&["x"], body));
    let payloads = vec![direct_payload("cov_fused", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 3, "fused-return callee must not be inlined");
    assert_eq!(count_calls(&instructions), 1);
}

// ── lookup guards ─────────────────────────────────────────────────

#[test]
fn payload_index_out_of_range_is_skipped_without_panic() {
    // Payload table holds one entry; the Call references index 5.
    let mut instructions = vec![Instruction::Call {
        dst: 3, payload_idx: 5, first_arg: 2, arg_count: 1,
    }];
    let funcs = one_func("cov_a", make_chunk(&["x"], add1_body()));
    let payloads = vec![direct_payload("cov_a", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 1, "out-of-range payload_idx is a no-op");
    assert!(matches!(
        &instructions[0],
        Instruction::Call { dst: 3, payload_idx: 5, first_arg: 2, arg_count: 1 }
    ));
}

#[test]
fn unknown_callee_name_is_skipped() {
    // Payload resolves to "cov_ghost", which is absent from the table
    // (only "cov_present" exists) — the Call must pass through untouched.
    let mut instructions = one_call_caller();
    let funcs = one_func("cov_present", make_chunk(&["x"], add1_body()));
    let payloads = vec![direct_payload("cov_ghost", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 3, "unresolvable callee is a no-op");
    assert_eq!(count_calls(&instructions), 1);
    assert!(matches!(
        &instructions[2],
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 }
    ));
}

// ── scan-index and clone characteristics ──────────────────────────

#[test]
fn adjacent_calls_only_the_first_is_inlined() {
    // Two back-to-back calls to the same inlinable function. After the
    // first splice the scan index lands one PAST the instruction following
    // the inserted block (i = i + body_len + 1), so the second Call —
    // now directly after the block — is never examined.
    let mut instructions = vec![
        Instruction::LoadIntConst { dst: 2, const_idx: 1 },
        Instruction::Call { dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1 },
        Instruction::Call { dst: 4, payload_idx: 0, first_arg: 2, arg_count: 1 },
    ];
    let funcs = one_func("cov_adj", make_chunk(&["x"], add1_body()));
    let payloads = vec![direct_payload("cov_adj", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 4, "only the first of two Calls is replaced");
    assert_eq!(count_calls(&instructions), 1);
    assert!(matches!(&instructions[1], Instruction::IntAddI { dst: 3, src: 2, imm: 1 }));
    assert!(matches!(&instructions[2], Instruction::Move { dst: 3, src: 3 }));
    // The skipped second Call survives verbatim.
    assert!(matches!(
        &instructions[3],
        Instruction::Call { dst: 4, payload_idx: 0, first_arg: 2, arg_count: 1 }
    ));
}

#[test]
fn nested_call_is_cloned_verbatim_and_not_reinlined() {
    // outer(x) calls inner (a different symbol, so the recursion guard does
    // not fire). inner is itself inlinable, but: (a) Call has no remap arm,
    // so it is cloned with its callee-frame operands, and (b) the scan
    // index jumps past the spliced block, so the clone is never revisited.
    let outer = vec![
        Instruction::IntSubI { dst: 1, src: 0, imm: 1 },
        Instruction::Call { dst: 2, payload_idx: 1, first_arg: 3, arg_count: 1 },
        Instruction::Return { src: 2 },
    ];
    let mut funcs = one_func("cov_inner", make_chunk(&["x"], add1_body()));
    funcs.insert("cov_outer".to_string(), Arc::new(make_chunk(&["x"], outer)));

    let mut instructions = vec![Instruction::Call {
        dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1,
    }];
    let payloads = vec![direct_payload("cov_outer", 1), direct_payload("cov_inner", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 3, "outer body spliced, inner clone kept");
    // base = 3: IntSubI local r1 -> r3, param r0 -> r2.
    assert!(matches!(&instructions[0], Instruction::IntSubI { dst: 3, src: 2, imm: 1 }));
    // The nested Call keeps its callee-frame registers (dst 2, first_arg 3).
    assert!(matches!(
        &instructions[1],
        Instruction::Call { dst: 2, payload_idx: 1, first_arg: 3, arg_count: 1 }
    ));
    // Return { src: 2 } -> Move { dst: 3, src: remap(2) }: register 2 is
    // past the parameter window (arg_count = 1), so remap_reg maps it to
    // base + (2 - 1) = 3 + 1 = 4 — a fresh local slot, NOT the call dst.
    assert!(matches!(&instructions[2], Instruction::Move { dst: 3, src: 4 }));
}

// ── non-mutation and no-op guarantees ─────────────────────────────

#[test]
fn callee_body_is_not_mutated_by_inlining() {
    let mut instructions = vec![Instruction::Call {
        dst: 3, payload_idx: 0, first_arg: 2, arg_count: 1,
    }];
    let funcs = one_func("cov_src", make_chunk(&["x"], add1_body()));
    let payloads = vec![direct_payload("cov_src", 1)];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    // The Arc<FunctionChunk> body stays intact: splicing operates on a
    // remapped copy, never on the shared callee instructions.
    let chunk = funcs.get("cov_src").expect("chunk present");
    assert_eq!(chunk.instructions.len(), 2);
    assert!(matches!(&chunk.instructions[0], Instruction::IntAddI { dst: 1, src: 0, imm: 1 }));
    assert!(matches!(&chunk.instructions[1], Instruction::Return { src: 1 }));
    // And the caller did receive the splice.
    assert_eq!(instructions.len(), 2);
}

#[test]
fn caller_without_calls_is_untouched() {
    let mut instructions = vec![
        Instruction::LoadIntConst { dst: 0, const_idx: 0 },
        Instruction::Return { src: 0 },
    ];
    let funcs: HashMap<String, Arc<FunctionChunk>> = HashMap::new();
    let payloads: Vec<CallPayload> = vec![];

    inline_small_functions(&mut instructions, &funcs, &payloads);

    assert_eq!(instructions.len(), 2, "no Call -> no transformation");
    assert!(matches!(&instructions[0], Instruction::LoadIntConst { dst: 0, const_idx: 0 }));
    assert!(matches!(&instructions[1], Instruction::Return { src: 0 }));
}
