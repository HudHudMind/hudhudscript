//! A2.3 Bug3: indexOf returns Int, not Float.

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
fn indexof_returns_int() {
    let src = "fn __t() { let s = \"abcdef\"; return s.indexOf(\"a\"); } __t()";
    let r = run(src);
    assert_eq!(r, "0", "indexOf should return Int 0");
}

#[test]
fn indexof_plus_three_mod_large() {
    let src = "fn __t() { let s = \"abcdef\"; let z = s.indexOf(\"c\"); return (z + 3) % 1000003; } __t()";
    let r = run(src);
    assert_eq!(r, "5", "indexOf(2)+3 % 1000003 should be 5");
}

#[test]
fn indexof_not_found_returns_minus_one_int() {
    let src = "fn __t() { let s = \"abc\"; return s.indexOf(\"z\"); } __t()";
    let r = run(src);
    assert_eq!(r, "-1", "indexOf not found should return Int -1");
}

#[test]
fn indexof_mod_small_modulus() {
    let src = "fn __t() { let s = \"abcdef\"; let z = s.indexOf(\"a\"); let t = z + 6; return t % 7; } __t()";
    let r = run(src);
    assert_eq!(r, "6", "6 % 7 should be 6");
}
