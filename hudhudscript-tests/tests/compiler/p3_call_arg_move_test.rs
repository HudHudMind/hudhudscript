//! P3-A2: Call arg Move elimination regression tests.
//! Lock: argc==1 direct calls use arg register as first_arg, no Move.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

// ── Helpers ──────────────────────────────────────────────────

fn compile(src: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).unwrap()
}

fn run_global(src: &str, name: &str) -> i64 {
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm.get_variable(name)
        .and_then(|v| v.as_int())
        .unwrap_or(-999)
}

// ── Test 1: fib semantic ─────────────────────────────────────

#[test]
fn p3_fib_10_semantic_after_call_arg_move_elim() {
    let src = "function fib(n) { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } let out = fib(10);";
    assert_eq!(run_global(src, "out"), 55);
}

// ── Test 2: fib bytecode — no arg Move before Call ────────────

#[test]
fn p3_fib_bytecode_call_uses_arg_register_directly() {
    let src = "function fib(n) { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); }";
    let bc = compile(src);
    let fib = bc.get_function("fib").unwrap().instructions.clone();

    let mut calls = 0;
    for (i, instr) in fib.iter().enumerate() {
        let first_arg: Option<u8> = match instr {
            Instruction::Call { first_arg, .. } => {
                calls += 1;
                Some(*first_arg)
            }
            Instruction::IntSubCall1(_) => {
                calls += 1;
                None
            } // fused, no separate Call
            _ => None,
        };
        // Check previous instruction is NOT a Move to the same first_arg
        if let Some(fa) = first_arg {
            if i > 0 {
                if let Instruction::Move { dst, .. } = &fib[i - 1] {
                    assert_ne!(
                        *dst, fa,
                        "trailing Move to first_arg({}) before call at ip={}",
                        fa, i
                    );
                }
            }
        }
    }
    assert!(
        calls >= 2,
        "expected at least 2 call instructions in fib, got {calls}"
    );
}

// ── Test 3: multi-arg call still correct ──────────────────────

#[test]
fn p3_multi_arg_call_still_correct() {
    let src = "function add2(a, b) { return a + b; } let out = add2(10, 20);";
    assert_eq!(run_global(src, "out"), 30);
}

// ── Test 4: zero-arg call still correct ───────────────────────

#[test]
fn p3_zero_arg_call_still_correct() {
    let src = "function forty_two() { return 42; } let out = forty_two();";
    assert_eq!(run_global(src, "out"), 42);
}

// ── P3-A4: second sub-call fusion tests ──────────────────────

#[test]
fn p3_fib_bytecode_fuses_both_recursive_sub_calls() {
    let src = "function fib(n) { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); }";
    let bc = compile(src);
    let fib = bc.get_function("fib").unwrap().instructions.clone();

    let sub_call_count = fib
        .iter()
        .filter(|i| matches!(i, Instruction::IntSubCall1(_)))
        .count();
    assert_eq!(
        sub_call_count, 2,
        "both fib(n-1) and fib(n-2) must be IntSubCall1"
    );

    for win in fib.windows(2) {
        if let &[Instruction::IntSubI { dst, .. }, Instruction::Call {
            first_arg,
            arg_count,
            ..
        }] = win
        {
            assert!(
                !(dst == first_arg && arg_count == 1),
                "fusible IntSubI + one-arg Call remained unfused: dst={dst} first_arg={first_arg}"
            );
        }
    }
}

#[test]
fn p3_fib_subcall_payload_keeps_distinct_arg_regs() {
    let src = "function fib(n) { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); }";
    let bc = compile(src);
    let fib = bc.get_function("fib").unwrap().instructions.clone();

    let sub_call_indices: Vec<u32> = fib
        .iter()
        .filter_map(|instr| match instr {
            Instruction::IntSubCall1(idx) => Some(*idx),
            _ => None,
        })
        .collect();

    assert_eq!(
        sub_call_indices.len(),
        2,
        "expected 2 IntSubCall1, got {}",
        sub_call_indices.len()
    );

    let mut arg_regs: Vec<u8> = sub_call_indices
        .iter()
        .map(|idx| bc.super_instr_payloads[*idx as usize].arg_reg)
        .collect();
    arg_regs.sort_unstable();

    assert_eq!(
        arg_regs,
        vec![1, 3],
        "fib(n-1) and fib(n-2) must keep distinct arg registers"
    );
}
