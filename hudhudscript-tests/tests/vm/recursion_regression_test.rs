//! RECURSION_BUG regression tests — verify deep nested call correctness.
//! ack(3,6)==509, ack(3,3)==61 etc. (Kural 4: kanıt, iddia değil).

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run_ack(m: i64, n: i64) -> i64 {
    let src = format!(
        r#"
        fn ack(m, n) {{
            if (m == 0) {{ return n + 1; }}
            if (n == 0) {{ return ack(m - 1, 1); }}
            return ack(m - 1, ack(m, n - 1));
        }}
        let result = ack({m}, {n});
    "#
    );
    let ast = parse(&src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bc).unwrap();
    vm.get_global("result")
        .and_then(|v| v.as_int())
        .unwrap_or(-1)
}

#[test]
fn test_ack_3_6_eq_509() {
    assert_eq!(run_ack(3, 6), 509);
}

#[test]
fn test_ack_3_3_eq_61() {
    assert_eq!(run_ack(3, 3), 61);
}

#[test]
fn test_ack_2_13_eq_29() {
    assert_eq!(run_ack(2, 13), 29);
}

#[test]
fn test_ack_2_12_eq_27() {
    assert_eq!(run_ack(2, 12), 27);
}

#[test]
fn test_deep_temp_registers_h10_eq_85() {
    // h(0)=0; h(n)=n+h(n-1) — tests temp register preservation in deep recursion
    let src = r#"
        fn h(n) {
            if (n == 0) { return 0; }
            return n + h(n - 1);
        }
        let result = h(10);
    "#;
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bc).unwrap();
    let v = vm.get_global("result").and_then(|v| v.as_int()).unwrap();
    assert_eq!(v, 55, "h(10) = sum 1..10 = 55, got {v}");
}
