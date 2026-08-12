//! A2.4 Bug4: nested while(true) + double break

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<String, String> {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc).map(|_| vm.last_return_value().display_string()).map_err(|e| format!("{}", e))
}

#[test]
fn nested_while_true_break_returns_count() {
    let src = r#"fn __t() {
fn asc(a, b) { return a - b; }
fn work(a, low, high, cmp) {
    let i = low; let count = 0;
    while (i < high) {
        while (true) {
            if (i >= high - 1) { break; }
            if (cmp(a[i], 500000) >= 0) { break; }
            i = i + 1; count = count + 1;
        }
        i = i + 1;
    }
    return count;
}
let arr = [];
let seed = 12345;
let i = 0;
while (i < 100) { seed = (seed * 16807) % 2147483647; arr.push(seed % 1000000); i = i + 1; }
return work(arr, 0, 99, asc);
}
__t()"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "50", "nested while(true) break should return 50, got: {}", r);
}
