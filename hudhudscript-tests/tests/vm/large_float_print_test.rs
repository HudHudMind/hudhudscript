use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

/// Test that large float values print correctly (not i64::MAX)
#[test]
fn test_large_float_print_not_i64_max() {
    // 1.5^400 ≈ 3.7e70 — should NOT print as i64::MAX
    let src = "let r = 1.0; let i = 0; while (i < 400) { r = r * 1.5; i = i + 1; } let out = r;";
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    let val = vm.get_global("out").expect("out");
    let s = vm.value_to_string(val);
    // Should NOT be "9223372036854775807" (i64::MAX)
    assert_ne!(
        s, "9223372036854775807",
        "large float should not saturate to i64::MAX"
    );
    // Should contain digits and possibly dot
    assert!(
        s.len() > 10,
        "large float should print as many digits: {}",
        s
    );
}

/// Small whole floats still print as integers (3.0 → "3")
#[test]
fn test_small_whole_float_prints_as_int() {
    let (vm, _) = run("let out = 3.0;");
    let val = vm.get_global("out").expect("out");
    let s = vm.value_to_string(val);
    assert_eq!(s, "3", "3.0 should print as '3'");

    let (vm, _) = run("let out = 1501.5;");
    let val = vm.get_global("out").expect("out");
    let s = vm.value_to_string(val);
    assert_eq!(s, "1501.5", "1501.5 should print as-is");
}

fn run(src: &str) -> (VM, hudhudscript_bytecode::Bytecode) {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let bc_copy = bc.clone();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    (vm, bc_copy)
}
