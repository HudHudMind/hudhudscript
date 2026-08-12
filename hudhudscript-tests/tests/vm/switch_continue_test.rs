//! C3 regression guard: switch-inside-loop with continue/break.

use hudhudscript_vm::VM;

fn run_global(src: &str, name: &str) -> hudhudscript_bytecode::Value16 {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc).unwrap();
    vm.get_global(name).unwrap_or(hudhudscript_bytecode::Value16::null())
}

#[test]
fn switch_continue_targets_enclosing_loop() {
    let src = r#"
let i = 0;
let count = 0;
while (i < 5) {
    switch (i) { case 2: i = i + 1; continue; }
    count = count + 1;
    i = i + 1;
}
"#;
    let v = run_global(src, "count");
    assert!(v.is_int(), "result should be Int, got {:?}", v);
    assert_eq!(v.as_int(), Some(4), "switch case continue must target the loop");
}

#[test]
fn switch_break_does_not_hoist_loop() {
    let src = r#"
let i = 0;
let count = 0;
while (i < 5) {
    switch (i) { case 2: i = i + 1; break; }
    count = count + 1;
    i = i + 1;
}
"#;
    let v = run_global(src, "count");
    assert!(v.is_int(), "result should be Int, got {:?}", v);
    assert_eq!(v.as_int(), Some(4), "switch case break must exit switch only");
}
