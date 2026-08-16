//! T5.2 regression: ctor `this` field writeback (StoreGlobal→cur_this).

use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler
        .compile(&stmts)
        .map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

#[test]
fn t52_ctor_field_write_read() {
    // Simple ctor: this.v = v; read via .v
    let src = "class P { constructor(v) { this.v = v; } } let r = new P(41); return r.v;";
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "41");
}

#[test]
fn t52_super_chain_ctor() {
    // Super chain: parent ctor sets field, child reads + extends
    let src = "class P { constructor(v) { this.v = v; } }
class C extends P { constructor() { super(41); this.w = this.v + 1; } }
let c = new C(); return c.w;";
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "42");
}

#[test]
fn t52_method_this_write_read() {
    // Method this write: this.k = 9; read via .k
    let src = "class M { public fn s() { this.k = 9; return this.k; } }
let m = new M(); let a = m.s(); let b = m.k; return a + b;";
    let vm = run(src).unwrap();
    assert_eq!(vm.last_return_value().display_string(), "18");
}
