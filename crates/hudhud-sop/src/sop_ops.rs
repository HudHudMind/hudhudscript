//! Subject-Oriented Programming (SOP) shared utilities
//!
//! Provides shared logic for SOP features that must work identically
//! in both the interpreter and VM runtimes (Kural 2 + Kural 7).

/// Check that a class implements all methods required by a trait.
///
/// Returns `Ok(())` if all required methods are present, or
/// `Err(missing_methods)` with the list of methods not found.
///
/// Both the interpreter and VM call this function to enforce
/// `class Foo implements Bar` declarations at runtime.
pub fn check_trait_implementation(
    _class_name: &str,
    _trait_name: &str,
    required_methods: &[String],
    class_methods: &[String],
) -> Result<(), Vec<String>> {
    let missing: Vec<String> = required_methods
        .iter()
        .filter(|m| !class_methods.contains(m))
        .cloned()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

/// Format a trait implementation error message.
///
/// Single source of truth for the error message format (Kural 7).
pub fn trait_not_implemented_error(
    class_name: &str,
    trait_name: &str,
    missing_methods: &[String],
) -> String {
    format!(
        "Class '{}' does not fully implement trait '{}': missing method(s): {}",
        class_name,
        trait_name,
        missing_methods.join(", ")
    )
}
