//! ADIM B: capture filtresi regresyon testleri (ileri ref, karsilikli ozyineleme, ic ice fn)
use hudhudscript_vm::VM;
fn run(src: &str) -> Result<String, String> {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc)
        .map(|_| vm.last_return_value().display_string())
        .map_err(|e| format!("{}", e))
}
#[test]
fn b1_forward_ref() {
    let r =
        run(r#"fn main() { return helper(10); } fn helper(x) { return x * 2; } print(main());"#)
            .unwrap();
    assert_eq!(r, "20", "forward ref, got: {}", r);
}
#[test]
fn b1_mutual_recursion() {
    let r = run(r#"fn even(n) { if (n==0) { return true; } return odd(n-1); } fn odd(n) { if (n==0) { return false; } return even(n-1); } print(even(5));"#).unwrap();
    assert_eq!(r, "false", "mutual recursion, got: {}", r);
}
#[test]
fn b1_nested_fn() {
    let r = run(r#"fn outer() { fn inner(x) { return x+1; } return inner(5); } print(outer());"#)
        .unwrap();
    assert_eq!(r, "6", "nested fn, got: {}", r);
}
#[test]
fn b1_lang2_unchanged() {
    let r = run(
        r#"fn outer(){ let n=1; let inc=()=>{n=n+1;}; inc();inc(); return n; } print(outer());"#,
    )
    .unwrap();
    assert_eq!(r, "3", "LANG-2 fixed, got: {}", r);
}
