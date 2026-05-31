use crate::{Error, ErrorCode};

/// Helper to build a runtime error from a message.
pub fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode(38), msg.into())
}

/// Helper to build a type error.
pub fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode(250),
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// Type error with context fields matching the runtime_codes format.
pub fn type_error_ctx(
    expected: impl Into<String>,
    found: impl Into<String>,
    operation: impl Into<String>,
) -> Error {
    let expected = expected.into();
    let found = found.into();
    let operation = operation.into();
    Error::new(
        ErrorCode(250),
        format!(
            "Type error in {}: expected {}, found {}",
            operation, expected, found
        ),
    )
}

/// Division by zero error.
pub fn division_by_zero() -> Error {
    Error::new(ErrorCode(230), "division by zero")
}

/// Call error with message and callee name.
pub fn call_error(message: impl Into<String>, callee: impl Into<String>) -> Error {
    let message = message.into();
    let callee = callee.into();
    Error::new(ErrorCode(38), format!("{} (calling {})", message, callee))
}

/// Index out of bounds error.
pub fn index_out_of_bounds(index: i64, length: usize) -> Error {
    Error::new(
        ErrorCode(234),
        format!("index {} out of bounds for length {}", index, length),
    )
}

/// Property not found error.
pub fn property_not_found(property: impl Into<String>, object_type: impl Into<String>) -> Error {
    let property = property.into();
    let object_type = object_type.into();
    Error::new(
        ErrorCode(239),
        format!("Property '{}' not found on {}", property, object_type),
    )
    .with_context("property", property)
    .with_context("object_type", object_type)
}
