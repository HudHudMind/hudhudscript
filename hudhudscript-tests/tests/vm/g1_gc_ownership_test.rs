//! G1: GC ownership and root correctness regression tests.

use hudhudscript_vm::VM;

fn run_vm(src: &str) -> VM {
    let stmts = hudhudscript_parser::parse(src).expect("parse");
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm
}

#[test]
fn g1_two_vms_same_thread_isolation() {
    let mut vm_a = run_vm(r#"let s = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"; return s.length;"#);
    assert_eq!(vm_a.last_return_value().display_string(), "42");
    let mut vm_b = run_vm(r#"let arr = [1, 2, 3, 4, 5]; return arr.length;"#);
    assert_eq!(vm_b.last_return_value().display_string(), "5");
    let val_a = vm_a.last_return_value().display_string();
    assert_eq!(val_a, "42");
    let val_b = vm_b.last_return_value().display_string();
    assert_eq!(val_b, "5");
}

#[test]
fn g1_cur_this_survives_collection() {
    let mut vm = run_vm(r#"class Counter { fn constructor(start) { this.value = start; } fn bump() { this.value = this.value + 1; return this.value; } } let c = new Counter(41); c.bump(); return c.bump();"#);
    assert_eq!(vm.last_return_value().display_string(), "43");
}

#[test]
fn g1_retired_frame_values_freed_active_survive() {
    let mut vm = run_vm(r#"fn make_temp() { let temp = "collectible"; return 42; } let result = make_temp(); let persistent = "this_must_survive"; return persistent.length;"#);
    assert_eq!(vm.last_return_value().display_string(), "17");
}

#[test]
fn g1_closure_captured_value_survives() {
    let mut vm = run_vm(r#"fn make_closure() { let captured = "closure_data_here"; return fn() { return captured; }; } let f = make_closure(); return f();"#);
    assert_eq!(vm.last_return_value().display_string(), "closure_data_here");
}

#[test]
fn g1_nested_return_value_survives() {
    let mut vm = run_vm(r#"fn outer() { fn inner() { return "inner_data"; } return inner(); } return outer();"#);
    assert_eq!(vm.last_return_value().display_string(), "inner_data");
}

#[test]
fn g1_multiple_returns_preserve_values() {
    let mut vm = run_vm(r#"fn make_obj() { return {name: "test", value: 123}; } let obj = make_obj(); return obj.name;"#);
    assert_eq!(vm.last_return_value().display_string(), "test");
}

// ── G1.5: GC stress — recursion + closure in same VM ─────────────────
#[test]
fn g1_gc_stress_recursion_and_closure() {
    let mut vm = run_vm(
        r#"
fn fib(n) {
    if (n < 2) { return n; }
    return fib(n - 1) + fib(n - 2);
}
fn make_counter(start) {
    return fn() { return start; };
}
let r = fib(10);
let c = make_counter(r);
return c();
"#
    );
    assert_eq!(vm.last_return_value().display_string(), "55");
}

// ── G1.6: Exception + finally GC safety ──────────────────────────────
#[test]
fn g1_exception_finally_gc_safety() {
    let mut vm = run_vm(
        r#"
let tracker = "alive";
try {
    let temp = "throw_me";
    throw temp;
} catch (e) {
    tracker = tracker + "_caught";
} finally {
    tracker = tracker + "_finally";
}
return tracker;
"#
    );
    assert_eq!(vm.last_return_value().display_string(), "alive_caught_finally");
}

// ── G1.7: Nested VM/module export ────────────────────────────────────
#[test]
fn g1_nested_module_export_survives() {
    let mut vm = run_vm(
        r#"
let secret = "exported_data";
fn get_secret() { return secret; }
return get_secret();
"#
    );
    assert_eq!(vm.last_return_value().display_string(), "exported_data");
}

// ── G1.8: Pin/churn — many allocations then read back ────────────────
#[test]
fn g1_allocation_churn_values_survive() {
    let mut vm = run_vm(
        r#"
let arr = [];
let i = 0;
while (i < 1000) {
    arr.push("item_" + i);
    i = i + 1;
}
return arr[999];
"#
    );
    // "item_" + 999 = "item_999"
    assert_eq!(vm.last_return_value().display_string(), "item_999");
}

// ── G1.1.4: External/native value transfer ──────────────────────────
// Values created in one VM and passed to another must remain valid.
#[test]
fn g1_external_value_transfer_survives() {
    let mut vm1 = run_vm(r#"return "shared_across_vms";"#);
    let val1 = vm1.last_return_value().display_string();
    let mut vm2 = run_vm(r#"return "second_vm_value";"#);
    assert_eq!(vm2.last_return_value().display_string(), "second_vm_value");
    assert_eq!(val1, "shared_across_vms");
}
