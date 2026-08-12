//! CROSS-1 — Tail-Call Optimisation (TCO) tests for the VM.
//!
//! These tests verify that self-recursive tail calls compile to
//! `Instruction::TailCall` and execute without pushing a new stack
//! frame, so deeply recursive computations that would blow the native
//! thread stack without TCO now terminate in O(1) stack space.
//!
//! Kural 1: tests are additive. These are NEW tests (TCO did not exist
//! before CROSS-1) and do not modify any existing tests.

use hudhudscript_bytecode::{Instruction, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run_named(src: &str, name: &str) -> Value16 {
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("vm execute failed");
    vm.get_variable(name).cloned().unwrap_or(Value16::null())
}

fn num(v: &Value16) -> f64 {
    v.as_number()
        .or_else(|| v.as_int().map(|i| i as f64))
        .unwrap_or_else(|| panic!("expected number, got {:?}", v))
}

/// The canonical TCO proof-of-life: a pure self-tail call that would
/// recurse ~100 000 frames deep without optimisation.  The default VM
/// thread stack is ~2 MB; a 100k-frame native recursion would overflow
/// with any realistic per-frame footprint.  With TCO, this terminates
/// in O(1) stack space with result 0.
#[test]
fn test_tco_self_recursive_loop_rec_does_not_stack_overflow() {
    let src = r#"
        function loop_rec(n) {
            if (n == 0) {
                return 0
            } else {
                return loop_rec(n - 1)
            }
        }

        let result = loop_rec(100000)
    "#;
    let v = run_named(src, "result");
    assert_eq!(num(&v), 0.0, "loop_rec(100000) should return 0 under TCO");
}

/// 500 000 iterations — even further beyond what the native stack
/// could hold — still terminates.  Proves the O(1) frame footprint.
#[test]
fn test_tco_self_recursive_loop_rec_500k() {
    let src = r#"
        function loop_rec(n) {
            if (n == 0) {
                return 0
            } else {
                return loop_rec(n - 1)
            }
        }

        let result = loop_rec(500000)
    "#;
    let v = run_named(src, "result");
    assert_eq!(num(&v), 0.0);
}

/// Verify the compiler actually emits `Instruction::TailCall` for the
/// `return loop_rec(n - 1)` branch — not just `Call + Return`.
#[test]
fn test_tco_emits_tail_call_instruction() {
    let src = r#"
        function loop_rec(n) {
            if (n == 0) {
                return 0
            } else {
                return loop_rec(n - 1)
            }
        }
    "#;
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let chunk = bytecode
        .get_function("loop_rec")
        .expect("loop_rec chunk missing");
    let has_tail_call = chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::TailCall { .. }));
    assert!(
        has_tail_call,
        "expected Instruction::TailCall in loop_rec body, got: {:?}",
        chunk.instructions
    );
}

/// fib(n) is NOT pure-tail (return fib(n-1) + fib(n-2) — the `+`
/// trails the call), so TailCall should NOT be emitted for fib.
/// Ensures the compiler's detection is conservative (no over-emission).
#[test]
fn test_tco_does_not_emit_for_non_tail_position() {
    let src = r#"
        function fib(n) {
            if (n < 2) { return n }
            return fib(n - 1) + fib(n - 2)
        }
    "#;
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let chunk = bytecode.get_function("fib").expect("fib chunk missing");
    let has_tail_call = chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::TailCall { .. }));
    assert!(
        !has_tail_call,
        "fib must NOT emit TailCall (return fib(n-1) + fib(n-2) is not a pure tail call), \
         got: {:?}",
        chunk.instructions
    );
}

/// Tail-call with multiple parameters — ensure every param slot is
/// rebound correctly on TCO re-entry.
#[test]
fn test_tco_multi_param_countdown() {
    let src = r#"
        function sumdown(n, acc) {
            if (n == 0) { return acc }
            return sumdown(n - 1, acc + n)
        }

        let result = sumdown(10000, 0)
    "#;
    let v = run_named(src, "result");
    // 1 + 2 + ... + 10000 = 10000 * 10001 / 2 = 50005000
    assert_eq!(num(&v), 50_005_000.0);
}

/// Ensure TCO preserves interpreter-parity semantics for a normal
/// (shallow) self-recursion — the optimisation must be behaviourally
/// transparent.
#[test]
fn test_tco_preserves_shallow_behaviour() {
    let src = r#"
        function count_to(target, cur) {
            if (cur >= target) { return cur }
            return count_to(target, cur + 1)
        }

        let result = count_to(42, 0)
    "#;
    let v = run_named(src, "result");
    assert_eq!(num(&v), 42.0);
}

/// Disassembly helper (manual run via `cargo test
/// tco_print_disassembly -- --nocapture --ignored`).  Prints the
/// bytecode of `loop_rec` and `sumdown` so the reader can verify the
/// `TailCall` emission pattern.  Gated with `#[ignore]` so it does
/// not pollute normal test output.
#[test]
#[ignore]
fn tco_print_disassembly() {
    let cases = [
        (
            "loop_rec",
            r#"
            function loop_rec(n) {
                if (n == 0) { return 0 }
                return loop_rec(n - 1)
            }
        "#,
        ),
        (
            "sumdown",
            r#"
            function sumdown(n, acc) {
                if (n == 0) { return acc }
                return sumdown(n - 1, acc + n)
            }
        "#,
        ),
        (
            "fib",
            r#"
            function fib(n) {
                if (n < 2) { return n }
                return fib(n - 1) + fib(n - 2)
            }
        "#,
        ),
    ];
    for (name, src) in cases {
        let ast = parse(src).expect("parse");
        let mut compiler = Compiler::new();
        let bc = compiler.compile(&ast).expect("compile");
        let chunk = bc.get_function(name).expect("chunk");
        println!("=== {} (local_count={}) ===", name, chunk.local_count);
        for (i, inst) in chunk.instructions.iter().enumerate() {
            println!("  {:3}: {:?}", i, inst);
        }
    }
}
