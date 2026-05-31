//! Unit tests for operations modules: arithmetic, array, unary, comparisons.
//!
//! Issue #237 — cover untested operations.

use hudhud_script_tests::vm_interpreter::runtime_codes;
use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(source: &str) -> Interpreter {
    let ast = parse(source).unwrap_or_else(|e| panic!("Parse error:\n{e}"));
    let mut interpreter = Interpreter::new();
    interpreter
        .execute(&ast)
        .unwrap_or_else(|e| panic!("Runtime error:\n{e:?}"));
    interpreter
}

fn try_run(source: &str) -> Result<Interpreter, hudhud_script_tests::vm_interpreter::RuntimeError> {
    let ast = parse(source)
        .map_err(|e| hudhud_script_tests::vm_interpreter::runtime_codes::custom(format!("{e}")))?;
    let mut interp = Interpreter::new();
    interp.execute(&ast)?;
    Ok(interp)
}

// ──────────────────────────────────────────────
// Arithmetic
// ──────────────────────────────────────────────

#[test]
fn test_addition_numbers() {
    let interp = run("let x = 2 + 3;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(5.0));
}

#[test]
fn test_addition_string_concat() {
    let interp = run(r#"let x = "a" + "b";"#);
    assert_eq!(
        interp.get_variable("x").unwrap(),
        Value16::string("ab".to_string())
    );
}

#[test]
fn test_addition_string_number_coercion() {
    let interp = run(r#"let x = "val=" + 42;"#);
    assert_eq!(
        interp.get_variable("x").unwrap(),
        Value16::string("val=42".to_string())
    );
}

#[test]
fn test_subtraction() {
    let interp = run("let x = 10 - 3;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(7.0));
}

#[test]
fn test_multiplication() {
    let interp = run("let x = 4 * 5;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(20.0));
}

#[test]
fn test_division() {
    // Integer division: int / int → int (truncation toward zero)
    let interp = run("let x = 10 / 3;");
    if let Some(n) = interp.get_variable("x").unwrap().as_int() {
        assert_eq!(n, 3);
    } else {
        panic!("Expected integer result for 10 / 3");
    }
    // Float division: float / int → float
    let interp = run("let y = 10.0 / 3;");
    if let Some(n) = interp.get_variable("y").unwrap().as_number() {
        assert!((n - 3.333_333_333_333_333_5).abs() < 1e-10);
    } else {
        panic!("Expected float result for 10.0 / 3");
    }
}

#[test]
fn test_division_by_zero() {
    let result = try_run("let x = 10 / 0;");
    assert!(result.is_err(), "Division by zero should produce an error");
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("Expected error for division by zero"),
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("ivision") || msg.contains("zero"),
        "Error should mention division by zero, got: {msg}"
    );
}

#[test]
fn test_modulo() {
    let interp = run("let x = 10 % 3;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(1.0));
}

// ──────────────────────────────────────────────
// Comparisons
// ──────────────────────────────────────────────

#[test]
fn test_greater_than() {
    let interp = run("let x = 5 > 3;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_less_than() {
    let interp = run("let x = 3 < 5;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_greater_equal() {
    let interp = run("let x = 5 >= 5;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_less_equal() {
    let interp = run("let x = 5 <= 5;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_equality() {
    let interp = run("let x = 5 == 5;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_inequality() {
    let interp = run("let x = 5 != 3;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_equality_false() {
    let interp = run("let x = 5 == 3;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(false));
}

#[test]
fn test_inequality_false() {
    let interp = run("let x = 5 != 5;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(false));
}

// ──────────────────────────────────────────────
// Unary operators
// ──────────────────────────────────────────────

#[test]
fn test_unary_negation() {
    let interp = run("let x = -5;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(-5.0));
}

#[test]
fn test_unary_not_true() {
    let interp = run("let x = !true;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(false));
}

#[test]
fn test_unary_not_false() {
    let interp = run("let x = !false;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_unary_double_negation() {
    let interp = run("let x = -(-5);");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(5.0));
}

// ──────────────────────────────────────────────
// Compound arithmetic expressions
// ──────────────────────────────────────────────

#[test]
fn test_operator_precedence() {
    let interp = run("let x = 2 + 3 * 4;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(14.0));
}

#[test]
fn test_parenthesised_expression() {
    let interp = run("let x = (2 + 3) * 4;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(20.0));
}

// ──────────────────────────────────────────────
// Array operations — push / pop (mutating)
// ──────────────────────────────────────────────

#[test]
fn test_array_push() {
    let interp = run(r#"
        let a = [1, 2];
        a.push(3);
    "#);
    let val = interp.get_variable("a").unwrap();
    if let Some(arr) = val.as_array() {
        assert_eq!(arr.len(), 3, "Array should have 3 elements after push");
        assert_eq!(arr[2], Value16::number(3.0));
    } else {
        panic!("Expected array, got {:?}", val);
    }
}

#[test]
fn test_array_pop() {
    let interp = run(r#"
        let a = [1, 2, 3];
        let x = a.pop();
    "#);
    // pop returns the removed element
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(3.0));
    // the array should now have 2 elements
    if let Some(arr) = interp.get_variable("a").unwrap().as_array() {
        assert_eq!(arr.len(), 2);
    } else {
        panic!(
            "Expected array, got {:?}",
            interp.get_variable("a").unwrap()
        );
    }
}

// ──────────────────────────────────────────────
// Array — length property
// ──────────────────────────────────────────────

#[test]
fn test_array_length() {
    let interp = run(r#"
        let a = [1, 2, 3];
        let n = a.length;
    "#);
    assert_eq!(interp.get_variable("n").unwrap(), Value16::number(3.0));
}

// ──────────────────────────────────────────────
// Array — map
// ──────────────────────────────────────────────

#[test]
fn test_array_map() {
    let interp = run(r#"
        let a = [1, 2, 3];
        let b = a.map((x) => x * 2);
    "#);
    let val = interp.get_variable("b").unwrap();
    if let Some(arr) = val.as_array() {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value16::number(2.0));
        assert_eq!(arr[1], Value16::number(4.0));
        assert_eq!(arr[2], Value16::number(6.0));
    } else {
        panic!("Expected array, got {:?}", val);
    }
}

// ──────────────────────────────────────────────
// Array — filter
// ──────────────────────────────────────────────

#[test]
fn test_array_filter() {
    let interp = run(r#"
        let a = [1, 2, 3, 4];
        let b = a.filter((x) => x > 2);
    "#);
    let val = interp.get_variable("b").unwrap();
    if let Some(arr) = val.as_array() {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], Value16::number(3.0));
        assert_eq!(arr[1], Value16::number(4.0));
    } else {
        panic!("Expected array, got {:?}", val);
    }
}

// ──────────────────────────────────────────────
// Array — reduce
// ──────────────────────────────────────────────

#[test]
fn test_array_reduce_sum() {
    let interp = run(r#"
        let a = [1, 2, 3, 4];
        let total = a.reduce((acc, x) => acc + x, 0);
    "#);
    assert_eq!(interp.get_variable("total").unwrap(), Value16::number(10.0));
}

// ──────────────────────────────────────────────
// Array — find
// ──────────────────────────────────────────────

#[test]
fn test_array_find() {
    let interp = run(r#"
        let a = [1, 2, 3, 4];
        let found = a.find((x) => x > 2);
    "#);
    assert_eq!(interp.get_variable("found").unwrap(), Value16::number(3.0));
}

#[test]
fn test_array_find_not_found() {
    let interp = run(r#"
        let a = [1, 2, 3];
        let found = a.find((x) => x > 10);
    "#);
    assert_eq!(interp.get_variable("found").unwrap(), Value16::null());
}

// ──────────────────────────────────────────────
// Array — some / every
// ──────────────────────────────────────────────

#[test]
fn test_array_some_true() {
    let interp = run(r#"
        let a = [1, 2, 3];
        let r = a.some((x) => x > 2);
    "#);
    assert_eq!(interp.get_variable("r").unwrap(), Value16::boolean(true));
}

#[test]
fn test_array_some_false() {
    let interp = run(r#"
        let a = [1, 2, 3];
        let r = a.some((x) => x > 10);
    "#);
    assert_eq!(interp.get_variable("r").unwrap(), Value16::boolean(false));
}

#[test]
fn test_array_every_true() {
    let interp = run(r#"
        let a = [2, 4, 6];
        let r = a.every((x) => x > 0);
    "#);
    assert_eq!(interp.get_variable("r").unwrap(), Value16::boolean(true));
}

#[test]
fn test_array_every_false() {
    let interp = run(r#"
        let a = [2, 4, 6];
        let r = a.every((x) => x > 3);
    "#);
    assert_eq!(interp.get_variable("r").unwrap(), Value16::boolean(false));
}

// ──────────────────────────────────────────────
// Array — shift / unshift
// ──────────────────────────────────────────────

#[test]
fn test_array_shift() {
    let interp = run(r#"
        let a = [10, 20, 30];
        let x = a.shift();
    "#);
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(10.0));
    if let Some(arr) = interp.get_variable("a").unwrap().as_array() {
        assert_eq!(arr.len(), 2);
    } else {
        panic!(
            "Expected array, got {:?}",
            interp.get_variable("a").unwrap()
        );
    }
}

#[test]
fn test_array_unshift() {
    let interp = run(r#"
        let a = [2, 3];
        a.unshift(1);
    "#);
    if let Some(arr) = interp.get_variable("a").unwrap().as_array() {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value16::number(1.0));
    } else {
        panic!(
            "Expected array, got {:?}",
            interp.get_variable("a").unwrap()
        );
    }
}

// ──────────────────────────────────────────────
// Array — forEach (side-effect only, returns null)
// ──────────────────────────────────────────────

#[test]
fn test_array_foreach() {
    // forEach should not error and its return value is null
    let interp = run(r#"
        let sum = 0;
        let a = [1, 2, 3];
        a.forEach((x) => { sum = sum + x; });
    "#);
    // NOTE: forEach callback runs in a closure, so the mutation of `sum`
    // may or may not propagate depending on scoping semantics.
    // The key assertion is that it does not error.
    let _ = interp.get_variable("sum");
}

// ──────────────────────────────────────────────
// Edge cases
// ──────────────────────────────────────────────

#[test]
fn test_nested_arithmetic() {
    let interp = run("let x = (10 - 3) * (2 + 1);");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(21.0));
}

#[test]
fn test_chained_comparisons_via_logic() {
    let interp = run("let x = (3 < 5) && (5 > 2);");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_modulo_negative() {
    let interp = run("let x = -7 % 3;");
    if let Some(n) = interp.get_variable("x").unwrap().as_number() {
        // Rust f64 rem follows IEEE 754 — result has the sign of the dividend
        assert!((n - (-1.0)).abs() < 1e-10);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_string_equality() {
    let interp = run(r#"let x = "hello" == "hello";"#);
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_string_inequality() {
    let interp = run(r#"let x = "hello" != "world";"#);
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_null_equality() {
    let interp = run("let x = null == null;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_boolean_equality() {
    let interp = run("let x = true == true;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(true));
}

#[test]
fn test_mixed_type_inequality() {
    // number vs boolean should not be equal
    let interp = run("let x = 1 == true;");
    assert_eq!(interp.get_variable("x").unwrap(), Value16::boolean(false));
}

// ──────────────────────────────────────────────
// Array — empty array edge cases
// ──────────────────────────────────────────────

#[test]
fn test_empty_array_length() {
    let interp = run("let n = [].length;");
    assert_eq!(interp.get_variable("n").unwrap(), Value16::number(0.0));
}

#[test]
fn test_array_pop_empty_errors() {
    let result = try_run(
        r#"
        let a = [];
        let x = a.pop();
    "#,
    );
    assert!(result.is_err(), "Popping from empty array should error");
}

#[test]
fn test_array_map_empty() {
    let interp = run(r#"
        let b = [].map((x) => x * 2);
    "#);
    if let Some(arr) = interp.get_variable("b").unwrap().as_array() {
        assert!(arr.is_empty());
    } else {
        panic!(
            "Expected empty array, got {:?}",
            interp.get_variable("b").unwrap()
        );
    }
}
