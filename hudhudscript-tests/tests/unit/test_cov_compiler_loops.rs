//! Coverage tests for `compiler/stmt_shared/loops.rs` — the three loop
//! compilation entry points: `compile_for_in`, `compile_for_c_style` and
//! `compile_for_range`.
//!
//! Strategy (mirrors tests/compiler/g3_fusion_emit_tests.rs):
//!   - parse → compile → inspect the FINAL instruction stream (main chunk +
//!     all function chunks) so the assertions hold after optimization;
//!   - `ForIn`/`IterNext`/`Break`/`Continue` are opaque to the optimizer,
//!     so exact counts of those variants are stable;
//!   - the C-style fast path (both loop-condition operands are integer
//!     locals) emits `IntLtRRJumpIfFalse`/`IntLeRRJumpIfFalse`, while the
//!     slow path (literal/non-identifier operand) emits neither — used as
//!     the discriminator between the two code paths;
//!   - C-style loops whose body contains break/continue register a
//!     `LoopPayload` whose start (continue target = update clause) and end
//!     (break target) must be patched inside the instruction stream.
//!
//! ForRange loops are asserted at the emission level only: no executing
//! test for `for (start, stop)` exists anywhere in the suite and the
//! emitted exit branch reads a direction-comparison register, so pinning
//! runtime iteration counts here would encode unverified behavior.

use hudhudscript_bytecode::{Bytecode, Instruction};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile(src: &str) -> Bytecode {
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("compile failed")
}

fn run(src: &str) -> VM {
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute failed");
    vm
}

/// Count instructions matching `pred` across the main chunk and every
/// function chunk (inlining may move code between them; the total stays).
fn count_instrs(bc: &Bytecode, pred: impl Fn(&Instruction) -> bool) -> usize {
    let mut n = bc.instructions.iter().filter(|i| pred(i)).count();
    for chunk in bc.functions.borrow().iter() {
        n += chunk.instructions.iter().filter(|i| pred(i)).count();
    }
    n
}

fn int_var(vm: &VM, name: &str) -> i64 {
    vm.get_variable(name)
        .and_then(|v| v.as_int())
        .unwrap_or_else(|| panic!("variable {} must hold an int", name))
}

// ── for-in: iterator protocol emission ──────────────────────────────────

#[test]
fn for_in_emits_exactly_one_iterator_pair() {
    let bc = compile("var c = 0; for (x in [10, 20]) { c = c + x }");
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::ForIn { .. })),
        1,
        "one for-in loop must emit exactly one ForIn"
    );
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::IterNext { .. })),
        1,
        "one for-in loop must emit exactly one IterNext"
    );
    assert_eq!(int_var(&run("var c = 0; for (x in [10, 20]) { c = c + x }"), "c"), 30);
}

#[test]
fn nested_for_in_duplicates_iterator_instructions() {
    let src = "var t = 0;
        for (a in [1, 2]) {
            for (b in [10, 20]) {
                t = t + a * b
            }
        }";
    let bc = compile(src);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::ForIn { .. })), 2);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::IterNext { .. })), 2);
    // Inner body runs 2x2 times: (1*10 + 1*20) + (2*10 + 2*20) = 90.
    assert_eq!(int_var(&run(src), "t"), 90);
}

#[test]
fn for_in_zero_iterations_array_and_string() {
    // An exhausted iterator must fail IterNext immediately: the body never
    // runs, yet the ForIn/IterNext pair is still compiled.
    let arr = "var c = 0; for (x in []) { c = c + 1 }";
    let bc = compile(arr);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::ForIn { .. })), 1);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::IterNext { .. })), 1);
    assert_eq!(int_var(&run(arr), "c"), 0);

    let s = "var n = 0; for (ch in \"\") { n = n + 1 }";
    assert_eq!(int_var(&run(s), "n"), 0, "empty string iterates zero times");
}

#[test]
fn for_in_break_and_continue_skip_and_stop() {
    let src = "var kept = 0;
        for (v in [1, 2, 3, 4, 5]) {
            if (v == 3) { continue }
            if (v == 4) { break }
            kept = kept + v
        }";
    let bc = compile(src);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::Break)), 1);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::Continue)), 1);
    // v=1,2 accumulate; v=3 is skipped; v=4 breaks: 1 + 2 = 3.
    assert_eq!(int_var(&run(src), "kept"), 3);
    // compile_for_in never registers a LoopPayload — break/continue ride
    // the iterator stack instead of the LoopBegin machinery.
    assert!(
        bc.loop_payloads.is_empty(),
        "for-in must not allocate loop payloads"
    );
}

#[test]
fn for_in_string_yields_every_character() {
    let src = "var n = 0; for (ch in \"abc\") { n = n + 1 }";
    assert_eq!(int_var(&run(src), "n"), 3);
}

// ── while: break/continue parity through the shared target stack ────────

#[test]
fn while_break_and_continue_jump_targets() {
    let src = "var i = 0; var log = 0;
        while (i < 10) {
            i = i + 1
            if (i == 3) { continue }
            if (i == 6) { break }
            log = log + 1
        }";
    let bc = compile(src);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::Break)), 1);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::Continue)), 1);
    let vm = run(src);
    // i=1,2,4,5 accumulate (3 skipped, 6 breaks): 4 iterations counted.
    assert_eq!(int_var(&vm, "log"), 4);
    assert_eq!(int_var(&vm, "i"), 6, "break must fire before the update");
}

// ── C-style for: fused fast-path condition emission ─────────────────────

#[test]
fn c_style_fast_path_lt_identifier_condition() {
    // `i < limit` with both operands integer locals → fused register
    // compare+branch instead of IntCmp + JumpIfFalse.
    let src = "var limit = 5; var sum = 0;
        for (var i = 0; i < limit; i = i + 1) {
            sum = sum + i
        }";
    let bc = compile(src);
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::IntLtRRJumpIfFalse { .. })),
        1,
        "identifier-vs-identifier Lt condition must fuse exactly once"
    );
    assert_eq!(int_var(&run(src), "sum"), 10, "0+1+2+3+4 = 10");
}

#[test]
fn c_style_fast_path_le_inclusive_boundary() {
    let src = "var limit = 4; var sum = 0;
        for (var i = 0; i <= limit; i = i + 1) {
            sum = sum + i
        }";
    let bc = compile(src);
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::IntLeRRJumpIfFalse { .. })),
        1
    );
    // `<=` includes the bound: 0+1+2+3+4 = 10.
    assert_eq!(int_var(&run(src), "sum"), 10);
}

#[test]
fn c_style_fast_path_gt_swaps_operands() {
    // Gt has no dedicated opcode: it compiles as swapped Lt. Counting down
    // 3,2,1 verifies the swap keeps the comparison direction correct.
    let src = "var floor_v = 0; var sum = 0;
        for (var i = 3; i > floor_v; i = i - 1) {
            sum = sum + i
        }";
    let bc = compile(src);
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::IntLtRRJumpIfFalse { .. })),
        1,
        "Gt must emit the swapped IntLtRRJumpIfFalse"
    );
    assert_eq!(int_var(&run(src), "sum"), 6, "3+2+1 = 6");
}

#[test]
fn c_style_fast_path_ge_swaps_operands() {
    let src = "var floor_v = 1; var n = 0;
        for (var i = 3; i >= floor_v; i = i - 1) {
            n = n + 1
        }";
    let bc = compile(src);
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::IntLeRRJumpIfFalse { .. })),
        1,
        "Ge must emit the swapped IntLeRRJumpIfFalse"
    );
    assert_eq!(int_var(&run(src), "n"), 3, "3,2,1 all satisfy i >= 1");
}

#[test]
fn c_style_slow_path_literal_bound_emits_no_fused_rr_branch() {
    // `i < 5` has a literal right operand: get_reg() fails, so the generic
    // JumpIfFalse path compiles the condition and NO RR-fused branch exists.
    let src = "var sum = 0;
        for (var i = 0; i < 5; i = i + 1) {
            sum = sum + i
        }";
    let bc = compile(src);
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::IntLtRRJumpIfFalse { .. })),
        0,
        "literal bound must not take the register-compare fast path"
    );
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::IntLeRRJumpIfFalse { .. })),
        0
    );
    assert_eq!(int_var(&run(src), "sum"), 10);
}

#[test]
fn c_style_zero_iterations_tests_condition_first() {
    // Init already violates the condition: the body must never run, and
    // the fused branch is still emitted (pre-test loop semantics).
    let src = "var lim = 3; var s = 0;
        for (var i = 5; i < lim; i = i + 1) {
            s = s + 1
        }";
    let bc = compile(src);
    assert_eq!(
        count_instrs(&bc, |i| matches!(i, Instruction::IntLtRRJumpIfFalse { .. })),
        1
    );
    assert_eq!(int_var(&run(src), "s"), 0);
}

// ── C-style for: loop payloads (break/continue targets) ─────────────────

#[test]
fn c_style_continue_targets_update_and_payload_is_well_formed() {
    let src = "var s = 0;
        for (var i = 0; i < 8; i = i + 1) {
            if (i == 2) { continue }
            if (i == 5) { break }
            s = s + i
        }";
    let bc = compile(src);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::Continue)), 1);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::Break)), 1);
    // 0,1,3,4 accumulate: 8. If continue jumped to the condition instead
    // of the update clause, i would stick at 2 and the loop would hang.
    assert_eq!(int_var(&run(src), "s"), 8);

    assert_eq!(bc.loop_payloads.len(), 1, "one exiting C-style loop → one payload");
    let begin_ip = bc
        .instructions
        .iter()
        .position(|i| matches!(i, Instruction::LoopBegin(_)))
        .expect("LoopBegin must be emitted when the body exits");
    let idx = match &bc.instructions[begin_ip] {
        Instruction::LoopBegin(i) => *i as usize,
        _ => unreachable!("checked via position() above"),
    };
    let p = &bc.loop_payloads[idx];
    assert!(
        p.start > begin_ip as u32,
        "continue target (update clause) must lie after LoopBegin: start={} begin={}",
        p.start,
        begin_ip
    );
    assert!(p.end > p.start, "break target must lie after continue target");
    assert!(
        (p.end as usize) <= bc.instructions.len(),
        "payload end must stay inside the instruction stream"
    );
}

#[test]
fn c_style_no_condition_runs_until_break() {
    // Empty init/condition/update: the condition slot is a `true` constant
    // and only break can leave the loop.
    let src = "var c = 0;
        for (;;) {
            c = c + 1
            if (c == 3) { break }
        }";
    let bc = compile(src);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::Break)), 1);
    assert_eq!(bc.loop_payloads.len(), 1);
    assert_eq!(int_var(&run(src), "c"), 3);
}

// ── for-range: direction/step emission structure ────────────────────────

#[test]
fn range_default_step_emits_direction_logic_without_iterator_ops() {
    // No step given: the compiler must synthesize the ±1 step and both the
    // ascending and descending bound checks, without the for-in protocol.
    // The synthesized step is materialized as a `LoadNumConst 1.0` directly
    // feeding the increment, so the fuse_slot immediate pass rewrites the
    // `NumAdd` into `NumAddI { imm: 1 }` — the final stream carries the
    // immediate form, not the register NumAdd (explicit int steps keep a
    // plain NumAdd because `LoadIntConst` has no such fusion; see the
    // explicit-step test below).
    let bc = compile("for (0, 5) { }");
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::ForIn { .. })), 0);
    assert_eq!(count_instrs(&bc, |i| matches!(i, Instruction::IterNext { .. })), 0);
    assert!(
        count_instrs(&bc, |i| matches!(i, Instruction::IntLeRRJumpIfFalse { .. })) >= 2,
        "range needs an ascending and a descending bound check"
    );
    assert!(
        count_instrs(&bc, |i| matches!(i, Instruction::NumAddI { .. })) >= 1,
        "iterator increment must survive as the immediate NumAddI fusion"
    );
}

#[test]
fn range_explicit_step_asc_and_desc_compile() {
    // Explicit step bypasses the synthesized ±1 branch entirely.
    let asc = compile("for (0, 10, 2) { }");
    assert!(
        count_instrs(&asc, |i| matches!(i, Instruction::IntLeRRJumpIfFalse { .. })) >= 2
    );
    let desc = compile("for (10, 0, -1) { }");
    assert_eq!(
        count_instrs(&desc, |i| matches!(i, Instruction::ForIn { .. })),
        0,
        "descending range is not a for-in loop"
    );
    assert!(
        count_instrs(&desc, |i| matches!(i, Instruction::IntLeRRJumpIfFalse { .. })) >= 2
    );
    assert!(
        count_instrs(&desc, |i| matches!(i, Instruction::NumAdd { .. })) >= 1,
        "explicit -1 step still increments via NumAdd"
    );
}
