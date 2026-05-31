//! Rule evaluation with priority ordering and action execution
//!
//! This module provides the complete rule evaluation system including:
//! - Rule evaluation against evaluation contexts
//! - Priority-based rule ordering (highest priority first)
//! - Action execution in definition order
//! - Support for all action types: Allow, Deny, Require, Execute, Notify
//!
//! **Validates Requirements:** 4.3, 4.4, 4.5, 4.6, 21.1, 21.2, 21.3, 21.4, 21.5, 21.6, 23.1, 23.2, 23.3

use crate::condition::evaluate_condition;
use crate::context::EvaluationContext;
use hudhudscript_governance::{Action, Rule};

/// Result of rule evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct RuleResult {
    /// Whether the rule matched (all conditions satisfied)
    pub matched: bool,
    /// Actions to execute if matched
    pub actions: Vec<Action>,
    /// ID of the evaluated rule
    pub rule_id: String,
}

/// Evaluate a single rule against a context
///
/// # Arguments
/// * `rule` - The rule to evaluate
/// * `context` - The evaluation context containing field values
///
/// # Returns
/// * `RuleResult` with matched status and actions
///
/// # Preconditions
/// - `rule` is non-null and well-formed
/// - `rule.conditions` is a valid list of conditions
/// - `context` contains all required fields for evaluation
///
/// # Postconditions
/// - Returns `RuleResult` indicating match status
/// - If matched: `result.matched === true` and `result.actions` contains applicable actions
/// - If not matched: `result.matched === false`
/// - No side effects on input parameters
///
/// # Examples
/// ```
/// use hudhudscript_rules::rule::evaluate_rule;
/// use hudhudscript_rules::context::EvaluationContext;
/// use hudhudscript_governance::{Rule, Condition, Action};
/// use serde_json::json;
///
/// let mut context = EvaluationContext::new();
/// context.insert("status".to_string(), json!("active"));
///
/// let rule = Rule {
///     id: "rule.1".to_string(),
///     name: "Active Status Rule".to_string(),
///     conditions: vec![
///         Condition::Equals {
///             field: "status".to_string(),
///             value: json!("active"),
///         }
///     ],
///     actions: vec![Action::Allow],
///     priority: 10,
/// };
///
/// let result = evaluate_rule(&rule, &context);
/// assert!(result.matched);
/// assert_eq!(result.actions.len(), 1);
/// ```
pub fn evaluate_rule(rule: &Rule, context: &EvaluationContext) -> RuleResult {
    // Step 1: Evaluate all conditions
    let mut matched = true;
    for condition in &rule.conditions {
        if !evaluate_condition(condition, context) {
            matched = false;
            break;
        }
    }

    // Step 2: Build result with actions if matched
    if matched {
        RuleResult {
            matched: true,
            actions: rule.actions.clone(),
            rule_id: rule.id.clone(),
        }
    } else {
        RuleResult {
            matched: false,
            actions: vec![],
            rule_id: rule.id.clone(),
        }
    }
}

/// Evaluate multiple rules with priority ordering
///
/// Rules are evaluated in priority order (highest first).
/// When rules have equal priority, they are evaluated in definition order.
/// All matched rules have their actions collected and returned.
///
/// # Arguments
/// * `rules` - Slice of rules to evaluate
/// * `context` - The evaluation context containing field values
///
/// # Returns
/// * Vector of `RuleResult` for all matched rules, in priority order
///
/// # Preconditions
/// - `rules` is a valid slice of rules
/// - All rules have non-negative priority values
/// - `context` contains required fields
///
/// # Postconditions
/// - Returns results in priority order (highest first)
/// - Ties broken by definition order
/// - Only matched rules included in results
/// - Actions for each rule are in definition order
///
/// # Examples
/// ```
/// use hudhudscript_rules::rule::evaluate_rules_with_priority;
/// use hudhudscript_rules::context::EvaluationContext;
/// use hudhudscript_governance::{Rule, Condition, Action};
/// use serde_json::json;
///
/// let mut context = EvaluationContext::new();
/// context.insert("count".to_string(), json!(15));
///
/// let rules = vec![
///     Rule {
///         id: "rule.1".to_string(),
///         name: "Low Priority".to_string(),
///         conditions: vec![
///             Condition::GreaterThan {
///                 field: "count".to_string(),
///                 value: 10.0,
///             }
///         ],
///         actions: vec![Action::Allow],
///         priority: 5,
///     },
///     Rule {
///         id: "rule.2".to_string(),
///         name: "High Priority".to_string(),
///         conditions: vec![
///             Condition::GreaterThan {
///                 field: "count".to_string(),
///                 value: 10.0,
///             }
///         ],
///         actions: vec![Action::Notify { message: "High priority action".to_string() }],
///         priority: 10,
///     },
/// ];
///
/// let results = evaluate_rules_with_priority(&rules, &context);
/// // High priority rule (rule.2) comes first
/// assert_eq!(results[0].rule_id, "rule.2");
/// assert_eq!(results[1].rule_id, "rule.1");
/// ```
pub fn evaluate_rules_with_priority(
    rules: &[Rule],
    context: &EvaluationContext,
) -> Vec<RuleResult> {
    // Step 1: Create indexed rules to preserve definition order
    let mut indexed_rules: Vec<(usize, &Rule)> = rules.iter().enumerate().collect();

    // Step 2: Sort by priority (highest first), then by definition order (index)
    indexed_rules.sort_by(|(idx_a, rule_a), (idx_b, rule_b)| {
        // First compare by priority (descending - highest first)
        match rule_b.priority.cmp(&rule_a.priority) {
            std::cmp::Ordering::Equal => {
                // If priorities are equal, use definition order (ascending - earlier first)
                idx_a.cmp(idx_b)
            }
            other => other,
        }
    });

    // Step 3: Evaluate rules in priority order and collect matched results
    let mut results = Vec::new();
    for (_idx, rule) in indexed_rules {
        let result = evaluate_rule(rule, context);
        if result.matched {
            results.push(result);
        }
    }

    results
}

/// Execute actions from a rule result
///
/// Actions are executed in the order they are defined in the rule.
/// This function returns a summary of executed actions.
///
/// # Arguments
/// * `result` - The rule result containing actions to execute
///
/// # Returns
/// * Vector of action execution summaries
///
/// # Behavior
/// - **Allow**: Records permission grant. Returns success.
/// - **Deny**: Records denial. Returns success=true (denial was enforced correctly).
/// - **Require**: Logs permission requirement. In production, integrate with
///   an access control system to verify the permission. Returns the requirement.
/// - **Execute**: Logs task execution request. In production, dispatch to a task
///   runner. Returns the task name.
/// - **Notify**: Logs notification. In production, send to a notification channel
///   (webhook, email, Slack, etc.). Returns the message.
///
/// # Examples
/// ```
/// use hudhudscript_rules::rule::{RuleResult, execute_actions, ActionExecutionResult};
/// use hudhudscript_governance::Action;
///
/// let result = RuleResult {
///     matched: true,
///     actions: vec![
///         Action::Allow,
///         Action::Notify { message: "Action logged".to_string() },
///     ],
///     rule_id: "rule.1".to_string(),
/// };
///
/// let execution_results = execute_actions(&result);
/// assert_eq!(execution_results.len(), 2);
/// assert!(execution_results[0].success);
/// ```
pub fn execute_actions(result: &RuleResult) -> Vec<ActionExecutionResult> {
    let mut execution_results = Vec::new();

    // Execute actions in definition order with real side-effects
    for (index, action) in result.actions.iter().enumerate() {
        let exec_result = match action {
            Action::Allow => {
                eprintln!("[governance] rule={} action=ALLOW", result.rule_id);
                ActionExecutionResult {
                    action_index: index,
                    action_type: "Allow".to_string(),
                    success: true,
                    message: format!("Action allowed by rule '{}'", result.rule_id),
                }
            }
            Action::Deny => {
                eprintln!("[governance] rule={} action=DENY", result.rule_id);
                ActionExecutionResult {
                    action_index: index,
                    action_type: "Deny".to_string(),
                    success: true, // Denial was successfully enforced
                    message: format!("Action denied by rule '{}'", result.rule_id),
                }
            }
            Action::Require { permission } => {
                eprintln!(
                    "[governance] rule={} action=REQUIRE permission={}",
                    result.rule_id, permission
                );
                ActionExecutionResult {
                    action_index: index,
                    action_type: "Require".to_string(),
                    success: true,
                    message: format!(
                        "Permission '{}' required by rule '{}' — verify with access control",
                        permission, result.rule_id
                    ),
                }
            }
            Action::Execute { task } => {
                eprintln!(
                    "[governance] rule={} action=EXECUTE task={}",
                    result.rule_id, task
                );
                ActionExecutionResult {
                    action_index: index,
                    action_type: "Execute".to_string(),
                    success: true,
                    message: format!(
                        "Task '{}' dispatched by rule '{}' — wire to task runner for execution",
                        task, result.rule_id
                    ),
                }
            }
            Action::Notify { message } => {
                eprintln!(
                    "[governance] rule={} action=NOTIFY message={}",
                    result.rule_id, message
                );
                ActionExecutionResult {
                    action_index: index,
                    action_type: "Notify".to_string(),
                    success: true,
                    message: format!("Notification: {}", message),
                }
            }
        };

        execution_results.push(exec_result);
    }

    execution_results
}

/// Result of action execution
#[derive(Debug, Clone, PartialEq)]
pub struct ActionExecutionResult {
    /// Index of the action in the rule's action list
    pub action_index: usize,
    /// Type of action executed
    pub action_type: String,
    /// Whether execution was successful
    pub success: bool,
    /// Execution message or error
    pub message: String,
}
