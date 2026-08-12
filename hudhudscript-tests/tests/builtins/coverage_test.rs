//! Coverage-focused tests — migrated from `hudhudscript-builtins` to
//! `hudhudscript-shared-builtins` (Kural 7).  After the deletion of the
//! hudhudscript-builtins crate, the shared crate holds the canonical
//! implementation of every operation, so these tests point at the same
//! code both runtimes call at runtime.
//!
//! The interpreter-era `promise.rs` direct-call tests (which probed the
//! `Value::NativeFunction`-wrapped `Promise.all`/`race` combinators
//! stored in the `create_promise_object()` map) are gone: they were
//! structural assertions on a runtime representation that has no shared
//! counterpart — the VM owns the real concurrent Promise.all/race
//! implementation in `shared::blocking_registry::await_all_blocking` /
//! `await_race_blocking`, and scripts hit that path through
//! `Promise.all([...])` / `Promise.race([...])` at the language level.
//! If/when you want Promise.all/race end-to-end coverage, write a .hud
//! snippet and run it through the VM.

use hudhudscript_ast::{Literal, UnaryOp};
use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::crypto_ops;
use hudhudscript_shared_builtins::operations::{arithmetic, helpers, validators};
use std::collections::HashMap;

/// Local json helpers replacing deleted operations/json.rs wrappers.
mod json {
    use hudhudscript_bytecode::Value16;
    use hudhudscript_errors::HudHudResult;

    pub fn value_to_json(value: &Value16) -> HudHudResult<serde_json::Value> {
        if value.is_null() {
            Ok(serde_json::Value::Null)
        } else if let Some(b) = value.as_bool() {
            Ok(serde_json::Value::Bool(b))
        } else if let Some(n) = value.as_number() {
            if n.is_nan() || n.is_infinite() {
                return Err(
                    hudhud_script_tests::vm_interpreter::runtime_codes::type_error(
                        "number".to_string(),
                        "NaN/Infinity".to_string(),
                        "JSON".to_string(),
                    ),
                );
            }
            Ok(serde_json::json!(n))
        } else if let Some(s) = value.as_string() {
            Ok(serde_json::Value::String(s))
        } else if let Some(arr) = value.as_array() {
            let items: Result<Vec<_>, _> = arr.iter().map(|v| value_to_json(v)).collect();
            Ok(serde_json::Value::Array(items?))
        } else if let Some(map) = value.as_object() {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                let key_str = hudhudscript_bytecode::interner::resolve(
                    hudhudscript_bytecode::interner::SymbolId(k.0),
                )
                .to_string();
                obj.insert(key_str, value_to_json(v)?);
            }
            Ok(serde_json::Value::Object(obj))
        } else if let Some(opt) = value.as_option() {
            match opt {
                Some(inner) => value_to_json(inner),
                None => Ok(serde_json::Value::Null),
            }
        } else if let Some(res) = value.as_result() {
            match res {
                Ok(inner) => value_to_json(inner),
                Err(msg) => Ok(serde_json::json!({"error": msg})),
            }
        } else {
            let json_str = hudhudscript_shared_builtins::json::value_to_json_string(value);
            Ok(serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null))
        }
    }

    pub fn json_to_value(json: &serde_json::Value) -> Value16 {
        hudhudscript_shared_builtins::json::serde_to_value(json)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// operations/validators.rs
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_check_arg_count_exact() {
    let args = vec![Value16::number(1.0), Value16::number(2.0)];
    assert!(validators::check_arg_count(&args, 2, "test_fn").is_ok());
}

#[test]
fn test_check_arg_count_too_few() {
    let args = vec![Value16::number(1.0)];
    let err = validators::check_arg_count(&args, 2, "test_fn").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("test_fn"));
}

#[test]
fn test_check_arg_count_too_many() {
    let args = vec![
        Value16::number(1.0),
        Value16::number(2.0),
        Value16::number(3.0),
    ];
    let err = validators::check_arg_count(&args, 2, "test_fn").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("test_fn"));
}

#[test]
fn test_require_string_success() {
    let args = vec![Value16::string("hello".to_string())];
    let result = validators::require_string(&args, 0, "test_fn").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_require_string_wrong_type() {
    let args = vec![Value16::number(42.0)];
    let err = validators::require_string(&args, 0, "test_fn").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("string") || msg.contains("number"));
}

#[test]
fn test_require_string_out_of_bounds() {
    let args: Vec<Value16> = vec![];
    let err = validators::require_string(&args, 0, "test_fn").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("test_fn"));
}

#[test]
fn test_require_number_success() {
    let args = vec![Value16::number(3.14)];
    let result = validators::require_number(&args, 0, "test_fn").unwrap();
    assert!((result - 3.14).abs() < 1e-10);
}

#[test]
fn test_require_number_wrong_type() {
    let args = vec![Value16::string("hello".to_string())];
    let err = validators::require_number(&args, 0, "test_fn").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("number") || msg.contains("string"));
}

#[test]
fn test_require_number_out_of_bounds() {
    let args: Vec<Value16> = vec![];
    let err = validators::require_number(&args, 1, "test_fn").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("test_fn"));
}

#[test]
fn test_require_array_success() {
    let args = vec![Value16::array(vec![
        Value16::number(1.0),
        Value16::number(2.0),
    ])];
    let result = validators::require_array(&args, 0, "test_fn").unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_require_array_wrong_type() {
    let args = vec![Value16::string("not an array".to_string())];
    let err = validators::require_array(&args, 0, "test_fn").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("array") || msg.contains("string"));
}

#[test]
fn test_require_array_out_of_bounds() {
    let args: Vec<Value16> = vec![];
    let err = validators::require_array(&args, 0, "test_fn").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("test_fn"));
}

// ─────────────────────────────────────────────────────────────────────────────
// operations/json.rs (local module above — shared does not re-export the
// owned-Value JSON wrappers).
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_value_to_json_null() {
    let result = json::value_to_json(&Value16::null()).unwrap();
    assert_eq!(result, serde_json::Value::Null);
}

#[test]
fn test_value_to_json_bool_true() {
    let result = json::value_to_json(&Value16::boolean(true)).unwrap();
    assert_eq!(result, serde_json::Value::Bool(true));
}

#[test]
fn test_value_to_json_bool_false() {
    let result = json::value_to_json(&Value16::boolean(false)).unwrap();
    assert_eq!(result, serde_json::Value::Bool(false));
}

#[test]
fn test_value_to_json_number() {
    let result = json::value_to_json(&Value16::number(42.0)).unwrap();
    assert_eq!(result, serde_json::json!(42.0));
}

#[test]
fn test_value_to_json_string() {
    let result = json::value_to_json(&Value16::string("hello".to_string())).unwrap();
    assert_eq!(result, serde_json::Value::String("hello".to_string()));
}

#[test]
fn test_value_to_json_array() {
    let arr = Value16::array(vec![
        Value16::number(1.0),
        Value16::string("two".to_string()),
    ]);
    let result = json::value_to_json(&arr).unwrap();
    assert_eq!(result, serde_json::json!([1.0, "two"]));
}

#[test]
fn test_value_to_json_object() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value16::string("val".to_string()));
    let obj = Value16::object(map);
    let result = json::value_to_json(&obj).unwrap();
    assert_eq!(result["key"], serde_json::Value::String("val".to_string()));
}

#[test]
fn test_value_to_json_option_some() {
    let v = Value16::option(Some(Value16::number(99.0)));
    let result = json::value_to_json(&v).unwrap();
    assert_eq!(result, serde_json::json!(99.0));
}

#[test]
fn test_value_to_json_option_none() {
    let v = Value16::option(None);
    let result = json::value_to_json(&v).unwrap();
    assert_eq!(result, serde_json::Value::Null);
}

#[test]
fn test_value_to_json_result_ok() {
    let v = Value16::result(Ok(Value16::boolean(true)));
    let result = json::value_to_json(&v).unwrap();
    assert_eq!(result, serde_json::Value::Bool(true));
}

#[test]
fn test_value_to_json_result_err() {
    let v = Value16::result(Err("something failed".to_string()));
    let result = json::value_to_json(&v).unwrap();
    assert_eq!(
        result["error"],
        serde_json::Value::String("something failed".to_string())
    );
}

#[test]
fn test_value_to_json_nan_errors() {
    let result = json::value_to_json(&Value16::number(f64::NAN));
    assert!(result.is_err());
}

#[test]
fn test_json_to_value_null() {
    let result = json::json_to_value(&serde_json::Value::Null);
    assert_eq!(result, Value16::null());
}

#[test]
fn test_json_to_value_bool() {
    let result = json::json_to_value(&serde_json::Value::Bool(true));
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_json_to_value_number() {
    let result = json::json_to_value(&serde_json::json!(3.14));
    assert!(result.as_number().is_some());
}

#[test]
fn test_json_to_value_string() {
    let result = json::json_to_value(&serde_json::Value::String("test".to_string()));
    assert_eq!(result, Value16::string("test".to_string()));
}

#[test]
fn test_json_to_value_array() {
    let json_arr = serde_json::json!([1, 2, 3]);
    let result = json::json_to_value(&json_arr);
    if let Some(arr) = result.as_array() {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_json_to_value_object() {
    let json_obj = serde_json::json!({"a": 1, "b": "two"});
    let result = json::json_to_value(&json_obj);
    if let Some(obj) = result.as_object() {
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("b"));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_json_roundtrip() {
    let original = serde_json::json!({"name": "Alice", "age": 30, "active": true});
    let value = json::json_to_value(&original);
    let back = json::value_to_json(&value).unwrap();
    assert_eq!(back["name"], serde_json::Value::String("Alice".to_string()));
    assert_eq!(back["age"], serde_json::json!(30.0));
}

// ─────────────────────────────────────────────────────────────────────────────
// operations/helpers.rs
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_literal_string() {
    let lit = Literal::String("hello".to_string());
    let result = helpers::eval_literal(&lit).unwrap();
    assert_eq!(result, Value16::string("hello".to_string()));
}

#[test]
fn test_eval_literal_number() {
    let lit = Literal::Number(42.5, false);
    let result = helpers::eval_literal(&lit).unwrap();
    assert_eq!(result, Value16::number(42.5));
}

#[test]
fn test_eval_literal_boolean_true() {
    let lit = Literal::Boolean(true);
    let result = helpers::eval_literal(&lit).unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_eval_literal_null() {
    let lit = Literal::Null;
    let result = helpers::eval_literal(&lit).unwrap();
    assert_eq!(result, Value16::null());
}

#[test]
fn test_eval_unary_op_not_truthy() {
    let result = helpers::eval_unary_op(UnaryOp::Not, Value16::boolean(true)).unwrap();
    assert_eq!(result, Value16::boolean(false));
}

#[test]
fn test_eval_unary_op_not_falsy() {
    let result = helpers::eval_unary_op(UnaryOp::Not, Value16::boolean(false)).unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_eval_unary_op_neg_number() {
    let result = helpers::eval_unary_op(UnaryOp::Neg, Value16::number(5.0)).unwrap();
    assert_eq!(result, Value16::number(-5.0));
}

#[test]
fn test_eval_unary_op_neg_type_error() {
    let err =
        helpers::eval_unary_op(UnaryOp::Neg, Value16::string("hello".to_string())).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("number") || msg.contains("string"));
}

#[test]
fn test_eval_unary_op_plus_number() {
    let result = helpers::eval_unary_op(UnaryOp::Plus, Value16::number(7.0)).unwrap();
    assert_eq!(result, Value16::number(7.0));
}

#[test]
fn test_eval_unary_op_plus_type_error() {
    let err = helpers::eval_unary_op(UnaryOp::Plus, Value16::null()).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("number") || msg.contains("null"));
}

#[test]
fn test_eval_member_access_object_found() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value16::string("Alice".to_string()));
    let obj = Value16::object(map);
    let result = helpers::eval_member_access(obj, "name").unwrap();
    assert_eq!(result, Value16::string("Alice".to_string()));
}

#[test]
fn test_eval_member_access_object_not_found() {
    let obj = Value16::object(HashMap::<hudhudscript_bytecode::sym::SymId, Value16>::new());
    let err = helpers::eval_member_access(obj, "missing").unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("missing") || msg.contains("PropertyNotFound"));
}

#[test]
fn test_eval_member_access_string_length() {
    let s = Value16::string("hello".to_string());
    let result = helpers::eval_member_access(s, "length").unwrap();
    assert_eq!(result, Value16::number(5.0));
}

#[test]
fn test_eval_member_access_array_length() {
    let arr = Value16::array(vec![
        Value16::number(1.0),
        Value16::number(2.0),
        Value16::number(3.0),
    ]);
    let result = helpers::eval_member_access(arr, "length").unwrap();
    assert_eq!(result, Value16::number(3.0));
}

#[test]
fn test_eval_member_access_set_size() {
    let sv = Value16::set(vec![Value16::number(1.0), Value16::number(2.0)]);
    let result = helpers::eval_member_access(sv, "size").unwrap();
    assert_eq!(result, Value16::number(2.0));
}

#[test]
fn test_eval_index_access_array_valid() {
    let arr = Value16::array(vec![
        Value16::string("a".to_string()),
        Value16::string("b".to_string()),
        Value16::string("c".to_string()),
    ]);
    let result = helpers::eval_index_access(arr, Value16::number(1.0)).unwrap();
    assert_eq!(result, Value16::string("b".to_string()));
}

#[test]
fn test_eval_index_access_array_out_of_bounds() {
    let arr = Value16::array(vec![Value16::number(1.0)]);
    let err = helpers::eval_index_access(arr, Value16::number(5.0)).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("IndexOutOfBounds") || msg.contains("index"));
}

#[test]
fn test_eval_index_access_negative_index() {
    let arr = Value16::array(vec![Value16::number(1.0)]);
    let err = helpers::eval_index_access(arr, Value16::number(-1.0)).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("IndexOutOfBounds") || msg.contains("index"));
}

#[test]
fn test_eval_index_access_object_key() {
    let mut map = HashMap::new();
    map.insert("foo".to_string(), Value16::number(42.0));
    let obj = Value16::object(map);
    let result = helpers::eval_index_access(obj, Value16::string("foo".to_string())).unwrap();
    assert_eq!(result, Value16::number(42.0));
}

#[test]
fn test_eval_index_access_type_mismatch() {
    let s = Value16::string("hello".to_string());
    let err = helpers::eval_index_access(s, Value16::number(0.0)).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("TypeError")
            || msg.contains("type")
            || msg.contains("error")
            || msg.contains("index"),
        "Got: {}",
        msg
    );
}

#[test]
fn test_create_array() {
    let result = helpers::create_array(vec![Value16::number(1.0), Value16::number(2.0)]);
    assert_eq!(
        result,
        Value16::array(vec![Value16::number(1.0), Value16::number(2.0)])
    );
}

#[test]
fn test_create_object() {
    let mut props = HashMap::new();
    props.insert("x".to_string(), Value16::number(10.0));
    let result = helpers::create_object(props);
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("x"), Some(&Value16::number(10.0)));
    } else {
        panic!("Expected object");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// operations/arithmetic.rs
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_eval_add_numbers() {
    let result = arithmetic::eval_add(&Value16::number(3.0), &Value16::number(4.0)).unwrap();
    assert_eq!(result, Value16::number(7.0));
}

#[test]
fn test_eval_add_strings() {
    let result = arithmetic::eval_add(
        &Value16::string("foo".to_string()),
        &Value16::string("bar".to_string()),
    )
    .unwrap();
    assert_eq!(result, Value16::string("foobar".to_string()));
}

#[test]
fn test_eval_add_string_number() {
    let result =
        arithmetic::eval_add(&Value16::string("num=".to_string()), &Value16::number(42.0)).unwrap();
    if let Some(s) = result.as_string() {
        assert!(s.starts_with("num="));
        assert!(s.contains("42"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_eval_add_number_string() {
    let result =
        arithmetic::eval_add(&Value16::number(1.0), &Value16::string(" item".to_string())).unwrap();
    if let Some(s) = result.as_string() {
        assert!(s.contains("1"));
        assert!(s.contains("item"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_eval_add_incompatible_types() {
    let err = arithmetic::eval_add(&Value16::null(), &Value16::null()).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("TypeError") || msg.contains("number or string"));
}

#[test]
fn test_eval_arithmetic_subtraction() {
    let result = arithmetic::eval_arithmetic(
        &Value16::number(10.0),
        &Value16::number(3.0),
        |a, b| a - b,
        "subtraction",
    )
    .unwrap();
    assert_eq!(result, Value16::number(7.0));
}

#[test]
fn test_eval_arithmetic_multiplication() {
    let result = arithmetic::eval_arithmetic(
        &Value16::number(4.0),
        &Value16::number(5.0),
        |a, b| a * b,
        "multiplication",
    )
    .unwrap();
    assert_eq!(result, Value16::number(20.0));
}

#[test]
fn test_eval_arithmetic_type_error() {
    let err = arithmetic::eval_arithmetic(
        &Value16::string("x".to_string()),
        &Value16::number(2.0),
        |a, b| a + b,
        "addition",
    )
    .unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("TypeError") || msg.contains("number"));
}

#[test]
fn test_eval_div_normal() {
    let result = arithmetic::eval_div(&Value16::number(10.0), &Value16::number(2.0)).unwrap();
    assert_eq!(result, Value16::number(5.0));
}

#[test]
fn test_eval_div_by_zero() {
    let err = arithmetic::eval_div(&Value16::number(5.0), &Value16::number(0.0)).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("DivisionByZero")
            || msg.contains("division")
            || msg.contains("zero")
            || msg.contains("error"),
        "Got: {}",
        msg
    );
}

#[test]
fn test_eval_div_type_error() {
    let err = arithmetic::eval_div(&Value16::boolean(true), &Value16::number(2.0)).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(msg.contains("TypeError") || msg.contains("number"));
}

#[test]
fn test_eval_comparison_less_than() {
    let result =
        arithmetic::eval_comparison(&Value16::number(3.0), &Value16::number(5.0), |a, b| a < b)
            .unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn test_eval_comparison_greater_than_false() {
    let result =
        arithmetic::eval_comparison(&Value16::number(3.0), &Value16::number(5.0), |a, b| a > b)
            .unwrap();
    assert_eq!(result, Value16::boolean(false));
}

#[test]
fn test_eval_comparison_mixed_types_returns_false() {
    // After KM-11 fix: mixed-type comparisons return false instead of TypeError
    let result = arithmetic::eval_comparison(
        &Value16::string("a".to_string()),
        &Value16::number(1.0),
        |a, b| a < b,
    )
    .unwrap();
    assert_eq!(result, Value16::boolean(false));
}
