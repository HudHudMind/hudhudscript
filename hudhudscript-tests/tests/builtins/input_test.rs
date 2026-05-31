// INPUT0007: stdin integration tests — basic sanity checks.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

#[test]
fn test_input_smoke() {
    // Just verify the input builtin is registered and doesn't crash the VM
    let src = r#"let x = input("> "); print(x)"#;
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    // Don't actually call input() in test (blocks on stdin) — just verify compilation
}

#[test]
fn test_oku_alias_compiles() {
    let src = r#"let x = oku("> "); print(x)"#;
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    let bc = c.compile(&ast).unwrap();
    // Just verify it compiles
}

#[test]
fn test_stdin_number_compiles() {
    let src = r#"let n = stdin.int("> "); print(n)"#;
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    c.compile(&ast).unwrap();
}
