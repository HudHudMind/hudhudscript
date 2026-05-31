//! Condition evaluation logic for rules
//!
//! This module provides evaluation logic for all condition types including:
//! - Comparison operators: Equals, NotEquals, GreaterThan, LessThan, Between, In
//! - Logical operators: And, Or, Not
//! - Short-circuit evaluation for And/Or operators
//! - Support for nested logical operators to arbitrary depth

use crate::context::EvaluationContext;
use hudhudscript_governance::Condition;
use serde_json::Value;

/// Evaluate a condition against a context
///
/// # Arguments
/// * `condition` - The condition to evaluate
/// * `context` - The evaluation context containing field values
///
/// # Returns
/// * `true` if the condition is satisfied
/// * `false` if the condition is not satisfied or if a referenced field is missing
///
/// # Examples
/// ```
/// use hudhudscript_rules::context::EvaluationContext;
/// use hudhudscript_rules::condition::evaluate_condition;
/// use hudhudscript_governance::Condition;
/// use serde_json::json;
///
/// let mut context = EvaluationContext::new();
/// context.insert("status".to_string(), json!("active"));
///
/// let condition = Condition::Equals {
///     field: "status".to_string(),
///     value: json!("active"),
/// };
///
/// assert!(evaluate_condition(&condition, &context));
/// ```
pub fn evaluate_condition(condition: &Condition, context: &EvaluationContext) -> bool {
    match condition {
        // Comparison operators
        Condition::Equals { field, value } => evaluate_equals(field, value, context),
        Condition::NotEquals { field, value } => evaluate_not_equals(field, value, context),
        Condition::GreaterThan { field, value } => evaluate_greater_than(field, *value, context),
        Condition::LessThan { field, value } => evaluate_less_than(field, *value, context),
        Condition::Between { field, min, max } => evaluate_between(field, *min, *max, context),
        Condition::In { field, values } => evaluate_in(field, values, context),

        // Logical operators with short-circuit evaluation
        Condition::And(conditions) => evaluate_and(conditions, context),
        Condition::Or(conditions) => evaluate_or(conditions, context),
        Condition::Not(condition) => evaluate_not(condition, context),
    }
}

/// Evaluate Equals condition
fn evaluate_equals(field: &str, value: &Value, context: &EvaluationContext) -> bool {
    match context.get(field) {
        Some(field_value) => field_value == value,
        None => {
            log::warn!("Field '{}' not found in evaluation context", field);
            false
        }
    }
}

/// Evaluate NotEquals condition
fn evaluate_not_equals(field: &str, value: &Value, context: &EvaluationContext) -> bool {
    match context.get(field) {
        Some(field_value) => field_value != value,
        None => {
            log::warn!("Field '{}' not found in evaluation context", field);
            false
        }
    }
}

/// Evaluate GreaterThan condition
fn evaluate_greater_than(field: &str, value: f64, context: &EvaluationContext) -> bool {
    match context.get(field) {
        Some(field_value) => {
            if let Some(num) = field_value.as_f64() {
                num > value
            } else {
                log::warn!("Field '{}' is not a number", field);
                false
            }
        }
        None => {
            log::warn!("Field '{}' not found in evaluation context", field);
            false
        }
    }
}

/// Evaluate LessThan condition
fn evaluate_less_than(field: &str, value: f64, context: &EvaluationContext) -> bool {
    match context.get(field) {
        Some(field_value) => {
            if let Some(num) = field_value.as_f64() {
                num < value
            } else {
                log::warn!("Field '{}' is not a number", field);
                false
            }
        }
        None => {
            log::warn!("Field '{}' not found in evaluation context", field);
            false
        }
    }
}

/// Evaluate Between condition (inclusive on both ends)
fn evaluate_between(field: &str, min: f64, max: f64, context: &EvaluationContext) -> bool {
    match context.get(field) {
        Some(field_value) => {
            if let Some(num) = field_value.as_f64() {
                num >= min && num <= max
            } else {
                log::warn!("Field '{}' is not a number", field);
                false
            }
        }
        None => {
            log::warn!("Field '{}' not found in evaluation context", field);
            false
        }
    }
}

/// Evaluate In condition
fn evaluate_in(field: &str, values: &[Value], context: &EvaluationContext) -> bool {
    match context.get(field) {
        Some(field_value) => values.contains(field_value),
        None => {
            log::warn!("Field '{}' not found in evaluation context", field);
            false
        }
    }
}

/// Evaluate And condition with short-circuit evaluation
/// Returns false as soon as any condition is false
fn evaluate_and(conditions: &[Condition], context: &EvaluationContext) -> bool {
    for condition in conditions {
        if !evaluate_condition(condition, context) {
            // Short-circuit: stop evaluation when first false is found
            return false;
        }
    }
    true
}

/// Evaluate Or condition with short-circuit evaluation
/// Returns true as soon as any condition is true
fn evaluate_or(conditions: &[Condition], context: &EvaluationContext) -> bool {
    for condition in conditions {
        if evaluate_condition(condition, context) {
            // Short-circuit: stop evaluation when first true is found
            return true;
        }
    }
    false
}

/// Evaluate Not condition
fn evaluate_not(condition: &Condition, context: &EvaluationContext) -> bool {
    !evaluate_condition(condition, context)
}
