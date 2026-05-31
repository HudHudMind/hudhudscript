//! Shared Map method implementations — used by both VM and interpreter.
//! Uses SharedValue::values_equal for deep equality on keys.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

/// Execute a Map method. `pairs` is the current map contents (key-value pairs).
/// Map is represented as Vec<(V, V)> with unique keys (deep equality).
///
/// Returns the new map value or a scalar result.
pub fn call_map_method(
    pairs: &[(Value16, Value16)],
    method: &str,
    args: &[Value16],
) -> SharedResult<Value16> {
    match method {
        "set" => {
            if args.len() < 2 {
                return Err(runtime_error("map.set requires 2 arguments (key, value)"));
            }
            let mut new_pairs = pairs.to_vec();
            if let Some(pos) = new_pairs.iter().position(|(k, _)| k.values_equal(&args[0])) {
                new_pairs[pos].1 = args[1].clone();
            } else {
                new_pairs.push((args[0].clone(), args[1].clone()));
            }
            Ok(Value16::map(new_pairs))
        }
        "get" => {
            let key = args
                .first()
                .ok_or_else(|| runtime_error("map.get requires 1 argument (key)"))?;
            let result = pairs
                .iter()
                .find(|(k, _)| k.values_equal(key))
                .map(|(_, v)| v.clone())
                .unwrap_or_else(Value16::null);
            Ok(result)
        }
        "delete" | "remove" => {
            let key = args
                .first()
                .ok_or_else(|| runtime_error("map.delete requires 1 argument (key)"))?;
            let new_pairs: Vec<(Value16, Value16)> = pairs
                .iter()
                .filter(|(k, _)| !k.values_equal(key))
                .cloned()
                .collect();
            Ok(Value16::map(new_pairs))
        }
        "has" | "contains" => {
            let key = args
                .first()
                .ok_or_else(|| runtime_error("map.has requires 1 argument (key)"))?;
            Ok(Value16::boolean(
                pairs.iter().any(|(k, _)| k.values_equal(key)),
            ))
        }
        "keys" => {
            let keys: Vec<Value16> = pairs.iter().map(|(k, _)| k.clone()).collect();
            Ok(Value16::array(keys))
        }
        "values" => {
            let values: Vec<Value16> = pairs.iter().map(|(_, v)| v.clone()).collect();
            Ok(Value16::array(values))
        }
        "entries" => {
            let entries: Vec<Value16> = pairs
                .iter()
                .map(|(k, v)| Value16::array(vec![k.clone(), v.clone()]))
                .collect();
            Ok(Value16::array(entries))
        }
        "size" | "length" | "len" => Ok(Value16::number(pairs.len() as f64)),
        "clear" => Ok(Value16::map(Vec::new())),
        _ => Err(runtime_error(format!("Unknown method '{}' on Map", method))),
    }
}
