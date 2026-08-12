//! Regression test for FAZ C fix (commit 12b09d07d).
//!
//! Bug: `restore_global` used wrong keyspace guard (`global_to_local.contains_key`)
//! for LoopBegin remap, causing `continue` inside while loops with push-built
//! arrays and function calls to jump to the wrong program counter — corrupting
//! array variables or triggering script re-execution.
//!
//! Fix: build a local→global reverse map and directly remap LoopBegin indices.
//!
//! This test exercises the minimal Dijkstra-like pattern that previously
//! triggered the bug: global push-built arrays, functions with inline heap
//! operations, and `continue` inside while loops.

use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> CompileResult<VM> {
    let stmts = parse(source).map_err(|e| {
        hudhudscript_bytecode::error::compile_codes::runtime_error(format!("Parse error: {:?}", e))
    })?;
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts)?;
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

#[test]
fn continue_push_reexec_does_not_corrupt_degs() {
    // Minimal Dijkstra-like pattern: global push-built arrays + heap functions
    // with continue in while loop. Previously this would corrupt deg[0] from 2
    // to large values due to incorrect LoopBegin remap.
    let source = r#"
let eto = []; eto.push(0); eto.push(1); eto.push(1); eto.push(0);
let ew = []; ew.push(5); ew.push(10); ew.push(5); ew.push(10);
let deg = []; deg.push(2); deg.push(2);
let start_ofs = []; start_ofs.push(0); start_ofs.push(2);
let visited = []; visited.push(0); visited.push(0);
let dist = []; dist.push(0); dist.push(999999999);
let hd = []; let hn = []; let hsz = 0;
fn hp(d, nd) { hn.push(nd); hd.push(d); let i2 = hsz; hsz = hsz + 1;
    while (i2 > 0) { let p = (i2 - 1) / 2; if (hd[p] <= hd[i2]) { break; }
        let t = hd[i2]; hd[i2] = hd[p]; hd[p] = t;
        t = hn[i2]; hn[i2] = hn[p]; hn[p] = t; i2 = p; } }
fn hpop() { if (hsz == 0) { return -1; } let r = hn[0]; hsz = hsz - 1;
    hn[0] = hn[hsz]; hd[0] = hd[hsz]; let i2 = 0;
    while (true) { let l = 2 * i2 + 1; let r2 = 2 * i2 + 2; let s = i2;
        if (l < hsz && hd[l] < hd[s]) { s = l; }
        if (r2 < hsz && hd[r2] < hd[s]) { s = r2; }
        if (s == i2) { break; }
        let t = hd[i2]; hd[i2] = hd[s]; hd[s] = t;
        t = hn[i2]; hn[i2] = hn[s]; hn[s] = t; i2 = s; } return r; }
hp(0, 0);
let iter = 0;
while (hsz > 0) { iter = iter + 1; if (iter > 4) { break; }
    let u = hpop(); if (u < 0) { break; }
    if (visited[u] == 1) { continue; }
    visited[u] = 1;
    let d = dist[u]; let base = start_ofs[u]; let k = 0;
    while (k < deg[u]) { let v = eto[base + k]; let w = ew[base + k]; let nd = d + w;
        if (nd < dist[v]) { dist[v] = nd; hp(nd, v); } k = k + 1; }
}
let final_deg0 = deg[0];
let final_deg1 = deg[1];
let final_eto_len = len(eto);
let final_iter = iter;
"#;

    let vm = run(source).expect("Execution should not crash or re-execute");

    // deg[0] must stay 2 — previously it would grow to 3008+ due to corruption
    let deg0 = vm.get_variable("final_deg0").expect("final_deg0 not found");
    let deg0_val = deg0.as_number().expect("deg0 should be number");
    assert!((deg0_val - 2.0).abs() < 1e-10, "deg[0] should be 2, got {}", deg0_val);

    // deg[1] must stay 2
    let deg1 = vm.get_variable("final_deg1").expect("final_deg1 not found");
    let deg1_val = deg1.as_number().expect("deg1 should be number");
    assert!((deg1_val - 2.0).abs() < 1e-10, "deg[1] should be 2, got {}", deg1_val);

    // eto length should stay 4
    let eto_len = vm.get_variable("final_eto_len").expect("final_eto_len not found");
    let eto_len_val = eto_len.as_number().expect("eto_len should be number");
    assert!((eto_len_val - 4.0).abs() < 1e-10, "len(eto) should be 4, got {}", eto_len_val);

    // iter should be ≤ 4 (bounded, not re-executed)
    let iter_val = vm.get_variable("final_iter").expect("final_iter not found");
    let iter_num = iter_val.as_number().expect("iter should be number");
    assert!(iter_num <= 4.0, "iter should be <= 4 (bounded), got {}", iter_num);
    assert!(iter_num >= 1.0, "iter should be >= 1 (executed), got {}", iter_num);
}

#[test]
fn loopbegin_remap_in_function_with_continue() {
    // After FAZ C fix, restore_global correctly remaps LoopBegin indices
    // using local→global map instead of the broken global_to_local guard.
    // This test ensures LoopBegin inside a function with continue still
    // maps to the correct global payload index.
    let source = r#"
let values = [];
fn collect(n) {
    let i = 0;
    while (i < n) {
        if (i == 2) { i = i + 1; continue; }
        values.push(i);
        i = i + 1;
    }
}
collect(5);
let len_vals = len(values);
let val0 = values[0];
let val1 = values[1];
let val2 = values[2];
"#;

    let vm = run(source).expect("Function with continue should execute normally");

    let len_vals = vm.get_variable("len_vals").expect("len_vals not found");
    let len_val = len_vals.as_number().expect("len_vals should be number");
    assert!((len_val - 4.0).abs() < 1e-10, "values should have 4 elements, got {}", len_val);

    let v0 = vm.get_variable("val0").expect("val0 not found");
    assert!((v0.as_number().expect("number") - 0.0).abs() < 1e-10);

    let v1 = vm.get_variable("val1").expect("val1 not found");
    assert!((v1.as_number().expect("number") - 1.0).abs() < 1e-10);

    // After skipping i=2, next element should be 3
    let v2 = vm.get_variable("val2").expect("val2 not found");
    assert!((v2.as_number().expect("number") - 3.0).abs() < 1e-10);
}
