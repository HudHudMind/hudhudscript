//! Evaluation Helpers Module (Value16)
//!
//! This module contains helper functions for evaluating expressions.
//! Single source of truth for the VM (Kural 7).

use hudhudscript_ast::{Literal, UnaryOp};
use hudhudscript_bytecode::shared_value::SharedResult;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

/// Evaluate literal
pub fn eval_literal(lit: &Literal) -> SharedResult<Value16> {
    Ok(match lit {
        Literal::String(s) => Value16::string(s.clone()),
        Literal::Number(n, _) => Value16::number(*n),
        Literal::Int(i) => Value16::int(*i),
        Literal::BigInt(s) => {
            let big = s.parse::<num_bigint::BigInt>().map_err(|_| {
                hudhudscript_bytecode::shared_value::runtime_error("Invalid BigInt literal".to_string())
            })?;
            Value16::bigint(big)
        }
        Literal::Boolean(b) => Value16::boolean(*b),
        Literal::Null => Value16::null(),
    })
}

/// Evaluate unary operation on a value
pub fn eval_unary_op(op: UnaryOp, val: Value16) -> SharedResult<Value16> {
    match op {
        UnaryOp::Not => Ok(Value16::boolean(!val.is_truthy())),
        UnaryOp::Neg => {
            if let Some(i) = val.as_int_fast() {
                Ok(Value16::int(-i))
            } else if let Some(n) = val.as_number_fast() {
                Ok(Value16::number(-n))
            } else if let Some(b) = val.as_bigint() {
                Ok(Value16::bigint(-b.clone()))
            } else {
                Err(hudhudscript_bytecode::shared_value::type_error_ctx(
                    "number".to_string(),
                    val.type_name_str().to_string(),
                    "negation".to_string(),
                ))
            }
        }
        UnaryOp::Plus => {
            if val.is_int() || val.is_number() || val.as_bigint().is_some() {
                Ok(val)
            } else {
                Err(hudhudscript_bytecode::shared_value::type_error_ctx(
                    "number".to_string(),
                    val.type_name_str().to_string(),
                    "unary plus".to_string(),
                ))
            }
        }
        UnaryOp::PostIncrement => match val.as_int() {
            Some(n) => Ok(Value16::int(n + 1)),
            None => match val.as_number() {
                Some(n) => Ok(Value16::number(n + 1.0)),
                None => Err(hudhudscript_bytecode::shared_value::type_error_ctx(
                    "number".to_string(),
                    val.type_name_str().to_string(),
                    "postfix ++".to_string(),
                )),
            },
        },
        UnaryOp::PostDecrement => match val.as_int() {
            Some(n) => Ok(Value16::int(n - 1)),
            None => match val.as_number() {
                Some(n) => Ok(Value16::number(n - 1.0)),
                None => Err(hudhudscript_bytecode::shared_value::type_error_ctx(
                    "number".to_string(),
                    val.type_name_str().to_string(),
                    "postfix --".to_string(),
                )),
            },
        },
    }
}

/// Evaluate member access on a value.
///
/// Semantics (match the interpreter-era implementation exactly):
/// 1. `Object`: direct property → `__static__<property>` → walk `__parent__`
///    chain → error.
/// 2. `Class`: `__static__<property>` in methods, then in fields, then
///    non-static methods, then non-static fields, then walk parent class.
/// 3. `Instance`: instance fields, then look up the property on the backing
///    class (prototype chain).
/// 4. `String`, `Array`, `Set`, `Map`: only the `length`/`size` property is
///    directly accessible; method calls are handled elsewhere.
/// 5. Any other value: `PropertyNotFound`.
pub fn eval_member_access(obj: Value16, property: &str) -> SharedResult<Value16> {
    // ── Object ─────────────────────────────────────────────────────────
    if let Some(map) = obj.as_object() {
        // First try direct property access.
        if let Some(value) = map.get(property) {
            return Ok(value.clone());
        }

        // If not found, try static member (prefixed with __static__).
        let static_key = format!("__static__{}", property);
        if let Some(value) = map.get(&static_key) {
            return Ok(value.clone());
        }

        // Walk the prototype chain (__parent__) for inherited properties.
        if let Some(parent) = map.get("__parent__") {
            if let Ok(val) = eval_member_access(parent.clone(), property) {
                return Ok(val);
            }
        }

        // Property not found.
        return Err(hudhudscript_bytecode::shared_value::property_not_found(
            property.to_string(),
            "object".to_string(),
        ));
    }

    // ── Class ──────────────────────────────────────────────────────────
    if let (Some(methods), Some(fields), name) = (
        obj.as_class_methods(),
        obj.as_class_fields(),
        obj.as_class_name(),
    ) {
        // Static member access on a class.
        let static_key = format!("__static__{}", property);
        if let Some(value) = methods.get(&static_key) {
            return Ok(value.clone());
        }
        if let Some(value) = fields.get(&static_key) {
            return Ok(value.clone());
        }
        // Also check non-static methods (class-level access).
        if let Some(value) = methods.get(property) {
            return Ok(value.clone());
        }
        if let Some(value) = fields.get(property) {
            return Ok(value.clone());
        }
        // Walk parent chain.
        if let Some(p) = obj.as_class_parent() {
            if let Ok(val) = eval_member_access(p.clone(), property) {
                return Ok(val);
            }
        }
        return Err(hudhudscript_bytecode::shared_value::property_not_found(
            property.to_string(),
            format!("class {}", name.unwrap_or("")),
        ));
    }

    // ── Instance ───────────────────────────────────────────────────────
    if let Some(fields) = obj.as_instance_fields() {
        // First check instance fields.
        if let Some(value) = fields.get(property) {
            return Ok(value.clone());
        }
        // Then check the class for methods (prototype chain).
        if let Some(class) = obj.as_instance_class() {
            if let Ok(val) = eval_member_access(class.clone(), property) {
                return Ok(val);
            }
        }
        let class_name = obj.as_class_name().unwrap_or("").to_string();
        return Err(hudhudscript_bytecode::shared_value::property_not_found(
            property.to_string(),
            format!("{} instance", class_name),
        ));
    }

    // ── String ─────────────────────────────────────────────────────────
    if let Some(char_len) = obj.str_char_len() {
        return match property {
            "length" => Ok(Value16::int(char_len as i64)),
            // For methods, callers handle dispatch via eval_call.
            _ => Err(hudhudscript_bytecode::shared_value::property_not_found(
                property.to_string(),
                "string".to_string(),
            )),
        };
    }

    // ── Array ──────────────────────────────────────────────────────────
    if let Some(arr) = obj.as_array() {
        return match property {
            "length" => Ok(Value16::int(arr.len() as i64)),
            _ => Err(hudhudscript_bytecode::shared_value::property_not_found(
                property.to_string(),
                "array".to_string(),
            )),
        };
    }

    // ── Set ────────────────────────────────────────────────────────────
    if let Some(items) = obj.as_set() {
        return match property {
            "size" | "length" => Ok(Value16::int(items.len() as i64)),
            _ => Err(hudhudscript_bytecode::shared_value::property_not_found(
                property.to_string(),
                "set".to_string(),
            )),
        };
    }

    // ── Map ────────────────────────────────────────────────────────────
    if let Some(pairs) = obj.as_map_pairs() {
        return match property {
            "size" | "length" => Ok(Value16::number(pairs.len() as f64)),
            _ => Err(hudhudscript_bytecode::shared_value::property_not_found(
                property.to_string(),
                "map".to_string(),
            )),
        };
    }

    // ── Fallback ───────────────────────────────────────────────────────
    Err(hudhudscript_bytecode::shared_value::property_not_found(
        property.to_string(),
        obj.type_name_str().to_string(),
    ))
}

/// Evaluate index access on a value.
pub fn eval_index_access(obj: Value16, idx: Value16) -> SharedResult<Value16> {
    // Array + Number → element clone (bounds-checked).
    if let (Some(arr), Some(n)) = (obj.as_array(), idx.as_number()) {
        let index = n as i64;
        if index < 0 || index >= arr.len() as i64 {
            return Err(hudhudscript_bytecode::shared_value::index_out_of_bounds(
                index,
                arr.len(),
            ));
        }
        return Ok(arr[index as usize].clone());
    }
    // Object + String → lookup.
    if let (Some(map), Some(key)) = (obj.as_object(), idx.as_str()) {
        return map.get(key).cloned().ok_or_else(|| {
            hudhudscript_bytecode::shared_value::property_not_found(
                key.to_string(),
                "object".to_string(),
            )
        });
    }
    Err(hudhudscript_bytecode::shared_value::type_error_ctx(
        "array with number index or object with string key".to_string(),
        format!("{} with {} index", obj.type_name_str(), idx.type_name_str()),
        "indexing".to_string(),
    ))
}

/// Create an array value from evaluated elements.
pub fn create_array(values: Vec<Value16>) -> Value16 {
    Value16::array(values)
}

/// Create an object value from evaluated properties.
pub fn create_object<I>(properties: I) -> Value16
where
    I: IntoIterator<Item = (String, Value16)>,
{
    Value16::object(properties)
}
