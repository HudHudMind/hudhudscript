//! Assertion functions for the HudHudScript testing framework.
//!
//! Provides a rich set of assertion utilities that produce clear, descriptive
//! error messages when a check fails.

use std::fmt::Debug;

/// Assertion error with structured context.
#[derive(Debug, thiserror::Error)]
#[error("Assertion failed: {message}")]
pub struct AssertionError {
    pub message: String,
    /// Optional label supplied by the test author for extra context.
    pub label: Option<String>,
    /// Source location hint (file:line) when available.
    pub location: Option<String>,
}

impl AssertionError {
    /// Create a simple assertion error from a message string.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            label: None,
            location: None,
        }
    }

    /// Attach an optional user-supplied label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Attach an optional source location hint.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Produce a human-readable description including optional context.
    pub fn display_message(&self) -> String {
        let mut parts = Vec::new();
        if let Some(loc) = &self.location {
            parts.push(format!("at {loc}"));
        }
        if let Some(lbl) = &self.label {
            parts.push(format!("[{lbl}]"));
        }
        parts.push(format!("Assertion failed: {}", self.message));
        parts.join(" ")
    }
}

/// Assertion utilities.
///
/// Every method returns `Ok(())` when the assertion holds or
/// `Err(AssertionError)` when it does not.
pub struct Assertion;

impl Assertion {
    // ── Equality ──────────────────────────────────────────────────────

    /// Assert that `actual == expected`.
    pub fn equal<T: PartialEq + Debug>(actual: T, expected: T) -> Result<(), AssertionError> {
        if actual == expected {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {:?}, got {:?}",
                expected, actual
            )))
        }
    }

    /// Assert that `actual != expected`.
    pub fn not_equal<T: PartialEq + Debug>(actual: T, expected: T) -> Result<(), AssertionError> {
        if actual != expected {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected value to differ from {:?}",
                expected
            )))
        }
    }

    // ── Boolean ───────────────────────────────────────────────────────

    /// Assert that `value` is `true`.
    pub fn is_true(value: bool) -> Result<(), AssertionError> {
        if value {
            Ok(())
        } else {
            Err(AssertionError::new("Expected true, got false"))
        }
    }

    /// Assert that `value` is `false`.
    pub fn is_false(value: bool) -> Result<(), AssertionError> {
        if !value {
            Ok(())
        } else {
            Err(AssertionError::new("Expected false, got true"))
        }
    }

    // ── Nullity / Option ──────────────────────────────────────────────

    /// Assert that the option is `None`.
    pub fn is_none<T: Debug>(value: &Option<T>) -> Result<(), AssertionError> {
        match value {
            None => Ok(()),
            Some(v) => Err(AssertionError::new(format!(
                "Expected None, got Some({:?})",
                v
            ))),
        }
    }

    /// Assert that the option is `Some(_)`.
    pub fn is_some<T: Debug>(value: &Option<T>) -> Result<(), AssertionError> {
        match value {
            Some(_) => Ok(()),
            None => Err(AssertionError::new("Expected Some(_), got None")),
        }
    }

    // ── Ordering ──────────────────────────────────────────────────────

    /// Assert `actual > expected`.
    pub fn greater_than<T: PartialOrd + Debug>(
        actual: T,
        expected: T,
    ) -> Result<(), AssertionError> {
        if actual > expected {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {:?} > {:?}",
                actual, expected
            )))
        }
    }

    /// Assert `actual < expected`.
    pub fn less_than<T: PartialOrd + Debug>(actual: T, expected: T) -> Result<(), AssertionError> {
        if actual < expected {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {:?} < {:?}",
                actual, expected
            )))
        }
    }

    /// Assert `actual >= expected`.
    pub fn greater_than_or_equal<T: PartialOrd + Debug>(
        actual: T,
        expected: T,
    ) -> Result<(), AssertionError> {
        if actual >= expected {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {:?} >= {:?}",
                actual, expected
            )))
        }
    }

    /// Assert `actual <= expected`.
    pub fn less_than_or_equal<T: PartialOrd + Debug>(
        actual: T,
        expected: T,
    ) -> Result<(), AssertionError> {
        if actual <= expected {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {:?} <= {:?}",
                actual, expected
            )))
        }
    }

    // ── Containment ───────────────────────────────────────────────────

    /// Assert that `haystack` contains `needle` (string containment).
    pub fn contains(haystack: &str, needle: &str) -> Result<(), AssertionError> {
        if haystack.contains(needle) {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected string to contain {:?}, got {:?}",
                needle, haystack
            )))
        }
    }

    /// Assert that `haystack` does NOT contain `needle`.
    pub fn not_contains(haystack: &str, needle: &str) -> Result<(), AssertionError> {
        if !haystack.contains(needle) {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected string NOT to contain {:?}, got {:?}",
                needle, haystack
            )))
        }
    }

    /// Assert that `haystack` starts with `prefix`.
    pub fn starts_with(haystack: &str, prefix: &str) -> Result<(), AssertionError> {
        if haystack.starts_with(prefix) {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {:?} to start with {:?}",
                haystack, prefix
            )))
        }
    }

    /// Assert that `haystack` ends with `suffix`.
    pub fn ends_with(haystack: &str, suffix: &str) -> Result<(), AssertionError> {
        if haystack.ends_with(suffix) {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {:?} to end with {:?}",
                haystack, suffix
            )))
        }
    }

    /// Assert that a slice/vec contains the given element.
    pub fn collection_contains<T: PartialEq + Debug>(
        collection: &[T],
        element: &T,
    ) -> Result<(), AssertionError> {
        if collection.contains(element) {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected collection to contain {:?}",
                element
            )))
        }
    }

    // ── Error / Panic ─────────────────────────────────────────────────

    /// Assert that the closure returns `Err(_)`.
    pub fn throws<T: Debug, E: Debug>(result: &Result<T, E>) -> Result<(), AssertionError> {
        match result {
            Err(_) => Ok(()),
            Ok(val) => Err(AssertionError::new(format!(
                "Expected an error, but got Ok({:?})",
                val
            ))),
        }
    }

    /// Assert that the closure returns `Err` whose debug representation
    /// contains `expected_msg`.
    pub fn throws_with_message<T: Debug, E: Debug>(
        result: &Result<T, E>,
        expected_msg: &str,
    ) -> Result<(), AssertionError> {
        match result {
            Err(e) => {
                let msg = format!("{:?}", e);
                if msg.contains(expected_msg) {
                    Ok(())
                } else {
                    Err(AssertionError::new(format!(
                        "Expected error containing {:?}, got {:?}",
                        expected_msg, msg
                    )))
                }
            }
            Ok(val) => Err(AssertionError::new(format!(
                "Expected an error containing {:?}, but got Ok({:?})",
                expected_msg, val
            ))),
        }
    }

    // ── Length ─────────────────────────────────────────────────────────

    /// Assert that a slice has the expected length.
    pub fn has_length<T>(collection: &[T], expected: usize) -> Result<(), AssertionError> {
        if collection.len() == expected {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected length {}, got {}",
                expected,
                collection.len()
            )))
        }
    }

    /// Assert that a slice is empty.
    pub fn is_empty<T>(collection: &[T]) -> Result<(), AssertionError> {
        if collection.is_empty() {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected empty collection, got {} element(s)",
                collection.len()
            )))
        }
    }

    /// Assert that a slice is NOT empty.
    pub fn is_not_empty<T>(collection: &[T]) -> Result<(), AssertionError> {
        if !collection.is_empty() {
            Ok(())
        } else {
            Err(AssertionError::new("Expected non-empty collection"))
        }
    }

    // ── Approximate equality ──────────────────────────────────────────

    /// Assert that two f64 values are approximately equal within `epsilon`.
    pub fn approx_equal(actual: f64, expected: f64, epsilon: f64) -> Result<(), AssertionError> {
        if (actual - expected).abs() <= epsilon {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {} to be approximately {} (epsilon={}), difference={}",
                actual,
                expected,
                epsilon,
                (actual - expected).abs()
            )))
        }
    }

    // ── Pattern / Regex (string) ──────────────────────────────────────

    /// Assert that a string matches the given pattern (simple `contains` check).
    /// For full regex support a dedicated dependency would be required; this
    /// helper keeps the crate dependency-light.
    pub fn matches_pattern(value: &str, pattern: &str) -> Result<(), AssertionError> {
        if value.contains(pattern) {
            Ok(())
        } else {
            Err(AssertionError::new(format!(
                "Expected {:?} to match pattern {:?}",
                value, pattern
            )))
        }
    }
}
