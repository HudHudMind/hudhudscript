//! Regression: exec_call_push_frame reg_size clobbered caller live registers.
//! Fix: reg_size = max(first_arg, callee_reg_count), mirroring fast_call_push_frame.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

#[test]
fn test_callee_window_does_not_clobber_caller_locals() {
    let src = r#"
fn is_prime(n) {
    if (n < 2) { return false; }
    if (n == 2) { return true; }
    if (n % 2 == 0) { return false; }
    let i = 3;
    while (i * i <= n) { if (n % i == 0) { return false; } i = i + 2; }
    return true;
}
let count = 0;
let n = 2;
let start = 777;
while (n <= 200) { if (is_prime(n)) { count = count + 1; } n = n + 1; }
let out_count = count;
let out_start = start;
"#;
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute must not fail");

    let count = vm.get_global("out_count").and_then(|v| v.as_int());
    let start = vm.get_global("out_start").and_then(|v| v.as_int());
    assert_eq!(count, Some(46), "primes up to 200 = 46");
    assert_eq!(start, Some(777), "start MUST NOT be clobbered to boolean");
}
