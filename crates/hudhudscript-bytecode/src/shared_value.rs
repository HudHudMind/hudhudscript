use crate::error::CompileResult;
use crate::Value16;
use std::collections::HashMap;

// ── Shared inline arithmetic (Kural 7 — single source for VM) ──

/// Shared inline arithmetic — used by VM.
/// Zero overhead: #[inline(always)] ensures no function-call cost.
#[inline(always)]
#[inline(always)]
pub fn num_add(a: f64, b: f64) -> f64 {
    a + b
}
#[inline(always)]
#[inline(always)]
pub fn num_sub(a: f64, b: f64) -> f64 {
    a - b
}
#[inline(always)]
#[inline(always)]
pub fn num_mul(a: f64, b: f64) -> f64 {
    a * b
}
#[inline(always)]
#[inline(always)]
pub fn num_div(a: f64, b: f64) -> f64 {
    a / b
}
#[inline(always)]
#[inline(always)]
pub fn num_mod(a: f64, b: f64) -> f64 {
    a % b
}
#[inline(always)]
#[inline(always)]
pub fn num_neg(a: f64) -> f64 {
    -a
}
#[inline(always)]
#[inline(always)]
pub fn num_eq(a: f64, b: f64) -> bool {
    a == b
}
#[inline(always)]
pub fn num_ne(a: f64, b: f64) -> bool {
    a != b
}
#[inline(always)]
#[inline(always)]
pub fn num_lt(a: f64, b: f64) -> bool {
    a < b
}
#[inline(always)]
#[inline(always)]
pub fn num_le(a: f64, b: f64) -> bool {
    a <= b
}
#[inline(always)]
#[inline(always)]
pub fn num_gt(a: f64, b: f64) -> bool {
    a > b
}
#[inline(always)]
#[inline(always)]
pub fn num_ge(a: f64, b: f64) -> bool {
    a >= b
}

/// Shared error type — both RuntimeError and CompileError are this.
pub type SharedError = hudhudscript_errors::Error;
pub type SharedResult<T> = hudhudscript_errors::HudHudResult<T>;

/// Helper to build a runtime error from a message.
pub fn runtime_error(msg: impl Into<String>) -> SharedError {
    hudhudscript_errors::Error::new(
        hudhudscript_errors::ErrorCode::CompileRuntimeError,
        msg.into(),
    )
}

/// Helper to build a type error.
pub fn type_error(expected: &str, got: &str, context: &str) -> SharedError {
    hudhudscript_errors::Error::new(
        hudhudscript_errors::ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// Type error with the same message format and context fields as
/// `runtime_codes::type_error`.
pub fn type_error_ctx(
    expected: impl Into<String>,
    found: impl Into<String>,
    operation: impl Into<String>,
) -> SharedError {
    let expected = expected.into();
    let found = found.into();
    let operation = operation.into();
    hudhudscript_errors::Error::new(
        hudhudscript_errors::ErrorCode::RuntimeTypeError,
        format!(
            "Type error in {}: expected {}, found {}",
            operation, expected, found
        ),
    )
    .with_context("expected", expected)
    .with_context("found", found)
    .with_context("operation", operation)
}

/// Division-by-zero error.
pub fn division_by_zero() -> SharedError {
    hudhudscript_errors::Error::new(
        hudhudscript_errors::ErrorCode::RuntimeDivisionByZero,
        "Division by zero",
    )
}

/// Call error (invalid argument count, bad argument type, etc.).
pub fn call_error(message: impl Into<String>, callee: impl Into<String>) -> SharedError {
    let message = message.into();
    let callee = callee.into();
    hudhudscript_errors::Error::new(
        hudhudscript_errors::ErrorCode::RuntimeCallError,
        format!("Call error on {}: {}", callee, message),
    )
    .with_context("callee", callee)
    .with_context("message", message)
}

/// Index-out-of-bounds error — identical to `runtime_codes::index_out_of_bounds`.
pub fn index_out_of_bounds(index: i64, length: usize) -> SharedError {
    hudhudscript_errors::Error::new(
        hudhudscript_errors::ErrorCode::RuntimeIndexOutOfBounds,
        format!("Index out of bounds: {} (length: {})", index, length),
    )
    .with_context("index", index.to_string())
    .with_context("length", length.to_string())
}

/// Property-not-found error — identical to `runtime_codes::property_not_found`.
pub fn property_not_found(
    property: impl Into<String>,
    object_type: impl Into<String>,
) -> SharedError {
    let property = property.into();
    let object_type = object_type.into();
    hudhudscript_errors::Error::new(
        hudhudscript_errors::ErrorCode::RuntimePropertyNotFound,
        format!("Property '{}' not found on {}", property, object_type),
    )
    .with_context("property", property)
    .with_context("object_type", object_type)
}

/// Construct error object fields from class name and arguments (Issue #1016).
///
/// Supports the JS-compatible `new Error("message", { cause: e })` pattern.
/// Returns a `HashMap` of fields to set on the error instance.
pub fn construct_error_fields(class_name: &str, args: &[Value16]) -> HashMap<String, Value16> {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Value16::string(class_name.to_string()));

    // First argument: message
    if let Some(msg) = args.first() {
        fields.insert("message".to_string(), msg.clone());
    } else {
        fields.insert("message".to_string(), Value16::string(String::new()));
    }

    // Second argument: options object { cause: ... }
    if let Some(options) = args.get(1) {
        if let Some(obj) = options.as_object() {
            if let Some(cause) = obj.get("cause") {
                fields.insert("cause".to_string(), cause.clone());
            }
        }
    }

    fields.insert(
        "stack".to_string(),
        Value16::string(format!("{} at <runtime>", class_name)),
    );

    fields
}

/// Shared number formatting: displays integers without decimal point,
/// floats normally. Replaces duplicated logic across json, csv, ini,
/// value_to_string, etc.
pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

/// Shared numeric comparison — matches interpreter `eval_comparison`
/// semantics: if both operands aren't numbers, return `false` (mixed-type
/// → false, JS-like). Used by VM for `<`, `<=`, `>`, `>=`.
pub fn shared_compare(left: &Value16, right: &Value16, op: fn(f64, f64) -> bool) -> Value16 {
    if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
        Value16::boolean(op(a, b))
    } else {
        Value16::boolean(false)
    }
}

/// Shared `+` operator — same semantics as JavaScript with strict rules
/// for non-primitive coercion (#745).
///
/// - Number + Number → Number (sum)
/// - String + String → String (concat)
/// - String + Number or Number + String → String (numeric-to-string coerce)
/// - Anything else → TypeError
pub fn shared_add(left: &Value16, right: &Value16) -> CompileResult<Value16> {
    if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
        return Ok(Value16::number(num_add(a, b)));
    }
    if let (Some(a), Some(b)) = (left.as_str(), right.as_str()) {
        let needed = a.len() + b.len();
        let cap = needed.max(a.len().saturating_mul(2));
        let mut s = String::with_capacity(cap);
        s.push_str(a);
        s.push_str(b);
        return Ok(Value16::string(s));
    }
    if let (Some(a), Some(b)) = (left.as_str(), right.as_number()) {
        let b_str = format_number(b);
        let mut s = String::with_capacity(a.len() + b_str.len());
        s.push_str(a);
        s.push_str(&b_str);
        return Ok(Value16::string(s));
    }
    if let (Some(a), Some(b)) = (left.as_number(), right.as_str()) {
        let a_str = format_number(a);
        let mut s = String::with_capacity(a_str.len() + b.len());
        s.push_str(&a_str);
        s.push_str(b);
        return Ok(Value16::string(s));
    }
    // Fix #4: Array + Array concatenation (array_sum benchmark)
    if let (Some(a), Some(b)) = (left.as_array(), right.as_array()) {
        let mut result = a.clone();
        result.extend_from_slice(b);
        return Ok(Value16::array(result));
    }
    // #745: String + Boolean is a TypeError — no auto-coercion.
    if left.as_str().is_some() || right.as_str().is_some() {
        return Err(type_error("String or Number", right.type_name_str(), "+"));
    }
    if right.as_str().is_some() {
        return Err(type_error("String or Number", left.type_name_str(), "+"));
    }
    Err(type_error(
        "number or string",
        &format!("{} and {}", left.type_name_str(), right.type_name_str()),
        "addition",
    ))
}

/// Trait for invoking callback functions — abstracts over VM call mechanisms.
/// Shared builtin code (e.g. array map/filter) calls back into the runtime.
pub trait CallbackInvoker {
    /// Invoke `callback` with the given arguments and return the result.
    fn invoke(&mut self, callback: &Value16, args: Vec<Value16>) -> SharedResult<Value16>;

    /// Check if a value is truthy (convenience).
    fn is_truthy_value(&self, val: &Value16) -> bool {
        val.is_truthy()
    }
}
