use hudhudscript_errors::{Error, ErrorCode};

/// STM max-retries error (RuntimeStmMaxRetriesExceeded).
pub fn err_max_retries_exceeded(retries: usize) -> Error {
    Error::new(
        ErrorCode::RuntimeStmMaxRetriesExceeded,
        format!(
            "STM transaction failed after {} retries (livelock?)",
            retries
        ),
    )
    .with_context("retries", retries.to_string())
}

/// STM timeout error (RuntimeStmTimeout).
pub fn err_timeout(timeout_ms: u64, elapsed_ms: u64) -> Error {
    Error::new(
        ErrorCode::RuntimeStmTimeout,
        format!(
            "STM transaction timed out after {}ms (limit: {}ms)",
            elapsed_ms, timeout_ms
        ),
    )
    .with_context("timeout_ms", timeout_ms.to_string())
    .with_context("elapsed_ms", elapsed_ms.to_string())
}

/// Generic runtime error for TVar-not-found and similar STM helpers.
pub fn err_tvar_not_found(id: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeInvalidOperation,
        format!("TVar '{}' not found", id),
    )
    .with_context("tvar_id", id.to_string())
}
