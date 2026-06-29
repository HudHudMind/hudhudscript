use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run_and_get_string(src: &str, var: &str) -> String {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm.get_global(var)
        .map(|v| vm.value_to_string(v))
        .unwrap_or_default()
}

#[test]
fn test_sci_notation_1e40() {
    let s = run_and_get_string("let r = 1e40;", "r");
    assert!(s.starts_with("1"), "1e40 should start with 1, got: {}", s);
    assert_eq!(s, "1e+40", "1e40 should format as 1e+40, got: {}", s);
}

#[test]
fn test_sci_notation_1_5e_neg7() {
    let s = run_and_get_string("let r = 1.5e-7;", "r");
    assert_eq!(s, "1.5e-07", "1.5e-7 should format as 1.5e-07, got: {}", s);
}

#[test]
fn test_sci_notation_2_5E10() {
    let s = run_and_get_string("let r = 2.5E10;", "r");
    assert_eq!(s, "25000000000", "2.5E10 should be 25000000000, got: {}", s);
}

#[test]
fn test_sci_notation_regression_basic_numbers() {
    assert_eq!(run_and_get_string("let r = 1;", "r"), "1");
    assert_eq!(run_and_get_string("let r = 1.5;", "r"), "1.5");
    assert_eq!(run_and_get_string("let r = 100000;", "r"), "100000");
}
