//! Shared Set method implementations — used by both VM and interpreter.
//! Uses SharedValue::values_equal for deep equality.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

/// Execute a Set method. `items` is the current set contents.
/// Set is represented as Vec with unique elements (deep equality).
///
/// Returns the new set value or a scalar result.
pub fn call_set_method(items: &[Value16], method: &str, args: &[Value16]) -> SharedResult<Value16> {
    match method {
        "add" => {
            let val = args
                .first()
                .ok_or_else(|| runtime_error("set.add requires 1 argument"))?;
            let mut new_items = items.to_vec();
            if !new_items.iter().any(|v| v.values_equal(val)) {
                new_items.push(val.clone());
            }
            Ok(set_value(new_items))
        }
        "remove" | "delete" => {
            let val = args
                .first()
                .ok_or_else(|| runtime_error("set.remove requires 1 argument"))?;
            let new_items: Vec<Value16> = items
                .iter()
                .filter(|v| !v.values_equal(val))
                .cloned()
                .collect();
            Ok(set_value(new_items))
        }
        "has" | "contains" => {
            let val = args
                .first()
                .ok_or_else(|| runtime_error("set.has requires 1 argument"))?;
            Ok(Value16::boolean(items.iter().any(|v| v.values_equal(val))))
        }
        "union" => {
            let other = args
                .first()
                .and_then(|v| v.as_set().or_else(|| v.as_array()))
                .ok_or_else(|| runtime_error("set.union requires a Set argument"))?;
            let mut result = items.to_vec();
            for v in other {
                if !result.iter().any(|x| x.values_equal(v)) {
                    result.push(v.clone());
                }
            }
            Ok(set_value(result))
        }
        "intersection" => {
            let other = args
                .first()
                .and_then(|v| v.as_set().or_else(|| v.as_array()))
                .ok_or_else(|| runtime_error("set.intersection requires a Set argument"))?;
            let result: Vec<Value16> = items
                .iter()
                .filter(|v| other.iter().any(|o| v.values_equal(o)))
                .cloned()
                .collect();
            Ok(set_value(result))
        }
        "difference" => {
            let other = args
                .first()
                .and_then(|v| v.as_set().or_else(|| v.as_array()))
                .ok_or_else(|| runtime_error("set.difference requires a Set argument"))?;
            let result: Vec<Value16> = items
                .iter()
                .filter(|v| !other.iter().any(|o| v.values_equal(o)))
                .cloned()
                .collect();
            Ok(set_value(result))
        }
        "size" | "length" | "len" => Ok(Value16::int(items.len() as i64)),
        "toArray" | "to_array" | "values" => Ok(Value16::array(items.to_vec())),
        "clear" => Ok(set_value(Vec::new())),
        _ => Err(runtime_error(format!("Unknown method '{}' on Set", method))),
    }
}

/// Construct a Set value using the SharedValue::set() constructor.
fn set_value(items: Vec<Value16>) -> Value16 {
    Value16::set(items)
}
