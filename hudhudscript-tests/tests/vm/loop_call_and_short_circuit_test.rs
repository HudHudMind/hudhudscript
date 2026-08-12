//! A2.2 Bug2: while condition with function call + && short-circuit
//! Exact repro — do not minimize.

use hudhudscript_vm::VM;

fn run(src: &str) -> String {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc).unwrap();
    vm.last_return_value().display_string()
}

#[test]
fn bug2_full_repro_returns_50() {
    let src = r#"
fn asc(a, b) { return a - b; }
fn work(a, low, high, cmp) {
    let i = low;
    let count = 0;
    while (i < high) {
        while (cmp(a[i], 500000) < 0 && i < high - 1) { i = i + 1; count = count + 1; }
        i = i + 1;
    }
    return count;
}
let arr = [];
let seed = 12345;
let i = 0;
while (i < 100) { seed = (seed * 16807) % 2147483647; arr.push(seed % 1000000); i = i + 1; }
work(arr, 0, 99, asc)
"#;
    let r = run(src);
    assert_eq!(r, "50", "Bug2: expected r=50, got r={}", r);
}

#[test]
fn bug2_naive_short_circuit_works() {
    let src = "fn __t() { fn f(x) { return x - 500000; } let a = [100000, 200000, 300000]; let i = 0; while (f(a[i]) < 0 && i < 2) { i = i + 1; } return i; } __t()";
    let r = run(src);
    assert_eq!(r, "2", "Naive short-circuit: expected i=2, got i={}", r);
}
