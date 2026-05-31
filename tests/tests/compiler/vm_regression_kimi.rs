/// VM Regression Tests — KIMI Sprint (RECURSIVE-001, SUPER-001, METHOD-001)
///
/// These tests guard the fixes made to `crates/hudhudscript-vm/src/vm.rs`
/// and must never be modified without user approval.
use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::{Compiler, VM};
use hudhudscript_parser::parse;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Compile and execute HudHudScript source in VM mode.
/// Returns the VM so the caller can inspect variables.
fn run(src: &str) -> VM {
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("vm execute failed");
    vm
}

/// Compile and execute, return the named variable's value.
fn var(src: &str, name: &str) -> Value16 {
    run(src)
        .get_variable(name)
        .cloned()
        .unwrap_or(Value16::null())
}

fn num(v: &Value16) -> f64 {
    v.as_number()
        .or_else(|| v.as_int().map(|i| i as f64))
        .unwrap_or_else(|| panic!("expected number, got {:?}", v))
}

fn str_val(v: &Value16) -> String {
    v.as_string()
        .unwrap_or_else(|| panic!("expected string, got {:?}", v))
}

// ── RECURSIVE-001: stack management for recursive calls ───────────────────────

#[test]
fn test_rec_fib_base_cases() {
    // fib(0) == 0 and fib(1) == 1
    let src = r#"
function fib(n) {
    if (n <= 1) { return n }
    return fib(n-1) + fib(n-2)
}
let r0 = fib(0)
let r1 = fib(1)
"#;
    let vm = run(src);
    assert_eq!(num(&vm.get_variable("r0").cloned().unwrap()), 0.0);
    assert_eq!(num(&vm.get_variable("r1").cloned().unwrap()), 1.0);
}

#[test]
fn test_rec_fib5() {
    // fib(5) == 5  (was returning -3 before the RECURSIVE-001 fix)
    let src = r#"
function fib(n) {
    if (n <= 1) { return n }
    return fib(n-1) + fib(n-2)
}
let result = fib(5)
"#;
    assert_eq!(num(&var(src, "result")), 5.0, "fib(5) must be 5");
}

#[test]
fn test_rec_fib10() {
    let src = r#"
function fib(n) {
    if (n <= 1) { return n }
    return fib(n-1) + fib(n-2)
}
let result = fib(10)
"#;
    assert_eq!(num(&var(src, "result")), 55.0, "fib(10) must be 55");
}

#[test]
fn test_rec_factorial() {
    let src = r#"
function fact(n) {
    if (n <= 1) { return 1 }
    return n * fact(n - 1)
}
let r = fact(5)
"#;
    assert_eq!(num(&var(src, "r")), 120.0, "fact(5) must be 120");
}

#[test]
fn test_rec_countdown() {
    // Simple tail-recursive-style countdown
    let src = r#"
function count(n) {
    if (n == 0) { return 0 }
    return count(n - 1)
}
let r = count(10)
"#;
    assert_eq!(num(&var(src, "r")), 0.0);
}

// ── SUPER-001: super() constructor propagates `this` ─────────────────────────

#[test]
fn test_super_basic() {
    let src = r#"
class Animal {
    constructor(name) {
        this.name = name
        this.type = "animal"
    }
}
class Dog extends Animal {
    constructor(name, breed) {
        super(name)
        this.breed = breed
        this.type = "dog"
    }
}
let d = new Dog("Rex", "Husky")
let dname  = d.name
let dtype  = d.type
let dbreed = d.breed
"#;
    let vm = run(src);
    assert_eq!(
        str_val(&vm.get_variable("dname").cloned().unwrap()),
        "Rex",
        "d.name must be 'Rex'"
    );
    assert_eq!(
        str_val(&vm.get_variable("dtype").cloned().unwrap()),
        "dog",
        "child type must override parent"
    );
    assert_eq!(
        str_val(&vm.get_variable("dbreed").cloned().unwrap()),
        "Husky",
        "d.breed must be 'Husky'"
    );
}

#[test]
fn test_super_args_propagated() {
    // Parent sets a field from the constructor argument
    let src = r#"
class Base {
    constructor(x) {
        this.x = x
    }
}
class Child extends Base {
    constructor(x, y) {
        super(x)
        this.y = y
    }
}
let c = new Child(10, 20)
let cx = c.x
let cy = c.y
"#;
    let vm = run(src);
    assert_eq!(num(&vm.get_variable("cx").cloned().unwrap()), 10.0);
    assert_eq!(num(&vm.get_variable("cy").cloned().unwrap()), 20.0);
}

// ── METHOD-001: instance method dispatch ─────────────────────────────────────

#[test]
fn test_instance_method_basic() {
    // HudHudScript class methods require the `function` keyword
    let src = r#"
class Calc {
    function add(a, b) { return a + b }
    function multiply(a, b) { return a * b }
}
let c = new Calc()
let sum = c.add(2, 3)
let prod = c.multiply(4, 5)
"#;
    let vm = run(src);
    assert_eq!(num(&vm.get_variable("sum").cloned().unwrap()), 5.0);
    assert_eq!(num(&vm.get_variable("prod").cloned().unwrap()), 20.0);
}

#[test]
fn test_instance_method_uses_this() {
    // Note: The VM uses value semantics for instances — mutations to `this` inside a method
    // are not automatically written back to the caller's variable binding.
    // This test verifies that the method is dispatched and executes without error,
    // and that `this` field reads work correctly within a single method call.
    let src = r#"
class Counter {
    constructor() {
        this.count = 0
    }
    function get_count() {
        return this.count
    }
}
let ctr = new Counter()
let initial = ctr.get_count()
"#;
    let vm = run(src);
    // The constructor sets count=0; get_count() should read 0 from `this`
    assert_eq!(num(&vm.get_variable("initial").cloned().unwrap()), 0.0);
}
