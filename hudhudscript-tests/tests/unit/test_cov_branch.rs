//! Coverage tests for the VM branch execution paths
//! (`crates/hudhudscript-vm/src/vm/execute/branch.rs`): if/else dispatch,
//! truthiness of every value family, fused loop-condition jumps and the
//! increment/decrement jump fusions. Tests compile real scripts and pin
//! exact values taken from the VM semantics.

use hudhudscript_bytecode::{Bytecode, Instruction, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile(src: &str) -> Bytecode {
    let ast = parse(src).expect("test source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("test source must compile")
}

fn run_ok(src: &str) -> VM {
    let bytecode = compile(src);
    let mut vm = VM::new();
    vm.execute(&bytecode)
        .unwrap_or_else(|e| panic!("script must run cleanly: {}", e.message));
    vm
}

fn var(vm: &VM, name: &str) -> Value16 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("variable '{}' must be published", name))
}

fn int_of(vm: &VM, name: &str) -> i64 {
    var(vm, name)
        .as_int()
        .unwrap_or_else(|| panic!("'{}' must hold an int", name))
}

fn string_of(vm: &VM, name: &str) -> String {
    var(vm, name)
        .as_string()
        .unwrap_or_else(|| panic!("'{}' must hold a string", name))
}

fn count_matching(bytecode: &Bytecode, pred: fn(&Instruction) -> bool) -> usize {
    let in_main = bytecode.instructions.iter().filter(|i| pred(i)).count();
    let chunks = bytecode.functions.borrow();
    let in_functions: usize = chunks
        .iter()
        .map(|chunk| chunk.instructions.iter().filter(|i| pred(i)).count())
        .sum();
    in_main + in_functions
}

fn is_jump_if_false(i: &Instruction) -> bool {
    matches!(i, Instruction::JumpIfFalse { .. })
}
fn is_jump_if_true(i: &Instruction) -> bool {
    matches!(i, Instruction::JumpIfTrue { .. })
}

// ── if / else dispatch ────────────────────────────────────────────────

#[test]
fn if_else_returns_matching_arm() {
    let src = r#"
        fn pick(c) {
            if (c) { return 1; } else { return 2; }
        }
        let yes = pick(7);
        let no = pick(0);
    "#;
    assert!(
        count_matching(&compile(src), is_jump_if_false) >= 1,
        "the condition must lower to JumpIfFalse"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "yes"), 1, "truthy condition takes the then-arm");
    assert_eq!(int_of(&vm, "no"), 2, "falsy condition takes the else-arm");
}

#[test]
fn if_without_else_falls_through() {
    let src = r#"
        fn f(c) {
            if (c) { return 10; }
            return 20;
        }
        let hit = f(1);
        let miss = f(0);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "hit"), 10, "taken branch returns early");
    assert_eq!(int_of(&vm, "miss"), 20, "skipped branch falls through");
}

#[test]
fn else_if_chain_selects_matching_arm() {
    let src = r#"
        fn grade(score) {
            if (score >= 90) { return "A"; }
            else if (score >= 80) { return "B"; }
            else if (score >= 70) { return "C"; }
            else { return "F"; }
        }
        let a = grade(95);
        let b = grade(85);
        let c = grade(72);
        let f = grade(10);
    "#;
    let vm = run_ok(src);
    assert_eq!(string_of(&vm, "a"), "A");
    assert_eq!(string_of(&vm, "b"), "B");
    assert_eq!(string_of(&vm, "c"), "C");
    assert_eq!(string_of(&vm, "f"), "F");
}

#[test]
fn nested_if_branches() {
    let src = r#"
        fn sign(x) {
            if (x > 0) {
                if (x > 100) { return 2; }
                return 1;
            } else {
                if (x == 0) { return 0; }
                return -1;
            }
        }
        let big = sign(150);
        let small = sign(5);
        let zero = sign(0);
        let neg = sign(-9);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "big"), 2);
    assert_eq!(int_of(&vm, "small"), 1);
    assert_eq!(int_of(&vm, "zero"), 0);
    assert_eq!(int_of(&vm, "neg"), -1);
}

#[test]
fn ternary_selects_on_truthiness() {
    let src = r#"
        fn label(c) { return c ? "yes" : "no"; }
        let a = label(1);
        let b = label(0);
        let c = label("");
    "#;
    let vm = run_ok(src);
    assert_eq!(string_of(&vm, "a"), "yes");
    assert_eq!(string_of(&vm, "b"), "no");
    assert_eq!(string_of(&vm, "c"), "no", "empty string is falsy");
}

// ── truthiness of every value family (Value16::is_truthy) ─────────────

#[test]
fn branch_on_int_truthiness() {
    let src = r#"
        fn t(c) { if (c) { return 1; } return 2; }
        let zero = t(0);
        let five = t(5);
        let neg = t(-3);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "zero"), 2, "int 0 is falsy");
    assert_eq!(int_of(&vm, "five"), 1, "non-zero int is truthy");
    assert_eq!(int_of(&vm, "neg"), 1, "negative int is truthy");
}

#[test]
fn branch_on_float_truthiness() {
    let src = r#"
        fn t(c) { if (c) { return 1; } return 2; }
        let zero = t(0.0);
        let half = t(0.5);
        let big = t(12.25);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "zero"), 2, "float 0.0 is falsy");
    assert_eq!(int_of(&vm, "half"), 1, "fractional float is truthy");
    assert_eq!(int_of(&vm, "big"), 1, "non-zero float is truthy");
}

#[test]
fn branch_on_string_truthiness() {
    let src = r#"
        fn t(c) { if (c) { return 1; } return 2; }
        let empty = t("");
        let short = t("x");
        let long = t("a-long-dynamic-heap-string");
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "empty"), 2, "empty inline string is falsy");
    assert_eq!(int_of(&vm, "short"), 1, "short inline string is truthy");
    assert_eq!(
        int_of(&vm, "long"),
        1,
        "dynamic (>15 byte) heap string is truthy"
    );
}

#[test]
fn branch_on_null_and_bool_truthiness() {
    let src = r#"
        fn t(c) { if (c) { return 1; } return 2; }
        let n = t(null);
        let yes = t(true);
        let no = t(false);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "n"), 2, "null is falsy");
    assert_eq!(int_of(&vm, "yes"), 1, "true is truthy");
    assert_eq!(int_of(&vm, "no"), 2, "false is falsy");
}

#[test]
fn arrays_are_truthy_even_when_empty_or_zeroed() {
    // Dynamic non-string values (arrays) are always truthy.
    let src = r#"
        fn t(c) { if (c) { return 1; } return 2; }
        let empty = t([]);
        let zeroed = t([0]);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "empty"), 1, "empty array is truthy");
    assert_eq!(int_of(&vm, "zeroed"), 1, "array holding 0 is truthy");
}

#[test]
fn not_operator_inverts_branch() {
    // `!c` lowers to Not + JumpIfFalse which peephole folds to JumpIfTrue.
    let src = r#"
        fn n(c) { if (!c) { return 1; } return 2; }
        let a = n(0);
        let b = n(1);
        let c = n("");
        let d = n("x");
    "#;
    assert!(
        count_matching(&compile(src), is_jump_if_true) >= 1,
        "Not + JumpIfFalse must fold to JumpIfTrue"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "a"), 1, "!0 branches");
    assert_eq!(int_of(&vm, "b"), 2, "!1 skips");
    assert_eq!(int_of(&vm, "c"), 1, "!\"\" branches");
    assert_eq!(int_of(&vm, "d"), 2, "!\"x\" skips");
}

// ── loop-condition branches (fused compare-and-jump) ──────────────────

#[test]
fn while_lt_loop_condition() {
    let src = r#"
        fn sum_lt(n) {
            let i = 0;
            let s = 0;
            while (i < n) { s = s + i; i = i + 1; }
            return s;
        }
        let five = sum_lt(5);
        let zero = sum_lt(0);
        let one = sum_lt(1);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "five"), 10, "0+1+2+3+4");
    assert_eq!(int_of(&vm, "zero"), 0, "strict < never runs at n=0");
    assert_eq!(int_of(&vm, "one"), 0, "body runs but adds 0");
}

#[test]
fn while_le_loop_is_inclusive() {
    let src = r#"
        fn sum_le(n) {
            let i = 0;
            let s = 0;
            while (i <= n) { s = s + i; i = i + 1; }
            return s;
        }
        let four = sum_le(4);
        let one = sum_le(1);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "four"), 10, "0+1+2+3+4 includes the bound");
    assert_eq!(int_of(&vm, "one"), 1, "0+1 includes the bound at n=1");
}

#[test]
fn while_le_compares_int_against_float_bound() {
    // Mixed Int/Number comparison in the loop condition.
    let src = r#"
        fn sumf(limit) {
            let i = 0;
            let s = 0;
            while (i <= limit) { s = s + i; i = i + 1; }
            return s;
        }
        let a = sumf(2.5);
        let b = sumf(1.5);
        let c = sumf(0.0);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "a"), 3, "i stops after 2 (2 <= 2.5)");
    assert_eq!(int_of(&vm, "b"), 1, "i stops after 1 (1 <= 1.5)");
    assert_eq!(int_of(&vm, "c"), 0, "only i=0 satisfies 0 <= 0.0");
}

#[test]
fn countdown_while_loop_with_sub_decrement() {
    let src = r#"
        fn blast(n) {
            let i = n;
            let c = 0;
            while (i > 0) { c = c + 1; i = i - 1; }
            return c;
        }
        let four = blast(4);
        let zero = blast(0);
        let one = blast(1);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "four"), 4, "counts every iteration");
    assert_eq!(int_of(&vm, "zero"), 0, "loop skipped at n=0");
    assert_eq!(int_of(&vm, "one"), 1, "single decrement exits");
}

#[test]
fn cstyle_for_loop_runs_update() {
    let src = r#"
        fn squares(n) {
            let s = 0;
            for (let i = 1; i <= n; i = i + 1) { s = s + i * i; }
            return s;
        }
        let three = squares(3);
        let one = squares(1);
        let zero = squares(0);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "three"), 14, "1+4+9");
    assert_eq!(int_of(&vm, "one"), 1);
    assert_eq!(int_of(&vm, "zero"), 0, "1 <= 0 fails on first check");
}

#[test]
fn equality_branches_against_immediates() {
    let src = r#"
        fn tier(x) {
            if (x == 5) { return 50; }
            if (x == 7) { return 70; }
            return 0;
        }
        let a = tier(5);
        let b = tier(7);
        let c = tier(9);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "a"), 50);
    assert_eq!(int_of(&vm, "b"), 70);
    assert_eq!(int_of(&vm, "c"), 0, "no arm matches");
}

#[test]
fn branch_inside_loop_early_return() {
    let src = r#"
        fn has_eight(n) {
            let i = 0;
            while (i < n) {
                if (i == 8) { return 1; }
                i = i + 1;
            }
            return 0;
        }
        let hit = has_eight(10);
        let miss = has_eight(5);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "hit"), 1, "returns from inside the loop");
    assert_eq!(int_of(&vm, "miss"), 0, "condition never met");
}
