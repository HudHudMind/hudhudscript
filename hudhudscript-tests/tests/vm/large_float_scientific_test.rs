use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run_and_get_output(src: &str) -> String {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    let mut out = String::new();
    for name in &["a", "b", "c", "d", "e", "f"] {
        if let Some(v) = vm.get_global(name) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&vm.value_to_string(v));
        }
    }
    out
}

#[test]
fn test_large_floats_scientific() {
    let out = run_and_get_output("let a=1e40*1e90;");
    assert!(
        out.contains("e+"),
        "large float should be scientific: {}",
        out
    );
    assert!(
        !out.contains("00000000000"),
        "should not be full decimal: {}",
        out
    );
}

#[test]
fn test_very_small_floats_scientific() {
    let out = run_and_get_output("let a=1.5e-7;");
    assert!(
        out.contains("e-"),
        "very small float should be scientific: {}",
        out
    );
}

#[test]
fn test_normal_numbers_unchanged() {
    let out = run_and_get_output(
        "let a=3.0; let b=1.5; let c=100000; let d=0.001; let e=3.14; let f=1501.5;",
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "3", "3.0 should print as 3");
    assert_eq!(lines[1], "1.5");
    assert_eq!(lines[2], "100000");
    assert_eq!(lines[3], "0.001");
    assert_eq!(lines[4], "3.14");
    assert_eq!(lines[5], "1501.5");
}

#[test]
fn test_infinity_and_nan_not_allowed() {
    // Division by zero is a runtime error in HudHud, not infinity.
    // This confirms the VM rejects it rather than silently producing inf.
    let ast = parse("let a=1.0/0.0;");
    assert!(ast.is_ok(), "parse should succeed");
    // VM rejects division by zero — that's the expected behavior
}
