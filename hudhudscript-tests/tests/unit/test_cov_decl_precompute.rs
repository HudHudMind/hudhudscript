//! Coverage tests for `compiler/decl_precompute.rs` — the pre-compile AST
//! pass that classifies top-level names as "shared" (`shared_top_level_names`).
//!
//! A name becomes shared when a nested function body, closure, class
//! method, or loop-engineering step body references it
//! (`collect_shared` / `walk_refs`), or when a top-level control-flow
//! condition or initializer calls it (`collect_top_level_calls`).
//!
//! Observables used here (all public on `Bytecode`):
//!   - `DeclGlobal` emission: for a top-level `let`/`var` this instruction
//!     is emitted ONLY when `ct_is_shared_top_level` is true, and that
//!     predicate reads exclusively the precompute result — a pure signal.
//!   - `main_local_names` / `main_local_shared`: the shared bitmap (the
//!     precompute result ORed with runtime reference tracking).
//!   - `StoreConst` for `const`: emitted only when the name is shared.
//!   - VM execution parity for the programs under test.
//!
//! Top-level function *declarations* always compile to `StoreGlobal`
//! regardless of precompute, so the call-classification arms
//! (`collect_top_level_calls`) are pinned through execution behavior.

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

fn count_instrs(bc: &Bytecode, pred: impl Fn(&Instruction) -> bool) -> usize {
    let mut n = bc.instructions.iter().filter(|i| pred(i)).count();
    for chunk in bc.functions.borrow().iter() {
        n += chunk.instructions.iter().filter(|i| pred(i)).count();
    }
    n
}

fn count_decl_globals(bc: &Bytecode) -> usize {
    count_instrs(bc, |i| matches!(i, Instruction::DeclGlobal { .. }))
}

/// The shared bit for a top-level local. `None` means the name never
/// became a top-level local at all (asserted separately where relevant).
fn shared_bit(bc: &Bytecode, name: &str) -> Option<bool> {
    bc.main_local_names
        .iter()
        .position(|n| n == name)
        .map(|i| bc.main_local_shared[i])
}

fn int_var(vm: &VM, name: &str) -> i64 {
    vm.get_variable(name)
        .and_then(|v| v.as_int())
        .unwrap_or_else(|| panic!("variable {} must hold an int", name))
}

// ── function body references (collect_shared → walk_refs) ───────────────

#[test]
fn var_referenced_from_function_becomes_shared_global() {
    let src = "var g = 42
        fn f() { return g }
        var out = f()";
    let bc = compile(src);
    // Precompute must classify g as shared → DeclGlobal mirrors the local
    // into the globals table so f's LoadGlobal resolves.
    assert!(
        count_decl_globals(&bc) >= 1,
        "function-referenced top-level var must emit DeclGlobal"
    );
    assert_eq!(shared_bit(&bc, "g"), Some(true));
    let vm = run(src);
    assert_eq!(int_var(&vm, "out"), 42);
    assert_eq!(int_var(&vm, "g"), 42, "global mirror must hold the initializer");
}

#[test]
fn unreferenced_top_level_vars_stay_pure_locals() {
    let src = "var solo = 7
        var total = solo + 1";
    let bc = compile(src);
    assert_eq!(
        count_decl_globals(&bc),
        0,
        "no nested reference → no global mirror"
    );
    assert_eq!(shared_bit(&bc, "solo"), Some(false));
    assert_eq!(shared_bit(&bc, "total"), Some(false));
    assert_eq!(int_var(&run(src), "total"), 8);
}

#[test]
fn shared_classification_is_per_name_not_per_program() {
    let src = "var used = 1
        var unused = 2
        fn f() { return used }
        var out = f()";
    let bc = compile(src);
    assert_eq!(
        count_decl_globals(&bc),
        1,
        "exactly the referenced var is mirrored"
    );
    assert_eq!(shared_bit(&bc, "used"), Some(true));
    assert_eq!(shared_bit(&bc, "unused"), Some(false));
    assert_eq!(int_var(&run(src), "out"), 1);
}

#[test]
fn assignment_inside_function_marks_target_and_updates_global() {
    // walk_refs handles Assignment via mark_id(target) + mark_expr(value).
    let src = "var g = 1
        fn bump() { g = g + 1 }
        bump()";
    let bc = compile(src);
    assert!(count_decl_globals(&bc) >= 1, "assigned target must be shared");
    assert_eq!(shared_bit(&bc, "g"), Some(true));
    assert_eq!(
        int_var(&run(src), "g"),
        2,
        "mutation inside the function must be visible at top level"
    );
}

#[test]
fn expression_shapes_mark_index_and_binary_references() {
    let src = "var g = 2
        var arr = [10]
        fn f() { return arr[0] + g * 3 }
        var out = f()";
    let bc = compile(src);
    assert_eq!(shared_bit(&bc, "g"), Some(true));
    assert_eq!(shared_bit(&bc, "arr"), Some(true));
    assert!(
        count_decl_globals(&bc) >= 2,
        "both referenced vars must be mirrored"
    );
    assert_eq!(int_var(&run(src), "out"), 16, "10 + 2*3 = 16");
}

// ── closures in top-level expressions (walk_closures_in_expr) ───────────

#[test]
fn closure_in_var_initializer_captures_top_level_base() {
    let src = "var base = 10
        var add = (x) => x + base
        var r = add(5)";
    let bc = compile(src);
    assert!(count_decl_globals(&bc) >= 1, "captured base must be shared");
    assert_eq!(shared_bit(&bc, "base"), Some(true));
    assert_eq!(int_var(&run(src), "r"), 15);
}

#[test]
fn closure_inside_while_body_marks_outer_var() {
    // collect_shared recurses through While bodies to find the closure.
    let src = "var g = 7
        var i = 0
        var got = 0
        while (i < 2) {
            var f = () => g
            got = f()
            i = i + 1
        }";
    let bc = compile(src);
    assert_eq!(shared_bit(&bc, "g"), Some(true));
    assert!(count_decl_globals(&bc) >= 1);
    let vm = run(src);
    assert_eq!(int_var(&vm, "got"), 7);
    assert_eq!(int_var(&vm, "i"), 2);
}

// ── container recursion (collect_shared through If/Block/Try) ───────────

#[test]
fn function_nested_in_if_block_marks_outer_var() {
    let src = "var g = 7
        var picked = 0
        if (true) {
            fn pick() { return g }
            picked = pick()
        }";
    let bc = compile(src);
    assert_eq!(shared_bit(&bc, "g"), Some(true));
    assert!(count_decl_globals(&bc) >= 1);
    assert_eq!(int_var(&run(src), "picked"), 7);
}

#[test]
fn try_body_reference_marks_var_and_executes() {
    // walk_refs recurses through Try bodies; the try path must read the
    // mirrored global.
    let src = "var g = 9
        fn probe() {
            var r = 0
            try { r = g } catch (e) { r = -1 }
            return r
        }
        var out = probe()";
    let bc = compile(src);
    assert!(count_decl_globals(&bc) >= 1);
    assert_eq!(shared_bit(&bc, "g"), Some(true));
    assert_eq!(int_var(&run(src), "out"), 9);
}

// ── class methods (collect_shared → Class member bodies) ────────────────

#[test]
fn class_method_reference_marks_top_level_var() {
    let src = "var threshold = 5
        class C {
            constructor() { }
            public get() { return threshold }
        }
        let c = new C()
        let r = c.get()";
    let bc = compile(src);
    assert_eq!(shared_bit(&bc, "threshold"), Some(true));
    assert!(
        count_decl_globals(&bc) >= 1,
        "method-referenced var must be mirrored"
    );
    assert_eq!(int_var(&run(src), "r"), 5);
}

// ── const declarations (Stmt::Const shared discriminator) ───────────────

#[test]
fn const_shared_vs_local_store_const_discriminator() {
    // Shared const → StoreConst mirrors it; a const only read at top level
    // stays a plain local and emits no StoreConst.
    let shared_src = "const k = 5
        fn f() { return k }
        var out = f()";
    let shared_bc = compile(shared_src);
    assert!(
        count_instrs(&shared_bc, |i| matches!(i, Instruction::StoreConst { .. })) >= 1,
        "shared const must emit StoreConst"
    );
    assert_eq!(int_var(&run(shared_src), "out"), 5);

    let local_src = "const k = 6
        var v = k + 1";
    let local_bc = compile(local_src);
    assert_eq!(
        count_instrs(&local_bc, |i| matches!(i, Instruction::StoreConst { .. })),
        0,
        "non-shared const must not emit StoreConst"
    );
    assert_eq!(shared_bit(&local_bc, "k"), Some(false));
    assert_eq!(int_var(&run(local_src), "v"), 7);
}

// ── loop-engineering declarations (collect_shared_decl / walk_refs_decl) ─

#[test]
fn loop_step_body_reference_marks_top_level_var() {
    // Decl::Loop → InlineStep → walk_refs_decl must mark cfg when the step
    // body reads it. Gate conditions themselves are declarative (Decl::Gate
    // is intentionally not walked), so the reference lives in the step code.
    let src = "var cfg = 0
        loop L1 {
            step s1 {
                let r = cfg + 1
                gate g1 { when r == 0 -> done else -> fail }
            }
        }";
    let bc = compile(src);
    assert_eq!(shared_bit(&bc, "cfg"), Some(true));
    assert!(
        count_decl_globals(&bc) >= 1,
        "step-body reference must mirror cfg"
    );
    assert!(
        bc.has_function("__loop::L1"),
        "loop declaration must still produce its function chunk"
    );
}

// ── top-level calls (collect_top_level_calls → mark_call_expr) ──────────

#[test]
fn top_level_call_in_while_condition_drives_loop() {
    // `more()` in a while condition is classified at precompute time; the
    // runtime behavior pins that the called function keeps working.
    let src = "var i = 0
        fn more() { i = i + 1; return i < 3 }
        while (more()) { }";
    assert_eq!(int_var(&run(src), "i"), 3, "more() fires for i=1,2 then fails");
}

#[test]
fn top_level_calls_in_if_condition_and_initializer() {
    let src = "fn five() { return 5 }
        var direct = five()
        var hit = 0
        if (five() == 5) { hit = 1 }";
    let vm = run(src);
    assert_eq!(int_var(&vm, "direct"), 5, "call in let-initializer");
    assert_eq!(int_var(&vm, "hit"), 1, "call in if-condition");
    assert_eq!(
        shared_bit(&compile(src), "direct"),
        Some(false),
        "plain initializer result stays local"
    );
}
