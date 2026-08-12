// CF1 regression test: const-fold correctness for top-level `let` variables
// mutated by function calls.  Ensures the compiler does NOT substitute the
// initial literal value for the variable in arithmetic expressions that
// follow a function call that mutated it.
//
// Introduced: f1c170e4d (v0.8.30, "A5: integer literals as i64/BigInt")
// Fixed:     v0.8.100 (CF1)
use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

// ======================================================================
// Test 1 — Basic double-fault repro from bug report:
//   seed=42; bump() → seed=43; (seed*16807)%499000 must be 223701 NOT 206894
// ======================================================================
#[test]
fn cf1_constfold_mutation_after_fn_call_223701() {
    let src = r#"
let seed = 42;
fn bump() {
    seed = seed + 1;
    return 0;
}
let x = bump();
return (seed * 16807) % 499000;
"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "223701",
        "Expected 223701=(43*16807)%499000, not 206894=(42*16807)%499000");
}

// ======================================================================
// Test 2 — Second proof: seed + literal after mutation
// ======================================================================
#[test]
fn cf1_direct_add_after_mutation() {
    let src = r#"
let seed = 42;
fn bump_twice() {
    seed = seed + 2;
    return 0;
}
let x = bump_twice();
return seed + 1;
"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "45",
        "seed=44 after bump_twice, +1 = 45");
}

// ======================================================================
// Test 3 — Legal fold: const variable declared with `const` should still fold
// ======================================================================
#[test]
fn cf1_const_variable_still_folds() {
    let src = r#"
const C = 100;
return C * 2;
"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "200");
}

// ======================================================================
// Test 4 — LCG pattern: multi-step seed evolution like in real workloads
// ======================================================================
#[test]
fn cf1_lcg_pattern_multi_step() {
    let src = r#"
let seed = 42;
let a = 16807;
let m = 499000;
fn lcg_step() {
    seed = (seed * a) % m;
    return seed;
}
let r1 = lcg_step();
let r2 = lcg_step();
let r3 = lcg_step();
return r3;
"#;
    let vm = run(src).unwrap();
    let r1: i64 = 206894;                        // (42*16807)%499000
    let r2: i64 = (r1 * 16807) % 499000;         // step 2
    let r3: i64 = (r2 * 16807) % 499000;         // step 3
    assert_eq!(vm.last_return_value().display_string(), r3.to_string(),
        "LCG step 3 must be computed from live seed values, not initial 42");
}

// ======================================================================
// Test 5 — Non-shared top-level: local-register optimization preserved
//   (v0.8.100 broke 8 benchmarks by always reloading — v0.8.101 fixes)
// ======================================================================
#[test]
fn cf1_non_shared_top_level_preserves_local_register() {
    let src = r#"
let a = [];
let b = [];
let i = 0;
while (i < 1) {
    a.push(i + 1);
    b.push(i + 2);
    i = i + 1;
}
let sum = 0;
let j = 0;
while (j < 1) {
    sum = sum + a[j] * b[j];
    j = j + 1;
}
return sum;
"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "2");
}

// ======================================================================
// Test 6 — Shared top-level: function mutation must be observed
//   (the original bug — v0.8.30-v0.8.99 returned stale local value)
// ======================================================================
#[test]
fn cf1_shared_top_level_observes_function_mutation() {
    let src = r#"
let seed = 42;
fn advance() {
    seed = seed + 1;
    return 0;
}
let x = advance();
return seed * 3;
"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "129",
        "seed=43 after advance(), 43*3=129");
}
