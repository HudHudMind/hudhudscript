//! Rule evaluation engine

use crate::condition::evaluate_condition;
use crate::context::EvaluationContext;
use hudhudscript_governance::{Action, Rule};

/// Result of rule evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct RuleResult {
    pub matched: bool,
    pub actions: Vec<Action>,
    pub rule_id: String,
}

/// Evaluate a rule against a context
pub fn evaluate_rule(rule: &Rule, context: &EvaluationContext) -> RuleResult {
    // Evaluate all conditions
    let mut matched = true;
    for condition in &rule.conditions {
        if !evaluate_condition(condition, context) {
            matched = false;
            break;
        }
    }

    // Return result with actions if matched
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
