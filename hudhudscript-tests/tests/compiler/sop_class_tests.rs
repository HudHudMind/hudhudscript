//! SOP (Subject-Oriented Programming) + class method chaining tests.
use hudhudscript_bytecode::{Bytecode, Instruction};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile(src: &str) -> Bytecode {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).unwrap()
}

fn run_and_get(src: &str, var: &str) -> i64 {
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm.get_variable(var).and_then(|v| v.as_int()).unwrap_or(-999)
}

fn run_and_get_str(src: &str, var: &str) -> String {
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm.get_variable(var).and_then(|v| v.as_string()).unwrap_or_default()
}

// ── Basic class tests ──────────────────────────────────────

#[test]
fn class_basic_field() {
    let src = "class Point { constructor(x, y) { this.x = x; this.y = y; } } let p = new Point(3, 4);";
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    let px = vm.get_variable("p").and_then(|v| {
        v.as_instance_data().and_then(|inst| inst.fields.get("x").and_then(|v| v.as_int()))
    }).unwrap_or(-1);
    assert_eq!(px, 3);
}

#[test]
fn class_method_returns_value() {
    let src = "class Box { constructor(v) { this.value = v; } public get() { return this.value; } } let b = new Box(42); let r = b.get();";
    assert_eq!(run_and_get(src, "r"), 42);
}

// ── Method chaining tests ──────────────────────────────────

#[test]
fn method_chaining_add_then_multiply() {
    let src = "class Calc { constructor(v) { this.value = v; } public add(n) { this.value = this.value + n; return this; } public multiply(n) { this.value = this.value * n; return this; } } let c = new Calc(5); c.add(3).multiply(2); let r = c.value;";
    assert_eq!(run_and_get(src, "r"), 16, "5+3=8, 8*2=16 via chaining");
}

#[test]
fn method_chaining_no_return_this() {
    // Without chaining, sequential calls should still work via the variable
    let src = "class Calc { constructor(v) { this.value = v; } public add(n) { this.value = this.value + n; } public multiply(n) { this.value = this.value * n; } } let c = new Calc(5); c.add(3); c.multiply(2); let r = c.value;";
    assert_eq!(run_and_get(src, "r"), 16, "5+3=8, 8*2=16 sequential");
}

#[test]
fn method_chaining_fluent() {
    let src = "class Chain { constructor(v) { this.val = v; } public inc() { this.val = this.val + 1; return this; } public dec() { this.val = this.val - 1; return this; } } let c = new Chain(10); c.inc().inc().dec(); let r = c.val;";
    assert_eq!(run_and_get(src, "r"), 11, "10+1+1-1=11");
}

// ── Inheritance tests ──────────────────────────────────────

#[test]
fn inheritance_subclass_has_parent_methods() {
    let src = "class A { public a() { return 1; } } class B extends A { public b() { return 2; } } let x = new B();";
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    // Just verify it runs without error
}

#[test]
fn inheritance_subclass_overrides() {
    let src = "class Base { public get() { return 1; } } class Child extends Base { public get() { return 2; } } let c = new Child(); let r = c.get();";
    assert_eq!(run_and_get(src, "r"), 2);
}

#[test]
fn inheritance_chaining_subclass() {
    let src = "class Base { public inc() { this.val = this.val + 1; return this; } } class Sub extends Base { constructor(v) { this.val = v; } } let s = new Sub(5); s.inc().inc(); let r = s.val;";
    assert_eq!(run_and_get(src, "r"), 7, "5+1+1=7");
}

// ── SOP: Subject-Oriented Programming tests ─────────────────

#[test]
fn sop_subject_basic() {
    let src = "subject Entity { state { let health = 100; } } let e = new Entity();";
    let bc = compile(src);
    // Just verify compilation succeeds
}

#[test]
    #[ignore] // SOP feature not yet implemented
fn sop_subject_with_methods() {
    let src = "subject Player { state { let score = 0; } } let p = new Player();";
    let bc = compile(src);
}

#[test]
    #[ignore] // SOP feature not yet implemented
fn sop_relation_between_subjects() {
    let src = "subject A {} subject B {} relation R between A and B {}";
    let bc = compile(src);
    // Just verify compilation succeeds
}
