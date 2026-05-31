//! Shared array method implementations — used by both VM and interpreter.
//!
//! Non-callback methods are handled directly. Callback-dependent methods
//! (map, filter, reduce, forEach, find, some, every) use the `CallbackInvoker`
//! trait to abstract over VM vs interpreter call mechanisms.

use hudhudscript_bytecode::shared_value::{runtime_error, CallbackInvoker, SharedResult};
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
                if !first { s.push_str(delimiter); }
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

        // Callback-dependent methods — return None so the caller can use
        // `call_array_method_with_callback` instead.
        "map" | "filter" | "reduce" | "forEach" | "find" | "some" | "every" | "sort" | "fill" => {
            None
        }

        _ => Some(Err(runtime_error(format!(
            "Unknown array method: {}",
            method
        )))),
    }
}

/// Execute a callback-dependent array method. The `invoker` provides the
/// runtime-specific mechanism for calling HudHudScript functions.
///
/// Returns `None` if the method is not a callback method (caller should
/// try `call_array_method` first).
#[inline(always)]
pub fn call_array_method_with_callback(
    arr: &[Value16],
    method: &str,
    args: &[Value16],
    invoker: &mut impl CallbackInvoker,
) -> Option<SharedResult<Value16>> {
    match method {
        "map" => {
            let callback = match args.first() {
                Some(cb) => cb,
                None => return Some(Err(runtime_error("map requires a callback argument"))),
            };
            let mut result = Vec::with_capacity(arr.len());
            for (i, item) in arr.iter().enumerate() {
                match invoker.invoke(callback, vec![item.clone(), Value16::number(i as f64)]) {
                    Ok(val) => result.push(val),
                    Err(e) => return Some(Err(e)),
                }
            }
            Some(Ok(Value16::array(result)))
        }

        "filter" => {
            let callback = match args.first() {
                Some(cb) => cb,
                None => return Some(Err(runtime_error("filter requires a callback argument"))),
            };
            let mut result = Vec::new();
            for (i, item) in arr.iter().enumerate() {
                match invoker.invoke(callback, vec![item.clone(), Value16::number(i as f64)]) {
                    Ok(val) => {
                        if invoker.is_truthy_value(&val) {
                            result.push(item.clone());
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
            Some(Ok(Value16::array(result)))
        }

        "reduce" => {
            let callback = match args.first() {
                Some(cb) => cb,
                None => return Some(Err(runtime_error("reduce requires a callback argument"))),
            };
            let mut iter = arr.iter().enumerate();
            let mut accumulator = if args.len() > 1 {
                args[1].clone()
            } else {
                match iter.next() {
                    Some((_, first)) => first.clone(),
                    None => {
                        return Some(Err(runtime_error(
                            "reduce of empty array with no initial value",
                        )))
                    }
                }
            };
            for (i, item) in iter {
                match invoker.invoke(
                    callback,
                    vec![accumulator, item.clone(), Value16::number(i as f64)],
                ) {
                    Ok(val) => accumulator = val,
                    Err(e) => return Some(Err(e)),
                }
            }
            Some(Ok(accumulator))
        }

        "forEach" => {
            let callback = match args.first() {
                Some(cb) => cb,
                None => return Some(Err(runtime_error("forEach requires a callback argument"))),
            };
            for (i, item) in arr.iter().enumerate() {
                if let Err(e) =
                    invoker.invoke(callback, vec![item.clone(), Value16::number(i as f64)])
                {
                    return Some(Err(e));
                }
            }
            Some(Ok(Value16::null()))
        }

        "find" => {
            let callback = match args.first() {
                Some(cb) => cb,
                None => return Some(Err(runtime_error("find requires a callback argument"))),
            };
            for (i, item) in arr.iter().enumerate() {
                match invoker.invoke(callback, vec![item.clone(), Value16::number(i as f64)]) {
                    Ok(val) => {
                        if invoker.is_truthy_value(&val) {
                            return Some(Ok(item.clone()));
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
            Some(Ok(Value16::null()))
        }

        "some" => {
            let callback = match args.first() {
                Some(cb) => cb,
                None => return Some(Err(runtime_error("some requires a callback argument"))),
            };
            for (i, item) in arr.iter().enumerate() {
                match invoker.invoke(callback, vec![item.clone(), Value16::number(i as f64)]) {
                    Ok(val) => {
                        if invoker.is_truthy_value(&val) {
                            return Some(Ok(Value16::boolean(true)));
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
            Some(Ok(Value16::boolean(false)))
        }

        "every" => {
            let callback = match args.first() {
                Some(cb) => cb,
                None => return Some(Err(runtime_error("every requires a callback argument"))),
            };
            for (i, item) in arr.iter().enumerate() {
                match invoker.invoke(callback, vec![item.clone(), Value16::number(i as f64)]) {
                    Ok(val) => {
                        if !invoker.is_truthy_value(&val) {
                            return Some(Ok(Value16::boolean(false)));
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
            Some(Ok(Value16::boolean(true)))
        }

        _ => None,
    }
}
