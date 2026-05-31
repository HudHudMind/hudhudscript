//! ID validation and sanitization for governance structures.
//!
//! This module provides validation functions for constitution, law, and rule IDs,
//! ensuring they match the required regex patterns and are sanitized to prevent
//! injection attacks.

use regex::Regex;
use std::sync::OnceLock;

// ============================================================================
// Regex Patterns
// ============================================================================

/// Regex pattern for constitution IDs: ^cons\.\d+$
/// Examples: cons.1, cons.2, cons.999
static CONSTITUTION_ID_REGEX: OnceLock<Regex> = OnceLock::new();

/// Regex pattern for law IDs: ^cons\d+\.law\d+$
/// Examples: cons1.law1, cons2.law3, cons999.law123
static LAW_ID_REGEX: OnceLock<Regex> = OnceLock::new();

/// Regex pattern for rule IDs: ^rule\.\d+$
/// Examples: rule.1, rule.2, rule.999
static RULE_ID_REGEX: OnceLock<Regex> = OnceLock::new();

// ============================================================================
// Validation Functions
// ============================================================================

/// Validates a constitution ID against the pattern ^cons\.\d+$
///
/// # Arguments
/// * `id` - The constitution ID to validate
///
/// # Returns
/// * `true` if the ID matches the pattern, `false` otherwise
///
/// # Examples
/// ```
/// use hudhudscript_ids::validator::validate_constitution_id;
///
/// assert!(validate_constitution_id("cons.1"));
/// assert!(validate_constitution_id("cons.999"));
/// assert!(!validate_constitution_id("cons1"));
/// assert!(!validate_constitution_id("cons."));
/// assert!(!validate_constitution_id("constitution.1"));
/// ```
pub fn validate_constitution_id(id: &str) -> bool {
    let regex = CONSTITUTION_ID_REGEX.get_or_init(|| {
        Regex::new(r"^cons\.\d+$").expect("Failed to compile constitution ID regex")
    });
    regex.is_match(id)
}

/// Validates a law ID against the pattern ^cons\d+\.law\d+$
///
/// # Arguments
/// * `id` - The law ID to validate
///
/// # Returns
/// * `true` if the ID matches the pattern, `false` otherwise
///
/// # Examples
/// ```
/// use hudhudscript_ids::validator::validate_law_id;
///
/// assert!(validate_law_id("cons1.law1"));
/// assert!(validate_law_id("cons999.law123"));
/// assert!(!validate_law_id("cons.1.law1"));
/// assert!(!validate_law_id("cons1.law"));
/// assert!(!validate_law_id("law1"));
/// ```
pub fn validate_law_id(id: &str) -> bool {
    let regex = LAW_ID_REGEX
        .get_or_init(|| Regex::new(r"^cons\d+\.law\d+$").expect("Failed to compile law ID regex"));
    regex.is_match(id)
}

/// Validates a rule ID against the pattern ^rule\.\d+$
///
/// # Arguments
/// * `id` - The rule ID to validate
///
/// # Returns
/// * `true` if the ID matches the pattern, `false` otherwise
///
/// # Examples
/// ```
/// use hudhudscript_ids::validator::validate_rule_id;
///
/// assert!(validate_rule_id("rule.1"));
/// assert!(validate_rule_id("rule.999"));
/// assert!(!validate_rule_id("rule1"));
/// assert!(!validate_rule_id("rule."));
/// assert!(!validate_rule_id("rules.1"));
/// ```
pub fn validate_rule_id(id: &str) -> bool {
    let regex = RULE_ID_REGEX
        .get_or_init(|| Regex::new(r"^rule\.\d+$").expect("Failed to compile rule ID regex"));
    regex.is_match(id)
}

// ============================================================================
// Sanitization Functions
// ============================================================================

/// Sanitizes an ID string to prevent injection attacks.
///
/// This function removes or escapes potentially dangerous characters:
/// - Removes null bytes
/// - Removes control characters (ASCII 0-31 and 127)
/// - Removes Unicode control characters
/// - Trims whitespace from start and end
/// - Limits length to 256 characters
///
/// # Arguments
/// * `id` - The ID string to sanitize
///
/// # Returns
/// * A sanitized version of the ID string
///
/// # Examples
/// ```
/// use hudhudscript_ids::validator::sanitize_id;
///
/// assert_eq!(sanitize_id("cons.1"), "cons.1");
/// assert_eq!(sanitize_id("  cons.1  "), "cons.1");
/// assert_eq!(sanitize_id("cons\x00.1"), "cons.1");
/// assert_eq!(sanitize_id("cons\n.1"), "cons.1");
/// ```
pub fn sanitize_id(id: &str) -> String {
    id.chars()
        // Remove null bytes
        .filter(|c| *c != '\0')
        // Remove all control characters (ASCII 0-31 and 127)
        .filter(|c| !c.is_control())
        // Remove Unicode control characters
        .filter(|c| !matches!(c, '\u{0080}'..='\u{009F}'))
        .collect::<String>()
        .trim()
        .chars()
        .take(256) // Limit length to prevent DoS
        .collect()
}

/// Sanitizes and validates a constitution ID.
///
/// This is a convenience function that combines sanitization and validation.
///
/// # Arguments
/// * `id` - The constitution ID to sanitize and validate
///
/// # Returns
/// * `Some(sanitized_id)` if the sanitized ID is valid, `None` otherwise
///
/// # Examples
/// ```
/// use hudhudscript_ids::validator::sanitize_and_validate_constitution_id;
///
/// assert_eq!(sanitize_and_validate_constitution_id("cons.1"), Some("cons.1".to_string()));
/// assert_eq!(sanitize_and_validate_constitution_id("  cons.1  "), Some("cons.1".to_string()));
/// assert_eq!(sanitize_and_validate_constitution_id("invalid"), None);
/// ```
pub fn sanitize_and_validate_constitution_id(id: &str) -> Option<String> {
    let sanitized = sanitize_id(id);
    if validate_constitution_id(&sanitized) {
        Some(sanitized)
    } else {
        None
    }
}

/// Sanitizes and validates a law ID.
///
/// This is a convenience function that combines sanitization and validation.
///
/// # Arguments
/// * `id` - The law ID to sanitize and validate
///
/// # Returns
/// * `Some(sanitized_id)` if the sanitized ID is valid, `None` otherwise
///
/// # Examples
/// ```
/// use hudhudscript_ids::validator::sanitize_and_validate_law_id;
///
/// assert_eq!(sanitize_and_validate_law_id("cons1.law1"), Some("cons1.law1".to_string()));
/// assert_eq!(sanitize_and_validate_law_id("  cons1.law1  "), Some("cons1.law1".to_string()));
/// assert_eq!(sanitize_and_validate_law_id("invalid"), None);
/// ```
pub fn sanitize_and_validate_law_id(id: &str) -> Option<String> {
    let sanitized = sanitize_id(id);
    if validate_law_id(&sanitized) {
        Some(sanitized)
    } else {
        None
    }
}

/// Sanitizes and validates a rule ID.
///
/// This is a convenience function that combines sanitization and validation.
///
/// # Arguments
/// * `id` - The rule ID to sanitize and validate
///
/// # Returns
/// * `Some(sanitized_id)` if the sanitized ID is valid, `None` otherwise
///
/// # Examples
/// ```
/// use hudhudscript_ids::validator::sanitize_and_validate_rule_id;
///
/// assert_eq!(sanitize_and_validate_rule_id("rule.1"), Some("rule.1".to_string()));
/// assert_eq!(sanitize_and_validate_rule_id("  rule.1  "), Some("rule.1".to_string()));
/// assert_eq!(sanitize_and_validate_rule_id("invalid"), None);
/// ```
pub fn sanitize_and_validate_rule_id(id: &str) -> Option<String> {
    let sanitized = sanitize_id(id);
    if validate_rule_id(&sanitized) {
        Some(sanitized)
    } else {
        None
    }
}
