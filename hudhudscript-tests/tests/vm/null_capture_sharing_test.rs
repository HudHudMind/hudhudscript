//! G5-slotvec: null-capture paylaşım regresyon testi (3 senaryo)
//! v0.8.152'de Option<Arc> sentinel yerine is_null() kullanıldığı için
//! null-başlangıçlı capture'lar iki closure arasında paylaşılamıyordu.

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
fn null_capture_shared_between_two_closures() {
    // Senaryo 1: null ile başlayan capture, iki closure arasında paylaşılır
    let src = r#"
fn make() {
    let x = null;
    let putx = () => { x = 5; };
    let getx = () => { return x; };
    putx();
    return getx();
}
print(make());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(
        r, "5",
        "null capture should be shared between closures, got: {}",
        r
    );
}

#[test]
fn null_capture_assigned_and_read() {
    // Senaryo 2: null capture'a değer atanır ve okunur
    let src = r#"
fn test() {
    let val = null;
    let put_val = fn(v) { val = v; };
    let get = fn() { return val; };
    put_val(42);
    return get();
}
print(test());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "42", "null capture assign+read, got: {}", r);
}

#[test]
fn null_capture_null_to_value_to_null_transition() {
    // Senaryo 3: null → değer → null geçişi (tek tek eleman kontrolü)
    let src = r#"
fn test() {
    let val = null;
    let put_val = fn(v) { val = v; };
    let get = fn() { return val; };
    let r1 = get();
    if (r1 != null) { return "FAIL: r1"; }
    put_val(99);
    let r2 = get();
    if (r2 != 99) { return "FAIL: r2"; }
    put_val(null);
    let r3 = get();
    if (r3 != null) { return "FAIL: r3"; }
    return "OK";
}
print(test());
"#;
    let r = run(src).unwrap_or_else(|e| e);
    assert_eq!(r, "OK", "null→value→null transition, got: {}", r);
}
