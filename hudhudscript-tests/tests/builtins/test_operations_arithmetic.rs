use hudhudscript_ast::{Literal, UnaryOp};
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use hudhudscript_shared_builtins::operations::arithmetic::{
    eval_add, eval_arithmetic, eval_comparison, eval_div, eval_mod,
};
use hudhudscript_shared_builtins::operations::helpers::{
    create_array, create_object, eval_index_access, eval_literal, eval_member_access, eval_unary_op,
};
use hudhudscript_shared_builtins::operations::validators::{
    check_arg_count, require_array, require_number, require_string,
};
use std::collections::HashMap;

/// Convert a HudHudScript Value16 to a serde_json::Value (inlined from deleted wrapper).
fn value_to_json(value: &Value16) -> HudHudResult<serde_json::Value> {
    if value.is_null() {
        Ok(serde_json::Value::Null)
    } else if let Some(b) = value.as_bool() {
        Ok(serde_json::Value::Bool(b))
    } else if let Some(n) = value.as_number() {
        Ok(serde_json::json!(n))
    } else if let Some(s) = value.as_str() {
        Ok(serde_json::Value::String(s.to_string()))
    } else if let Some(arr) = value.as_array() {
        let items: Result<Vec<_>, _> = arr.iter().map(|v| value_to_json(v)).collect();
        Ok(serde_json::Value::Array(items?))
    } else if let Some(map) = value.as_object() {
        let mut obj = serde_json::Map::new();
        for (k, v) in map {
            let key_str = hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(k.0)).to_string();
            obj.insert(key_str, value_to_json(v)?);
        }
        Ok(serde_json::Value::Object(obj))
    } else if let Some(opt) = value.as_option() {
        match opt {
            Some(inner) => value_to_json(inner),
            None => Ok(serde_json::Value::Null),
        }
    } else if let Some(r) = value.as_result() {
        match r {
            Ok(inner) => value_to_json(inner),
            Err(msg) => Ok(serde_json::json!({"error": msg})),
        }
    } else {
        let json_str = hudhudscript_shared_builtins::json::value_to_json_string(value);
        Ok(serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null))
    }
}

/// Convert a serde_json::Value to a HudHudScript Value (inlined from deleted wrapper).
fn json_to_value(json: &serde_json::Value) -> Value16 {
    hudhudscript_shared_builtins::json::serde_to_value(json)
}

// ══════════════════════════════════════════════════════════════════════
// Arithmetic operations
// ══════════════════════════════════════════════════════════════════════

#[test]
fn add_numbers() {
    let result = eval_add(&Value16::number(2.0), &Value16::number(3.0)).unwrap();
    assert_eq!(result, Value16::number(5.0));
}

#[test]
fn add_strings() {
    let result = eval_add(&Value16::string("hello"), &Value16::string(" world")).unwrap();
    assert_eq!(result, Value16::string("hello world"));
}

#[test]
fn add_string_and_number() {
    let result = eval_add(&Value16::string("count: "), &Value16::number(42.0)).unwrap();
    assert_eq!(result, Value16::string("count: 42"));
}

#[test]
fn add_number_and_string() {
    let result = eval_add(&Value16::number(7.0), &Value16::string(" items")).unwrap();
    assert_eq!(result, Value16::string("7 items"));
}

#[test]
fn add_string_and_bool_is_type_error() {
    // #745: String + Boolean is no longer auto-coerced — use explicit toString()
    let result = eval_add(&Value16::string("flag: "), &Value16::boolean(true));
    assert!(result.is_err(), "String + Boolean should be a type error");
}

#[test]
fn add_string_and_null_is_type_error() {
    // #745: String + Null is no longer auto-coerced
    let result = eval_add(&Value16::string("val: "), &Value16::null());
    assert!(result.is_err(), "String + Null should be a type error");
}

#[test]
fn add_incompatible_types_errors() {
    let result = eval_add(&Value16::boolean(true), &Value16::number(1.0));
    assert!(result.is_err());
}

#[test]
fn eval_subtraction() {
    let result = eval_arithmetic(
        &Value16::number(10.0),
        &Value16::number(3.0),
        |a, b| a - b,
        "subtraction",
    )
    .unwrap();
    assert_eq!(result, Value16::number(7.0));
}

#[test]
fn eval_multiplication() {
    let result = eval_arithmetic(
        &Value16::number(4.0),
        &Value16::number(5.0),
        |a, b| a * b,
        "multiplication",
    )
    .unwrap();
    assert_eq!(result, Value16::number(20.0));
}

#[test]
fn arithmetic_type_error() {
    let result = eval_arithmetic(
        &Value16::string("a"),
        &Value16::number(1.0),
        |a, b| a + b,
        "addition",
    );
    assert!(result.is_err());
}

#[test]
fn division_normal() {
    let result = eval_div(&Value16::number(10.0), &Value16::number(4.0)).unwrap();
    assert_eq!(result, Value16::number(2.5));
}

#[test]
fn division_by_zero() {
    let result = eval_div(&Value16::number(1.0), &Value16::number(0.0));
    assert!(result.is_err());
}

#[test]
fn division_type_error() {
    let result = eval_div(&Value16::string("a"), &Value16::number(1.0));
    assert!(result.is_err());
}

#[test]
fn modulo_normal() {
    let result = eval_mod(&Value16::number(10.0), &Value16::number(3.0)).unwrap();
    assert_eq!(result, Value16::number(1.0));
}

#[test]
fn modulo_by_zero() {
    let result = eval_mod(&Value16::number(5.0), &Value16::number(0.0));
    assert!(result.is_err());
}

#[test]
fn comparison_less_than() {
    let result =
        eval_comparison(&Value16::number(1.0), &Value16::number(2.0), |a, b| a < b).unwrap();
    assert_eq!(result, Value16::boolean(true));
}

#[test]
fn comparison_mixed_types_returns_false() {
    let result =
        eval_comparison(&Value16::string("a"), &Value16::number(1.0), |a, b| a < b).unwrap();
    assert_eq!(result, Value16::boolean(false));
}

// ══════════════════════════════════════════════════════════════════════
// Helpers: literals, unary ops, member access, indexing
// ══════════════════════════════════════════════════════════════════════

#[test]
fn literal_string() {
    assert_eq!(
        eval_literal(&Literal::String("hi".into())).unwrap(),
        Value16::string("hi")
    );
}

#[test]
fn literal_number() {
    assert_eq!(
        eval_literal(&Literal::Number(3.14, true)).unwrap(),
        Value16::number(3.14)
    );
}

#[test]
fn literal_boolean() {
    assert_eq!(
        eval_literal(&Literal::Boolean(false)).unwrap(),
        Value16::boolean(false)
    );
}

#[test]
fn literal_null() {
    assert_eq!(eval_literal(&Literal::Null).unwrap(), Value16::null());
}

#[test]
fn unary_not_truthy() {
    let result = eval_unary_op(UnaryOp::Not, Value16::boolean(true)).unwrap();
    assert_eq!(result, Value16::boolean(false));
}

#[test]
fn unary_neg() {
    let result = eval_unary_op(UnaryOp::Neg, Value16::number(5.0)).unwrap();
    assert_eq!(result, Value16::number(-5.0));
}

#[test]
fn unary_neg_type_error() {
    let result = eval_unary_op(UnaryOp::Neg, Value16::string("x"));
    assert!(result.is_err());
}

#[test]
fn unary_plus() {
    let result = eval_unary_op(UnaryOp::Plus, Value16::number(5.0)).unwrap();
    assert_eq!(result, Value16::number(5.0));
}

#[test]
fn unary_plus_type_error() {
    let result = eval_unary_op(UnaryOp::Plus, Value16::boolean(true));
    assert!(result.is_err());
}

#[test]
fn member_access_object_property() {
    let mut obj = HashMap::new();
    obj.insert("name".to_string(), Value16::string("alice"));
    let result = eval_member_access(Value16::object(obj), "name").unwrap();
    assert_eq!(result, Value16::string("alice"));
}

#[test]
fn member_access_object_not_found() {
    let obj: HashMap<hudhudscript_bytecode::sym::SymId, Value16> = HashMap::new();
    let result = eval_member_access(Value16::object(obj), "missing");
    assert!(result.is_err());
}

#[test]
fn member_access_string_length() {
    let result = eval_member_access(Value16::string("hello"), "length").unwrap();
    assert_eq!(result, Value16::number(5.0));
}

#[test]
fn member_access_array_length() {
    let arr = vec![Value16::number(1.0), Value16::number(2.0)];
    let result = eval_member_access(Value16::array(arr), "length").unwrap();
    assert_eq!(result, Value16::number(2.0));
}

#[test]
fn member_access_set_size() {
    let items = vec![
        Value16::number(1.0),
        Value16::number(2.0),
        Value16::number(3.0),
    ];
    let result = eval_member_access(Value16::set(items), "size").unwrap();
    assert_eq!(result, Value16::number(3.0));
}

#[test]
fn member_access_map_size() {
    let pairs = vec![
        (Value16::string("a"), Value16::number(1.0)),
        (Value16::string("b"), Value16::number(2.0)),
    ];
    let result = eval_member_access(Value16::map(pairs), "size").unwrap();
    assert_eq!(result, Value16::number(2.0));
}

#[test]
fn index_access_array() {
    let arr = vec![Value16::string("a"), Value16::string("b")];
    let result = eval_index_access(Value16::array(arr), Value16::number(1.0)).unwrap();
    assert_eq!(result, Value16::string("b"));
}

#[test]
fn index_access_array_out_of_bounds() {
    let arr = vec![Value16::number(1.0)];
    let result = eval_index_access(Value16::array(arr), Value16::number(5.0));
    assert!(result.is_err());
}

#[test]
fn index_access_object_by_key() {
    let mut obj = HashMap::new();
    obj.insert("x".to_string(), Value16::number(42.0));
    let result = eval_index_access(Value16::object(obj), Value16::string("x")).unwrap();
    assert_eq!(result, Value16::number(42.0));
}

#[test]
fn create_array_helper() {
    let arr = create_array(vec![Value16::number(1.0), Value16::number(2.0)]);
    assert_eq!(
        arr,
        Value16::array(vec![Value16::number(1.0), Value16::number(2.0)])
    );
}

#[test]
fn create_object_helper() {
    let mut props = HashMap::new();
    props.insert("key".to_string(), Value16::boolean(true));
    let obj = create_object(props.clone());
    assert_eq!(obj, Value16::object(props));
}

// ══════════════════════════════════════════════════════════════════════
// JSON conversion
// ══════════════════════════════════════════════════════════════════════

#[test]
fn json_roundtrip_null() {
    let json = value_to_json(&Value16::null()).unwrap();
    assert_eq!(json, serde_json::Value::Null);
    assert_eq!(json_to_value(&json), Value16::null());
}

#[test]
fn json_roundtrip_bool() {
    let json = value_to_json(&Value16::boolean(true)).unwrap();
    assert_eq!(json, serde_json::Value::Bool(true));
    assert_eq!(json_to_value(&json), Value16::boolean(true));
}

#[test]
fn json_roundtrip_number() {
    let json = value_to_json(&Value16::number(3.14)).unwrap();
    let back = json_to_value(&json);
    if let Some(n) = back.as_number() {
        assert!((n - 3.14).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

#[test]
fn json_roundtrip_string() {
    let json = value_to_json(&Value16::string("test")).unwrap();
    assert_eq!(json, serde_json::Value::String("test".into()));
    assert_eq!(json_to_value(&json), Value16::string("test"));
}

#[test]
fn json_roundtrip_array() {
    let arr = Value16::array(vec![Value16::number(1.0), Value16::string("two")]);
    let json = value_to_json(&arr).unwrap();
    let back = json_to_value(&json);
    if let Some(items) = back.as_array() {
        assert_eq!(items.len(), 2);
        assert_eq!(items[1], Value16::string("two"));
    } else {
        panic!("expected array");
    }
}

#[test]
fn json_roundtrip_object() {
    let mut obj = HashMap::new();
    obj.insert("key".to_string(), Value16::boolean(false));
    let json = value_to_json(&Value16::object(obj)).unwrap();
    let back = json_to_value(&json);
    if let Some(map) = back.as_object() {
        assert_eq!(map.get("key"), Some(&Value16::boolean(false)));
    } else {
        panic!("expected object");
    }
}

#[test]
fn json_option_some() {
    let val = Value16::option(Some(Value16::number(99.0)));
    let json = value_to_json(&val).unwrap();
    // Option(Some(x)) should serialize as x
    assert_eq!(json, serde_json::json!(99.0));
}

#[test]
fn json_option_none() {
    let val = Value16::option(None);
    let json = value_to_json(&val).unwrap();
    assert_eq!(json, serde_json::Value::Null);
}

#[test]
fn json_result_ok() {
    let val = Value16::result(Ok(Value16::string("ok")));
    let json = value_to_json(&val).unwrap();
    assert_eq!(json, serde_json::Value::String("ok".into()));
}

#[test]
fn json_result_err() {
    let val = Value16::result(Err("boom".to_string()));
    let json = value_to_json(&val).unwrap();
    let obj = json.as_object().unwrap();
    assert_eq!(
        obj.get("error").unwrap(),
        &serde_json::Value::String("boom".into())
    );
}

// ══════════════════════════════════════════════════════════════════════
// Validators
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validator_check_arg_count_ok() {
    let args = vec![Value16::number(1.0), Value16::number(2.0)];
    assert!(check_arg_count(&args, 2, "test").is_ok());
}

#[test]
fn validator_check_arg_count_mismatch() {
    let args = vec![Value16::number(1.0)];
    assert!(check_arg_count(&args, 3, "test").is_err());
}

#[test]
fn validator_require_string_ok() {
    let args = vec![Value16::string("hello")];
    assert_eq!(require_string(&args, 0, "test").unwrap(), "hello");
}

#[test]
fn validator_require_string_wrong_type() {
    let args = vec![Value16::number(1.0)];
    assert!(require_string(&args, 0, "test").is_err());
}

#[test]
fn validator_require_string_out_of_bounds() {
    let args: Vec<Value16> = vec![];
    assert!(require_string(&args, 0, "test").is_err());
}

#[test]
fn validator_require_number_ok() {
    let args = vec![Value16::number(3.14)];
    assert!((require_number(&args, 0, "test").unwrap() - 3.14).abs() < 1e-10);
}

#[test]
fn validator_require_number_wrong_type() {
    let args = vec![Value16::boolean(true)];
    assert!(require_number(&args, 0, "test").is_err());
}

#[test]
fn validator_require_array_ok() {
    let args = vec![Value16::array(vec![Value16::number(1.0)])];
    let arr = require_array(&args, 0, "test").unwrap();
    assert_eq!(arr.len(), 1);
}

#[test]
fn validator_require_array_wrong_type() {
    let args = vec![Value16::string("not array")];
    assert!(require_array(&args, 0, "test").is_err());
}
