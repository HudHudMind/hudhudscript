//! F2: GC döngü toplama testi (REFSEM sonrası)
use hudhudscript_vm::VM;
fn run(src: &str) -> Result<String, String> {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc).map(|_| vm.last_return_value().display_string()).map_err(|e| format!("{}", e))
}
#[test] fn gc_cycle_object_self_ref() {
    let r = run(r#"let o = {}; o.self = o; fn probe() { return "ok"; } print(probe());"#).unwrap();
    assert_eq!(r, "ok", "object self-ref cycle");
}
#[test] fn gc_cycle_array_self_ref() {
    let r = run(r#"let a = []; a.push(a); fn probe() { return "ok"; } print(probe());"#).unwrap();
    assert_eq!(r, "ok", "array self-ref cycle");
}
#[test] fn gc_cycle_mutual() {
    let r = run(r#"let a = {}; let b = {}; a.b = b; b.a = a; fn probe() { return "ok"; } print(probe());"#).unwrap();
    assert_eq!(r, "ok", "mutual object cycle");
}
