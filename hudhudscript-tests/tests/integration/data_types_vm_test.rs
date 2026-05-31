//! VM parity tests for v0.4.38 Batch 3: Data Types

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::{Compiler, VM};
use hudhudscript_parser::parse;

fn vm_run_and_get(code: &str, var: &str) -> Value16 {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("VM execution failed");
    vm.get_variable(var)
        .cloned()
        .map(|v| v)
        .unwrap_or_else(|| panic!("variable '{}' not found", var))
}

fn assert_number(val: Value16, expected: f64) {
    if let Some(n) = val.as_number() {
        assert!(
            (n - expected).abs() < f64::EPSILON,
            "Expected {}, got {}",
            expected,
            n
        );
    } else {
        panic!("Expected Number({}), got {:?}", expected, val);
    }
}

fn assert_bool(val: Value16, expected: bool) {
    if let Some(b) = val.as_bool() {
        assert_eq!(b, expected, "Expected {}, got {}", expected, b);
    } else {
        panic!("Expected Boolean({}), got {:?}", expected, val);
    }
}

fn assert_string(val: Value16, expected: &str) {
    if let Some(s) = val.as_str() {
        assert_eq!(s, expected, "Expected '{}', got '{}'", expected, s);
    } else {
        panic!("Expected String(\"{}\"), got {:?}", expected, val);
    }
}

// ─── Set (#653) ─────────────────────────────────────────────────────────────

#[test]
fn test_vm_set_new_empty() {
    let val = vm_run_and_get("var s = Set.new();", "s");
    assert!(val.as_set().map(|items| items.is_empty()).unwrap_or(false));
}

#[test]
fn test_vm_set_new_from_array() {
    let val = vm_run_and_get("var s = Set.new([1, 2, 3, 2, 1]);", "s");
    if let Some(items) = val.as_set() {
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected Set, got {:?}", val);
    }
}

#[test]
fn test_vm_set_add_has() {
    let code = r#"
        var s = Set.new();
        s = s.add(1);
        s = s.add(2);
        s = s.add(1);
        var has1 = s.has(1);
        var has3 = s.has(3);
        var size = s.size();
    "#;
    assert_bool(vm_run_and_get(code, "has1"), true);
    assert_bool(vm_run_and_get(code, "has3"), false);
    assert_number(vm_run_and_get(code, "size"), 2.0);
}

#[test]
fn test_vm_set_remove() {
    let code = r#"
        var s = Set.new([1, 2, 3]);
        s = s.remove(2);
        var has2 = s.has(2);
        var size = s.size();
    "#;
    assert_bool(vm_run_and_get(code, "has2"), false);
    assert_number(vm_run_and_get(code, "size"), 2.0);
}

#[test]
fn test_vm_set_union() {
    let code = r#"
        var a = Set.new([1, 2, 3]);
        var b = Set.new([3, 4, 5]);
        var c = a.union(b);
        var size = c.size();
    "#;
    assert_number(vm_run_and_get(code, "size"), 5.0);
}

#[test]
fn test_vm_set_intersection() {
    let code = r#"
        var a = Set.new([1, 2, 3]);
        var b = Set.new([2, 3, 4]);
        var c = a.intersection(b);
        var size = c.size();
    "#;
    assert_number(vm_run_and_get(code, "size"), 2.0);
}

#[test]
fn test_vm_set_difference() {
    let code = r#"
        var a = Set.new([1, 2, 3]);
        var b = Set.new([2, 3, 4]);
        var c = a.difference(b);
        var size = c.size();
    "#;
    assert_number(vm_run_and_get(code, "size"), 1.0);
}

// ─── Map (#654) ─────────────────────────────────────────────────────────────

#[test]
fn test_vm_map_new_empty() {
    let val = vm_run_and_get("var m = Map.new();", "m");
    assert!(val.as_map_pairs().map(|p| p.is_empty()).unwrap_or(false));
}

#[test]
fn test_vm_map_set_get() {
    let code = r#"
        var m = Map.new();
        m = m.set("a", 1);
        m = m.set("b", 2);
        var a = m.get("a");
        var b = m.get("b");
        var c = m.get("c");
    "#;
    assert_number(vm_run_and_get(code, "a"), 1.0);
    assert_number(vm_run_and_get(code, "b"), 2.0);
    assert!(vm_run_and_get(code, "c").is_null());
}

#[test]
fn test_vm_map_has_delete() {
    let code = r#"
        var m = Map.new();
        m = m.set("x", 10);
        var has_x = m.has("x");
        m = m.delete("x");
        var has_x_after = m.has("x");
    "#;
    assert_bool(vm_run_and_get(code, "has_x"), true);
    assert_bool(vm_run_and_get(code, "has_x_after"), false);
}

#[test]
fn test_vm_map_keys_values_entries() {
    let code = r#"
        var m = Map.new();
        m = m.set("a", 1);
        m = m.set("b", 2);
        var ks = m.keys();
        var vs = m.values();
        var size = m.size();
    "#;
    assert_number(vm_run_and_get(code, "size"), 2.0);
    if let Some(keys) = vm_run_and_get(code, "ks").as_array() {
        assert_eq!(keys.len(), 2);
    } else {
        panic!("Expected Array");
    }
}

// ─── typeof / is (#671) ─────────────────────────────────────────────────────

#[test]
fn test_vm_typeof() {
    assert_string(vm_run_and_get(r#"var x = typeof(42);"#, "x"), "number");
    assert_string(vm_run_and_get(r#"var x = typeof("hello");"#, "x"), "string");
    assert_string(vm_run_and_get(r#"var x = typeof(true);"#, "x"), "boolean");
    assert_string(vm_run_and_get(r#"var x = typeof(null);"#, "x"), "null");
}

#[test]
fn test_vm_typeof_set_map() {
    assert_string(
        vm_run_and_get("var s = Set.new(); var x = typeof(s);", "x"),
        "set",
    );
    assert_string(
        vm_run_and_get("var m = Map.new(); var x = typeof(m);", "x"),
        "map",
    );
}

#[test]
fn test_vm_is_function() {
    assert_bool(vm_run_and_get(r#"var x = is(42, "Number");"#, "x"), true);
    assert_bool(vm_run_and_get(r#"var x = is("hi", "String");"#, "x"), true);
    assert_bool(vm_run_and_get(r#"var x = is(42, "String");"#, "x"), false);
    assert_bool(vm_run_and_get(r#"var x = is(true, "Boolean");"#, "x"), true);
}
