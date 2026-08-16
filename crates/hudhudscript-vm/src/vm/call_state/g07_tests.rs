//! G07 acceptance tests: this/class context restore across deferred calls,
//! deferred throw/finally routing, and GC survival of pending call state.

use crate::vm::VM;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn compile_source(source: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = parse(source).expect("test source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("test source must compile")
}

fn run_source(source: &str) -> VM {
    let bytecode = compile_source(source);
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("test source must execute");
    vm
}

fn variable(vm: &VM, name: &str) -> hudhudscript_bytecode::Value16 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name))
}

/// Forced GC mid-execution: with a 1-object threshold the safepoint in the
/// dispatch loop collects while requests are pending and between callback
/// resumes — not only at test end.
fn run_source_with_forced_gc(source: &str) -> VM {
    hudhudscript_bytecode::gc::set_gc_tuning(1, 2);
    let bytecode = compile_source(source);
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("test source must execute");
    hudhudscript_bytecode::gc::set_gc_tuning(
        hudhudscript_bytecode::gc::DEFAULT_GC_MIN_OBJECTS,
        hudhudscript_bytecode::gc::DEFAULT_GC_GROWTH,
    );
    vm
}

#[test]
fn deferred_method_restores_previous_this() {
    let vm = run_source(
        r#"
class A { fn who() { return "A" } }
class B {
    fn who() { return "B" }
    fn both() {
        let x = (new A()).who()
        return x + "/" + this.who()
    }
}
let r = (new B()).both()
"#,
    );
    assert_eq!(variable(&vm, "r").as_string(), Some("A/B".to_string()));
}

#[test]
fn deferred_method_writes_mutated_receiver_back() {
    let vm = run_source(
        r#"
class C { fn bump() { this.n = this.n + 1; return this.n } }
let c = new C()
c.n = 5
let r = c.bump()
let after = c.n
"#,
    );
    assert_eq!(variable(&vm, "r").as_number(), Some(6.0));
    assert_eq!(variable(&vm, "after").as_number(), Some(6.0));
}

#[test]
fn nested_class_method_restores_class_context() {
    let vm = run_source(
        r#"
class M { fn go() { return 1 } }
class N {
    private fn hidden() { return 1 }
    fn outer() {
        let m = (new M()).go()
        return this.hidden() + m
    }
}
let r = (new N()).outer()
"#,
    );
    assert_eq!(variable(&vm, "r").as_number(), Some(2.0));
}

#[test]
fn deferred_throw_reaches_caller_catch() {
    let vm = run_source(
        r#"
class T { fn boom() { throw "inner-boom" } }
let r = "none"
try {
    let x = (new T()).boom()
} catch (e) {
    r = "caught:" + e.description
}
"#,
    );
    assert_eq!(
        variable(&vm, "r").as_string(),
        Some("caught:inner-boom".to_string())
    );
}

#[test]
fn deferred_finally_runs_once() {
    let vm = run_source(
        r#"
class T { fn boom() { throw "x" } }
let count = 0
try {
    let x = (new T()).boom()
} catch (e) {
    count = count + 10
} finally {
    count = count + 1
}
let r = count
"#,
    );
    assert_eq!(variable(&vm, "r").as_number(), Some(11.0));
}

#[test]
fn pending_call_args_survive_forced_gc() {
    let vm = run_source_with_forced_gc(
        r#"
class P { fn echo(s) { return s } }
let payload = "gc-pending-arg-dynamic-value"
let out = []
let i = 0
while (i < 200) {
    out.push((new P()).echo(payload))
    i = i + 1
}
let first = out[0]
let last = out[199]
"#,
    );
    assert_eq!(
        variable(&vm, "first").as_string(),
        Some("gc-pending-arg-dynamic-value".to_string())
    );
    assert_eq!(
        variable(&vm, "last").as_string(),
        Some("gc-pending-arg-dynamic-value".to_string())
    );
}

#[test]
fn pending_receiver_survives_forced_gc() {
    let vm = run_source_with_forced_gc(
        r#"
class R { fn tag() { return this.name } }
let out = []
let i = 0
while (i < 200) {
    let r = new R()
    r.name = "receiver-" + i
    out.push(r.tag())
    i = i + 1
}
let first = out[0]
let last = out[199]
"#,
    );
    assert_eq!(
        variable(&vm, "first").as_string(),
        Some("receiver-0".to_string())
    );
    assert_eq!(
        variable(&vm, "last").as_string(),
        Some("receiver-199".to_string())
    );
}

#[test]
fn continuation_accumulator_survives_forced_gc() {
    let mut items = String::new();
    for value in 0..200 {
        if !items.is_empty() {
            items.push_str(", ");
        }
        items.push_str(&value.to_string());
    }
    let source = format!(
        r#"
let values = [{items}]
let total = values.reduce((acc, item, index) => {{ return acc + item }}, 0)
"#
    );
    let vm = run_source_with_forced_gc(&source);
    assert_eq!(variable(&vm, "total").as_number(), Some(19900.0));
}

#[test]
fn deferred_capture_cell_survives_forced_gc() {
    let vm = run_source_with_forced_gc(
        r#"
let captured = "cap-dynamic-value-xyz"
let f = (x) => { return captured + "-" + x }
let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let mapped = values.map(f)
let last = mapped[9]
"#,
    );
    assert_eq!(
        variable(&vm, "last").as_string(),
        Some("cap-dynamic-value-xyz-10".to_string())
    );
}
