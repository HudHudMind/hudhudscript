//! Integration tests for v0.4.38 Batch 3: Data Types
//!
//! Tests Set, Map, typeof/is, and spread operator in the interpreter.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(code: &str) -> Interpreter {
    let ast = parse(code).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.eval_program(&ast).expect("execution failed");
    interp
}

fn run_and_get(code: &str, var: &str) -> Value16 {
    let interp = run(code);
    interp
        .get_variable(var)
        .unwrap_or_else(|_| panic!("variable '{}' not found", var))
}

// ─── Set (#653) ─────────────────────────────────────────────────────────────

#[test]
fn test_set_new_empty() {
    let val = run_and_get("var s = Set.new();", "s");
    assert!(val.as_set().map(|items| items.is_empty()).unwrap_or(false));
}

#[test]
fn test_set_new_from_array() {
    let val = run_and_get("var s = Set.new([1, 2, 3, 2, 1]);", "s");
    if let Some(items) = val.as_set() {
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected Set, got {:?}", val);
    }
}

#[test]
fn test_set_add_and_has() {
    let code = r#"
        var s = Set.new();
        s = s.add(1);
        s = s.add(2);
        s = s.add(1);
        var has1 = s.has(1);
        var has3 = s.has(3);
        var size = s.size();
    "#;
    let interp = run(code);
    assert_eq!(interp.get_variable("has1").unwrap(), Value16::boolean(true));
    assert_eq!(
        interp.get_variable("has3").unwrap(),
        Value16::boolean(false)
    );
    assert_eq!(interp.get_variable("size").unwrap(), Value16::number(2.0));
}

#[test]
fn test_set_remove() {
    let code = r#"
        var s = Set.new([1, 2, 3]);
        s = s.remove(2);
        var has2 = s.has(2);
        var size = s.size();
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("has2").unwrap(),
        Value16::boolean(false)
    );
    assert_eq!(interp.get_variable("size").unwrap(), Value16::number(2.0));
}

#[test]
fn test_set_union() {
    let code = r#"
        var a = Set.new([1, 2, 3]);
        var b = Set.new([3, 4, 5]);
        var c = a.union(b);
        var size = c.size();
    "#;
    assert_eq!(run_and_get(code, "size"), Value16::number(5.0));
}

#[test]
fn test_set_intersection() {
    let code = r#"
        var a = Set.new([1, 2, 3]);
        var b = Set.new([2, 3, 4]);
        var c = a.intersection(b);
        var size = c.size();
    "#;
    assert_eq!(run_and_get(code, "size"), Value16::number(2.0));
}

#[test]
fn test_set_difference() {
    let code = r#"
        var a = Set.new([1, 2, 3]);
        var b = Set.new([2, 3, 4]);
        var c = a.difference(b);
        var size = c.size();
    "#;
    assert_eq!(run_and_get(code, "size"), Value16::number(1.0));
}

#[test]
fn test_set_to_array() {
    let code = r#"
        var s = Set.new([3, 1, 2]);
        var arr = s.toArray();
        var len = len(arr);
    "#;
    assert_eq!(run_and_get(code, "len"), Value16::number(3.0));
}

// ─── Map (#654) ─────────────────────────────────────────────────────────────

#[test]
fn test_map_new_empty() {
    let val = run_and_get("var m = Map.new();", "m");
    assert!(val
        .as_map_pairs()
        .map(|pairs| pairs.is_empty())
        .unwrap_or(false));
}

#[test]
fn test_map_set_and_get() {
    let code = r#"
        var m = Map.new();
        m = m.set("a", 1);
        m = m.set("b", 2);
        var a = m.get("a");
        var b = m.get("b");
        var c = m.get("c");
    "#;
    let interp = run(code);
    assert_eq!(interp.get_variable("a").unwrap(), Value16::number(1.0));
    assert_eq!(interp.get_variable("b").unwrap(), Value16::number(2.0));
    assert_eq!(interp.get_variable("c").unwrap(), Value16::null());
}

#[test]
fn test_map_has_and_delete() {
    let code = r#"
        var m = Map.new();
        m = m.set("x", 10);
        var has_x = m.has("x");
        m = m.delete("x");
        var has_x_after = m.has("x");
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("has_x").unwrap(),
        Value16::boolean(true)
    );
    assert_eq!(
        interp.get_variable("has_x_after").unwrap(),
        Value16::boolean(false)
    );
}

#[test]
fn test_map_keys_values_entries() {
    let code = r#"
        var m = Map.new();
        m = m.set("a", 1);
        m = m.set("b", 2);
        var ks = m.keys();
        var vs = m.values();
        var es = m.entries();
        var size = m.size();
    "#;
    let interp = run(code);
    assert_eq!(interp.get_variable("size").unwrap(), Value16::number(2.0));
    if let Some(keys) = interp.get_variable("ks").unwrap().as_array() {
        assert_eq!(keys.len(), 2);
    } else {
        panic!("Expected Array for keys");
    }
    if let Some(values) = interp.get_variable("vs").unwrap().as_array() {
        assert_eq!(values.len(), 2);
    } else {
        panic!("Expected Array for values");
    }
    if let Some(entries) = interp.get_variable("es").unwrap().as_array() {
        assert_eq!(entries.len(), 2);
    } else {
        panic!("Expected Array for entries");
    }
}

#[test]
fn test_map_overwrite_key() {
    let code = r#"
        var m = Map.new();
        m = m.set("x", 1);
        m = m.set("x", 99);
        var v = m.get("x");
        var size = m.size();
    "#;
    let interp = run(code);
    assert_eq!(interp.get_variable("v").unwrap(), Value16::number(99.0));
    assert_eq!(interp.get_variable("size").unwrap(), Value16::number(1.0));
}

// ─── typeof / is (#671) ─────────────────────────────────────────────────────

#[test]
fn test_typeof_primitives() {
    let code = r#"
        var t1 = typeof(42);
        var t2 = typeof("hello");
        var t3 = typeof(true);
        var t4 = typeof(null);
        var t5 = typeof([1, 2]);
        var t6 = typeof({x: 1});
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("t1").unwrap(),
        Value16::string("number".to_string())
    );
    assert_eq!(
        interp.get_variable("t2").unwrap(),
        Value16::string("string".to_string())
    );
    assert_eq!(
        interp.get_variable("t3").unwrap(),
        Value16::string("boolean".to_string())
    );
    assert_eq!(
        interp.get_variable("t4").unwrap(),
        Value16::string("null".to_string())
    );
    assert_eq!(
        interp.get_variable("t5").unwrap(),
        Value16::string("array".to_string())
    );
    assert_eq!(
        interp.get_variable("t6").unwrap(),
        Value16::string("object".to_string())
    );
}

#[test]
fn test_typeof_set_map() {
    let code = r#"
        var s = Set.new();
        var m = Map.new();
        var ts = typeof(s);
        var tm = typeof(m);
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("ts").unwrap(),
        Value16::string("set".to_string())
    );
    assert_eq!(
        interp.get_variable("tm").unwrap(),
        Value16::string("map".to_string())
    );
}

#[test]
fn test_is_function() {
    let code = r#"
        var r1 = is(42, "Number");
        var r2 = is("hello", "String");
        var r3 = is(true, "Boolean");
        var r4 = is(null, "Null");
        var r5 = is([1], "Array");
        var r6 = is(42, "String");
    "#;
    let interp = run(code);
    assert_eq!(interp.get_variable("r1").unwrap(), Value16::boolean(true));
    assert_eq!(interp.get_variable("r2").unwrap(), Value16::boolean(true));
    assert_eq!(interp.get_variable("r3").unwrap(), Value16::boolean(true));
    assert_eq!(interp.get_variable("r4").unwrap(), Value16::boolean(true));
    assert_eq!(interp.get_variable("r5").unwrap(), Value16::boolean(true));
    assert_eq!(interp.get_variable("r6").unwrap(), Value16::boolean(false));
}

// ─── Spread (#672) ──────────────────────────────────────────────────────────

#[test]
fn test_array_spread() {
    let code = r#"
        var a = [1, 2];
        var b = [0, ...a, 3];
        var len = len(b);
    "#;
    let interp = run(code);
    assert_eq!(interp.get_variable("len").unwrap(), Value16::number(4.0));
    if let Some(arr) = interp.get_variable("b").unwrap().as_array() {
        assert_eq!(arr[0], Value16::number(0.0));
        assert_eq!(arr[1], Value16::number(1.0));
        assert_eq!(arr[2], Value16::number(2.0));
        assert_eq!(arr[3], Value16::number(3.0));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_object_spread() {
    // Test object spread copies all properties
    let code = r#"
        var a = {x: 1, y: 2};
        var b = {...a};
    "#;
    let interp = run(code);
    let b = interp.get_variable("b").unwrap();
    if let Some(map) = b.as_object() {
        assert_eq!(map.get("x"), Some(&Value16::number(1.0)));
        assert_eq!(map.get("y"), Some(&Value16::number(2.0)));
    } else {
        panic!("Expected Object, got {:?}", b);
    }
}

#[test]
fn test_object_spread_merge() {
    // Spread two objects together
    let code = r#"
        var a = {x: 1, y: 2};
        var b = {z: 3, w: 4};
        var c = {...a};
        var d = {...b};
        var x_val = c.x;
        var z_val = d.z;
    "#;
    let interp = run(code);
    assert_eq!(interp.get_variable("x_val").unwrap(), Value16::number(1.0));
    assert_eq!(interp.get_variable("z_val").unwrap(), Value16::number(3.0));
}

#[test]
fn test_rest_params() {
    let code = r#"
        function sum(first, ...rest) {
            var total = first;
            for (x in rest) {
                total = total + x;
            }
            return total;
        }
        var result = sum(1, 2, 3, 4);
    "#;
    assert_eq!(run_and_get(code, "result"), Value16::number(10.0));
}

#[test]
fn test_spread_in_call() {
    let code = r#"
        function add(a, b, c) {
            return a + b + c;
        }
        var args = [10, 20, 30];
        var result = add(...args);
    "#;
    assert_eq!(run_and_get(code, "result"), Value16::number(60.0));
}
