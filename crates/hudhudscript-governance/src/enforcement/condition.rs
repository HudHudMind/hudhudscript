//! Condition evaluation logic

use super::EvaluationContext;
use crate::types::Condition;

/// Evaluate a condition against a context
pub fn evaluate_condition(condition: &Condition, context: &EvaluationContext) -> bool {
    match condition {
        Condition::Equals { field, value } => match context.get(field) {
            Some(field_value) => field_value == value,
            None => false,
        },
        Condition::NotEquals { field, value } => match context.get(field) {
            Some(field_value) => field_value != value,
            None => false,
        },
        Condition::GreaterThan { field, value } => match context.get(field) {
            Some(field_value) => {
                if let Some(num) = field_value.as_f64() {
                    num > *value
                } else {
                    log::warn!("Field '{}' is not a number", field);
                    false
                }
            }
            None => false,
        },
        Condition::LessThan { field, value } => match context.get(field) {
            Some(field_value) => {
                if let Some(num) = field_value.as_f64() {
                    num < *value
                } else {
                    log::warn!("Field '{}' is not a number", field);
                    false
                }
            }
            None => false,
        },
        Condition::Between { field, min, max } => match context.get(field) {
            Some(field_value) => {
                if let Some(num) = field_value.as_f64() {
                    num >= *min && num <= *max
                } else {
                    log::warn!("Field '{}' is not a number", field);
                    false
                }
            }
            None => false,
        },
        Condition::In { field, values } => match context.get(field) {
            Some(field_value) => values.contains(field_value),
            None => false,
        },
        Condition::And(conditions) => {
            for condition in conditions {
                if !evaluate_condition(condition, context) {
                    return false;
                }
            }
            true
        }
        Condition::Or(conditions) => {
            for condition in conditions {
                if evaluate_condition(condition, context) {
                    return true;
                }
            }
            false
        }
        Condition::Not(condition) => !evaluate_condition(condition, context),
    }
}
