//! Governance model enforcement

use crate::audit::AuditLogger;
use crate::types::{AgentRole, Constitution, EnforcementLevel, GovernanceModel};

use super::{check_law_compliance, EnforcementResult, EvaluationContext};

/// Enforce constitution with governance model
///
/// This function applies governance model coefficients to enforcement decisions.
/// The governance model defines how strictly constitution, laws, and rules are enforced.
///
/// # Arguments
/// * `constitution` - The constitution containing laws to enforce
/// * `action_context` - Evaluation context representing the agent action
/// * `governance_model` - Governance model defining enforcement flexibility
/// * `agent_role` - Role of the agent performing the action (for privilege checks)
/// * `audit_logger` - Optional audit logger to record the enforcement decision
///
/// # Returns
/// * `EnforcementResult` with allowed status, violations list, and advisory violations
///
/// # Examples
/// ```
/// use hudhudscript_governance::enforcement::{enforce_with_model, EvaluationContext};
/// use hudhudscript_governance::{Constitution, Law, EnforcementLevel, Condition, GovernanceModel, AgentRole};
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
/// // Create action context with non-compliant data
/// let mut context = EvaluationContext::new();
/// context.insert("data_size".to_string(), json!(1500));
///
/// // With democracy model (strict enforcement), action is denied
/// let democracy = GovernanceModel::democracy();
/// let result1 = enforce_with_model(&constitution, &context, &democracy, &AgentRole::Member, None);
/// assert!(!result1.allowed);
///
/// // With anarchy model (no enforcement), action is allowed
/// let anarchy = GovernanceModel::anarchy();
/// let result2 = enforce_with_model(&constitution, &context, &anarchy, &AgentRole::Member, None);
/// assert!(result2.allowed);
/// ```
pub fn enforce_with_model(
    constitution: &Constitution,
    action_context: &EvaluationContext,
    governance_model: &GovernanceModel,
    agent_role: &AgentRole,
    audit_logger: Option<&AuditLogger>,
) -> EnforcementResult {
    // Step 1: Check if agent has special privileges
    let role_name = match agent_role {
        AgentRole::Custom(name) => name.as_str(),
        AgentRole::Prosecutor => "prosecutor",
        AgentRole::Judge => "judge",
        AgentRole::Executor => "executor",
        AgentRole::Member => "member",
    };

    if let Some(privileges) = governance_model.has_privileges(role_name) {
        if privileges.bypass_constitution {
            let result = EnforcementResult::allowed().with_message(format!(
                "Agent role '{}' has constitutional bypass privilege",
                role_name
            ));

            if let Some(logger) = audit_logger {
                logger.log_enforcement_decision(
                    constitution.id.clone(),
                    format!("Enforcement bypassed by privileged role '{}'", role_name),
                    result.allowed,
                    result.violations.clone(),
                    result.advisory_violations.clone(),
                );
            }

            return result;
        }
    }

    // Step 2: Apply constitution compliance coefficient
    let compliance_required = governance_model.constitution_compliance;

    if compliance_required < 0.5 {
        // Low compliance requirement - allow with warning
        let result = EnforcementResult::allowed().with_message(format!(
            "Constitution compliance not required ({}% enforcement)",
            (compliance_required * 100.0) as u32
        ));

        if let Some(logger) = audit_logger {
            logger.log_enforcement_decision(
                constitution.id.clone(),
                format!(
                    "Low compliance requirement: {}%",
                    (compliance_required * 100.0) as u32
                ),
                result.allowed,
                result.violations.clone(),
                result.advisory_violations.clone(),
            );
        }

        return result;
    }

    // Step 3: Check all laws with flexibility applied
    let mut mandatory_violations = Vec::new();
    let mut advisory_violations = Vec::new();

    for law in constitution.laws.values() {
        let complies = check_law_compliance(law, action_context);

        if !complies {
            // Apply law flexibility coefficient
            let flexibility = governance_model.law_flexibility;
            let enforcement_level = match law.enforcement_level {
                EnforcementLevel::Mandatory => {
                    if flexibility > 0.7 {
                        // High flexibility - downgrade to advisory
                        EnforcementLevel::Advisory
                    } else {
                        EnforcementLevel::Mandatory
                    }
                }
                EnforcementLevel::Advisory => EnforcementLevel::Advisory,
                EnforcementLevel::Optional => EnforcementLevel::Optional,
            };

            match enforcement_level {
                EnforcementLevel::Mandatory => {
                    mandatory_violations.push(law.id.clone());
                }
                EnforcementLevel::Advisory => {
                    advisory_violations.push(law.id.clone());
                }
                EnforcementLevel::Optional => {}
            }
        }
    }

    // Step 4: Apply rule enforcement coefficient
    let enforcement_threshold = governance_model.rule_enforcement;

    let result = if mandatory_violations.is_empty() || enforcement_threshold < 0.5 {
        let mut result =
            EnforcementResult::allowed().with_advisory_violations(advisory_violations.clone());

        if !mandatory_violations.is_empty() && enforcement_threshold < 0.5 {
            result = result.with_message(format!(
                "Violations ignored due to low enforcement threshold ({}%)",
                (enforcement_threshold * 100.0) as u32
            ));
        }

        result
    } else {
        EnforcementResult::denied(mandatory_violations.clone())
            .with_advisory_violations(advisory_violations.clone())
    };

    // Step 5: Log enforcement decision if audit logger provided
    if let Some(logger) = audit_logger {
        logger.log_enforcement_decision(
            constitution.id.clone(),
            format!(
                "Enforcement with {:?} model (compliance: {}%, flexibility: {}%, enforcement: {}%)",
                governance_model.model_type,
                (compliance_required * 100.0) as u32,
                (governance_model.law_flexibility * 100.0) as u32,
                (enforcement_threshold * 100.0) as u32
            ),
            result.allowed,
            result.violations.clone(),
            result.advisory_violations.clone(),
        );
    }

    result
}
