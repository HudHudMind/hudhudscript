//! C3 full matrix: loop constructs × break/continue placements.
//! Guards against silent correctness regressions from LoopBegin/End hoisting.

use hudhudscript_vm::VM;

fn run_global(src: &str, name: &str) -> hudhudscript_bytecode::Value16 {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc).unwrap();
    vm.get_global(name)
        .unwrap_or(hudhudscript_bytecode::Value16::null())
}

fn assert_int(src: &str, var: &str, expected: i64) {
    let v = run_global(src, var);
    assert!(v.is_int(), "{} should be Int, got {:?}", var, v);
    assert_eq!(v.as_int(), Some(expected), "{} expected {}", var, expected);
}

// ── while × exits ───────────────────────────────────────────────
#[test]
fn while_plain_break() {
    // i=0,1,2 → break at i=3 → c=3
    let src =
        "let c = 0; let i = 0; while (i < 10) { if (i == 3) { break; } c = c + 1; i = i + 1; }";
    assert_int(src, "c", 3);
}

#[test]
fn while_plain_continue() {
    // i=0..9; skip c++ when i+1==3 → c=8? No: i starts at 0, body does i++ first then checks i==3
    // i=0→body: i=1, 1!=3→c++→c=1. i=1→body: i=2, c++→c=2. i=2→body: i=3, continue→c=3→wrong
    // Let me redo: body does i=i+1; if i==3 continue; c=c+1
    // i=0→body: i=1, c++→c=1. i=1→body: i=2, c++→c=2. i=2→body: i=3, continue→skips c++. i=3→body: i=4, c++→c=3... i=9→body: i=10→condition false→exit. c=8
    let src =
        "let c = 0; let i = 0; while (i < 10) { i = i + 1; if (i == 3) { continue; } c = c + 1; }";
    assert_int(src, "c", 9);
}

#[test]
fn while_break_in_switch() {
    // break inside switch exits switch only. c++ runs for all 5 iter.
    let src = "let c = 0; let i = 0; while (i < 5) { switch (i) { case 2: i = i + 1; break; } c = c + 1; i = i + 1; }";
    assert_int(src, "c", 4);
}

#[test]
fn while_continue_in_switch() {
    // continue inside switch targets the while loop.
    let src = "let c = 0; let i = 0; while (i < 5) { switch (i) { case 2: i = i + 1; continue; } c = c + 1; i = i + 1; }";
    assert_int(src, "c", 4);
}

#[test]
fn while_nested_break() {
    // outer 5 iter × inner 2 iter (break at j==2) = 10
    let src = "let c = 0; let i = 0; while (i < 5) { let j = 0; while (j < 5) { if (j == 2) { break; } c = c + 1; j = j + 1; } i = i + 1; }";
    assert_int(src, "c", 10);
}

#[test]
fn while_nested_continue() {
    // outer 5 iter × inner 4 iter (skip continue at j==2) = 20
    let src = "let c = 0; let i = 0; while (i < 5) { let j = 0; while (j < 5) { j = j + 1; if (j == 2) { continue; } c = c + 1; } i = i + 1; }";
    assert_int(src, "c", 20);
}

// ── for-cstyle × exits ──────────────────────────────────────────
#[test]
fn for_cstyle_break() {
    let src = "let c = 0; for (let i = 0; i < 10; i = i + 1) { if (i == 3) { break; } c = c + 1; }";
    assert_int(src, "c", 3);
}

#[test]
fn for_cstyle_continue() {
    let src =
        "let c = 0; for (let i = 0; i < 10; i = i + 1) { if (i == 3) { continue; } c = c + 1; }";
    assert_int(src, "c", 9);
}

#[test]
fn for_cstyle_break_in_switch() {
    // break inside switch exits switch only. for loop does NOT skip c++. All 5 iterations get c++.
    let src =
        "let c = 0; for (let i = 0; i < 5; i = i + 1) { switch (i) { case 2: break; } c = c + 1; }";
    assert_int(src, "c", 5);
}

#[test]
fn for_cstyle_continue_in_switch() {
    // continue inside switch targets the for loop (skip c++)
    let src = "let c = 0; for (let i = 0; i < 5; i = i + 1) { switch (i) { case 2: continue; } c = c + 1; }";
    assert_int(src, "c", 4);
}

#[test]
fn for_cstyle_nested_break() {
    // outer 5 iter × inner 2 iter (break at j==2) = 10
    let src = "let c = 0; for (let i = 0; i < 5; i = i + 1) { for (let j = 0; j < 5; j = j + 1) { if (j == 2) { break; } c = c + 1; } }";
    assert_int(src, "c", 10);
}

#[test]
fn for_cstyle_nested_continue() {
    // outer 5 iter × inner 4 iter (skip continue at j==2) = 20
    let src = "let c = 0; for (let i = 0; i < 5; i = i + 1) { for (let j = 0; j < 5; j = j + 1) { if (j == 2) { continue; } c = c + 1; } }";
    assert_int(src, "c", 20);
}

// FAZ C regression: continue in while loop with push-built arrays + fn calls
// that have their own while loops. Tests that continue jumps to the correct
// loop start (not a wrong payload index from function-merged loop payloads).
#[test]
fn while_continue_with_push_arrays_and_fn_calls() {
    let src = r#"
let eto = []; eto.push(0); eto.push(1);
let ew = []; ew.push(5); ew.push(10);
let deg = []; deg.push(2); deg.push(2);
let visited = []; visited.push(0); visited.push(0);
let dist = []; dist.push(0); dist.push(999999999);
let hd = []; let hn = []; let hsz = 0;
fn hp(d, nd) { hn.push(nd); hd.push(d); let i2 = hsz; hsz = hsz + 1;
    while (i2 > 0) { let p = (i2 - 1) / 2; if (hd[p] <= hd[i2]) { break; }
        let t = hd[i2]; hd[i2] = hd[p]; hd[p] = t; t = hn[i2]; hn[i2] = hn[p]; hn[p] = t; i2 = p; } }
fn hpop() { if (hsz == 0) { return -1; } let r = hn[0]; hsz = hsz - 1;
    hn[0] = hn[hsz]; hd[0] = hd[hsz]; let i2 = 0;
    while (true) { let l = 2 * i2 + 1; let r2 = 2 * i2 + 2; let s = i2;
        if (l < hsz && hd[l] < hd[s]) { s = l; }
        if (r2 < hsz && hd[r2] < hd[s]) { s = r2; }
        if (s == i2) { break; }
        let t = hd[i2]; hd[i2] = hd[s]; hd[s] = t; t = hn[i2]; hn[i2] = hn[s]; hn[s] = t; i2 = s; } return r; }
hp(0, 0);
let count = 0;
while (hsz > 0) {
    let u = hpop(); if (u < 0) { break; }
    if (visited[u] == 1) { count = count + 1; continue; }
    visited[u] = 1;
    let k = 0; while (k < deg[u]) { hp(dist[u] + 1, u); k = k + 1; }
}
"#;
    assert_int(src, "count", 2);
}
