//! Immediate array method implementations.
//!
//! Callback-dependent methods are owned by the VM continuation state machine
//! in `call_state::array_callback`; this module never executes user code.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

/// Execute a non-callback array method. Returns `None` if the method
/// requires a callback (so the caller can handle it runtime-specifically).
#[inline(always)]
pub fn call_array_method(
    arr: &[Value16],
    method: &str,
    args: &[Value16],
) -> Option<SharedResult<Value16>> {
    match method {
        "length" => Some(Ok(Value16::int(arr.len() as i64))),

        "push" => {
            let mut new_arr = arr.to_vec();
            for arg in args {
                new_arr.push(arg.clone());
            }
            Some(Ok(Value16::array(new_arr)))
        }

        "pop" => {
            // Interpreter parity: popping from empty array is an error.
            if arr.is_empty() {
                return Some(Err(runtime_error("Cannot pop from empty array")));
            }
            let mut new_arr = arr.to_vec();
            let popped = new_arr.pop().unwrap_or_else(Value16::null);
            Some(Ok(popped))
        }

        "shift" => {
            if arr.is_empty() {
                Some(Ok(Value16::null()))
            } else {
                Some(Ok(arr[0].clone()))
            }
        }

        "unshift" => {
            let mut new_arr = args.to_vec();
            new_arr.extend(arr.iter().cloned());
            Some(Ok(Value16::array(new_arr)))
        }

        "concat" => {
            let mut new_arr = arr.to_vec();
            for arg in args {
                if let Some(other) = arg.as_array() {
                    new_arr.extend(other.iter().cloned());
                }
            }
            Some(Ok(Value16::array(new_arr)))
        }

        "join" => {
            let delimiter = args.first().and_then(|v| v.as_str()).unwrap_or(",");
            let sep_len = delimiter.len();
            let mut total = sep_len * arr.len().saturating_sub(1);
            for v in arr.iter() {
                total += match v.as_str() {
                    Some(st) => st.len(),
                    None => 8,
                };
            }
            let mut s = String::with_capacity(total);
            let mut first = true;
            for v in arr {
                if !first {
                    s.push_str(delimiter);
                }
                first = false;
                match v.as_str() {
                    Some(st) => s.push_str(st),
                    None => s.push_str(&v.display_string()),
                }
            }
            Some(Ok(Value16::string(s)))
        }

        "slice" => {
            let start = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
            let end = args
                .get(1)
                .and_then(|v| v.as_number())
                .map(|n| n as usize)
                .unwrap_or(arr.len());
            Some(Ok(Value16::array(arr[start..end.min(arr.len())].to_vec())))
        }

        "concat" => {
            let mut result = arr.to_vec();
            for arg in args {
                if let Some(other) = arg.as_array() {
                    result.extend(other.iter().cloned());
                } else {
                    result.push(arg.clone());
                }
            }
            Some(Ok(Value16::array(result)))
        }

        "reverse" => {
            let mut new_arr = arr.to_vec();
            new_arr.reverse();
            Some(Ok(Value16::array(new_arr)))
        }

        "flat" => {
            let mut result = Vec::new();
            for item in arr {
                if let Some(inner) = item.as_array() {
                    result.extend(inner.iter().cloned());
                } else {
                    result.push(item.clone());
                }
            }
            Some(Ok(Value16::array(result)))
        }

        // Equality-based lookup methods — no callback required, use the
        // SharedValue::values_equal default trait method which every
        // runtime's Value type already implements via the shared trait.
        "indexOf" | "index_of" => {
            let needle = match args.first() {
                Some(v) => v,
                None => return Some(Err(runtime_error("indexOf requires an argument"))),
            };
            let idx = arr
                .iter()
                .position(|v| v.values_equal(needle))
                .map(|i| i as f64)
                .unwrap_or(-1.0);
            Some(Ok(Value16::number(idx)))
        }
        "contains" | "includes" => {
            let needle = match args.first() {
                Some(v) => v,
                None => return Some(Err(runtime_error("contains requires an argument"))),
            };
            Some(Ok(Value16::boolean(
                arr.iter().any(|v| v.values_equal(needle)),
            )))
        }

        // Callback-dependent methods are scheduled by the VM continuation
        // lane. `sort` and `fill` remain unsupported here as before.
        "map" | "filter" | "reduce" | "forEach" | "find" | "some" | "every" | "sort" | "fill" => {
            None
        }

        _ => Some(Err(runtime_error(format!(
            "Unknown array method: {}",
            method
        )))),
    }
}
