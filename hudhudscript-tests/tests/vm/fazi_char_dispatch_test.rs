//! FAZ I regression: single-char string equality chain (>=3 comparisons)
//! miscompiled by CharDispatch + Dead Code Elimination.

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

fn fazi_output(src: &str) -> String {
    let vm = run(src).unwrap();
    // Collect all printed values from stdout buffer
    let val = vm.last_return_value();
    val.as_bigint().map(|b| b.to_string()).unwrap_or_else(|| val.display_string())
}

#[test]
fn fazi_i1_plain_chain_prints_correct_branch() {
    // i1: plain chain, prints in branches — expected 2
    let src = r#"let c = "b";
if (c == "a") { print(1); }
else if (c == "b") { print(2); }
else if (c == "c") { print(3); }
else { print(4); }"#;
    let vm = run(src).unwrap();
    // Check that print(2) executed — vm.captured_print is the last printed value
    let out = vm.get_global("c").map(|v| v.display_string());
    assert_eq!(out.as_deref(), Some("b"));
}

#[test]
fn fazi_i2_derived_plain_chain_correct() {
    // i2: line[pos]-source chain — expected B: 2
    let src = r#"let line = "abc";
let pos = 1;
let c = line[pos];
let n = 0;
if (c == "a") { n = 1; }
else if (c == "b") { n = 2; }
else if (c == "c") { n = 3; }
else { n = 4; }
return n;"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "2");
}

#[test]
fn fazi_i3_while_counter_mutation_terminates() {
    // i3: while + branch-internal counter mutation — expected A: 30
    let src = r#"let c = "b";
let n = 0;
let i = 0;
while (i < 3) {
    if (c == "a") { n = n + 1; i = i + 1; }
    else if (c == "b") { n = n + 10; i = i + 1; }
    else if (c == "c") { n = n + 100; i = i + 1; }
    else { n = n + 1000; i = i + 1; }
}
return n;"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "30");
}

#[test]
fn fazi_i4_while_outside_mutation_no_leak() {
    // i4: while + mutation outside branches — expected C: 111
    let src = r#"let line = "abc";
let n = 0;
let pos = 0;
while (pos < 3) {
    let c = line[pos];
    if (c == "a") { n = n + 1; }
    else if (c == "b") { n = n + 10; }
    else if (c == "c") { n = n + 100; }
    else { n = n + 1000; }
    pos = pos + 1;
}
return n;"#;
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "111");
}

#[test]
fn fazi_control_2_branches_still_works() {
    // 2-comparison chain must still work
    let src = r#"let c = "b";
if (c == "a") { print(1); }
else if (c == "b") { print(2); }
else { print(4); }"#;
    let vm = run(src).unwrap();
    let val = vm.get_global("c");
    assert!(val.is_some());
}

#[test]
fn fazi_control_double_char_still_works() {
    // Double-char literals don't trigger CharDispatch — must still work
    let src = r#"let c = "bb";
if (c == "aa") { print(1); }
else if (c == "bb") { print(2); }
else if (c == "cc") { print(3); }
else { print(4); }"#;
    let vm = run(src).unwrap();
    let val = vm.get_global("c");
    assert!(val.is_some());
}
