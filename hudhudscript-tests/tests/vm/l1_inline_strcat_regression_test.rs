//! L1: inline_compile const-fold register clobber regression test
//! v0.8.156'da `"U=" + f()` → "14" basıyordu (r0 clobber).

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<String, String> {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc).map(|_| vm.last_return_value().display_string()).map_err(|e| format!("{}", e))
}

#[test]
fn l1_strcat_inline_fn_returns_int() {
    let src = r#"
fn f() { return 7; }
let out = "U=" + f();
fn probe() { return out; }
print(probe());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "U=7", "V1: basic str+fn, got: {}", r);
}

#[test]
fn l1_strcat_inline_fn_left_operand() {
    let src = r#"
fn f() { return 7; }
let out = f() + "=U";
fn probe() { return out; }
print(probe());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "7=U", "V2: fn left, got: {}", r);
}

#[test]
fn l1_strcat_inline_triple() {
    let src = r#"
fn f() { return 7; }
let out = "U=" + f() + "=V";
fn probe() { return out; }
print(probe());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "U=7=V", "V3: triple concat, got: {}", r);
}

#[test]
fn l1_strcat_var_before() {
    let src = r#"
fn f() { return 7; }
let x = f();
let out = "U=" + x;
fn probe() { return out; }
print(probe());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "U=7", "V4: var first, got: {}", r);
}

#[test]
fn l1_strcat_fn_returns_string() {
    let src = r#"
fn g() { return "W"; }
let out = "U=" + g();
fn probe() { return out; }
print(probe());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "U=W", "V5: string-returning fn, got: {}", r);
}

#[test]
fn l1_strcat_fn_string_left() {
    let src = r#"
fn g() { return "W"; }
let out = g() + "=U";
fn probe() { return out; }
print(probe());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "W=U", "V6: string fn left, got: {}", r);
}

#[test]
fn l1_strcat_triple_string_fn() {
    let src = r#"
fn g() { return "W"; }
let out = "U=" + g() + "=V";
fn probe() { return out; }
print(probe());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "U=W=V", "V7: triple string fn, got: {}", r);
}
