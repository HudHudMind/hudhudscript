//! Core enforcement engine

use crate::audit::AuditLogger;
use crate::types::{Constitution, EnforcementLevel, Law};

use super::{evaluate_condition, EnforcementResult, EvaluationContext};

/// Enforce constitution laws against an agent action
///
/// This function checks all mandatory laws in the constitution against the provided
/// action context. If any mandatory law is violated, the action is denied. Advisory
/// laws are evaluated but do not block the action.
///
/// If an audit logger is provided, the enforcement decision will be logged.
///
/// # Arguments
/// * `constitution` - The constitution containing laws to enforce
/// * `action_context` - Evaluation context representing the agent action
/// * `audit_logger` - Optional audit logger to record the enforcement decision
///
/// # Returns
/// * `EnforcementResult` with allowed status, violations list, and advisory violations
///
/// # Preconditions
/// - `constitution` is valid and active
/// - `action_context` is well-formed
///
/// # Postconditions
/// - Returns `EnforcementResult` with validation status
/// - If allowed: `result.allowed === true`
/// - If denied: `result.allowed === false` and `result.violations` lists violated laws
/// - No state changes if action is denied
/// - Deterministic: same inputs always produce same result
/// - If audit_logger provided: enforcement decision is logged
///
/// # Examples
/// ```
/// use hudhudscript_governance::enforcement::{enforce_constitution, EnforcementResult, EvaluationContext};
/// use hudhudscript_governance::audit::AuditLogger;
/// use hudhudscript_governance::{Constitution, Law, EnforcementLevel, Condition};
/// use serde_json::json;
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// // Create a constitution with a mandatory law
/// let mut laws = HashMap::new();
/// laws.insert(
///     "cons1.law1".to_string(),
///     Law {
///         id: "cons1.law1".to_string(),
///         constitution_id: "cons.1".to_string(),
///         name: "Data Size Limit".to_string(),
///         description: "Data must be under 1000 bytes".to_string(),
///         enforcement_level: EnforcementLevel::Mandatory,
///         conditions: vec![
///             Condition::LessThan {
///                 field: "data_size".to_string(),
///                 value: 1000.0,
///             }
///         ],
///     },
/// );
///
/// let constitution = Constitution {
///     id: "cons.1".to_string(),
///     name: "Data Governance".to_string(),
///     description: None,
///     laws,
///     created_at: Utc::now(),
///     version: 1,
/// };
///
/// // Create action context with compliant data
/// let mut context = EvaluationContext::new();
/// context.insert("data_size".to_string(), json!(500));
///
/// // Enforce without audit logging
/// let result = enforce_constitution(&constitution, &context, None);
/// assert!(result.allowed);
///
/// // Enforce with audit logging
/// let audit_logger = AuditLogger::new();
/// let result2 = enforce_constitution(&constitution, &context, Some(&audit_logger));
/// assert!(result2.allowed);
/// assert_eq!(audit_logger.count(), 1);
/// ```
pub fn enforce_constitution(
    constitution: &Constitution,
    action_context: &EvaluationContext,
    audit_logger: Option<&AuditLogger>,
) -> EnforcementResult {
    let mut mandatory_violations = Vec::new();
    let mut advisory_violations = Vec::new();

    // Step 1: Check all laws in the constitution
    for law in constitution.laws.values() {
        // Evaluate law compliance
        let complies = check_law_compliance(law, action_context);

        // Step 2: Handle violations based on enforcement level
        if !complies {
            match law.enforcement_level {
                EnforcementLevel::Mandatory => {
                    // Mandatory law violation blocks the action
                    mandatory_violations.push(law.id.clone());
                }
                EnforcementLevel::Advisory => {
                    // Advisory law violation is informational only
                    advisory_violations.push(law.id.clone());
                }
                EnforcementLevel::Optional => {
                    // Optional law violations are ignored
                }
            }
        }
    }

    // Step 3: Determine result based on mandatory violations
    let result = if mandatory_violations.is_empty() {
        EnforcementResult::allowed().with_advisory_violations(advisory_violations.clone())
    } else {
        EnforcementResult::denied(mandatory_violations.clone())
            .with_advisory_violations(advisory_violations.clone())
    };

    // Step 4: Log enforcement decision if audit logger provided
    if let Some(logger) = audit_logger {
        logger.log_enforcement_decision(
            constitution.id.clone(),
            format!("Enforcement check for constitution {}", constitution.id),
            result.allowed,
            result.violations.clone(),
            result.advisory_violations.clone(),
        );
    }

    result
}

/// Check if an action complies with a specific law
///
/// A law is complied with if all its conditions are satisfied.
/// If the law has no conditions, it is considered complied with.
///
/// # Arguments
/// * `law` - The law to check
/// * `action_context` - Evaluation context representing the agent action
///
/// # Returns
/// * `true` if the action complies with the law, `false` otherwise
///
/// # Examples
/// ```
/// use hudhudscript_governance::enforcement::{check_law_compliance, EvaluationContext};
/// use hudhudscript_governance::{Law, EnforcementLevel, Condition};
/// use serde_json::json;
///
/// let law = Law {
///     id: "cons1.law1".to_string(),
///     constitution_id: "cons.1".to_string(),
///     name: "Active Status Required".to_string(),
///     description: "Agent must have active status".to_string(),
///     enforcement_level: EnforcementLevel::Mandatory,
///     conditions: vec![
///         Condition::Equals {
///             field: "status".to_string(),
///             value: json!("active"),
///         }
///     ],
/// };
///
/// let mut context = EvaluationContext::new();
/// context.insert("status".to_string(), json!("active"));
///
/// assert!(check_law_compliance(&law, &context));
///
/// let mut context2 = EvaluationContext::new();
/// context2.insert("status".to_string(), json!("inactive"));
///
/// assert!(!check_law_compliance(&law, &context2));
/// ```
pub fn check_law_compliance(law: &Law, action_context: &EvaluationContext) -> bool {
    // If law has no conditions, it's always complied with
    if law.conditions.is_empty() {
        return true;
    }

    // All conditions must be satisfied for compliance
    for condition in &law.conditions {
        if !evaluate_condition(condition, action_context) {
            return false;
        }
    }

    true
}
