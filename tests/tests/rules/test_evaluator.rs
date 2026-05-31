//! External tests for hudhudscript_rules::evaluator

use hudhudscript_governance::{Action, Condition, Rule};
use hudhudscript_rules::context::EvaluationContext;
use hudhudscript_rules::evaluator::evaluate_rule;
use serde_json::json;

#[test]
fn test_evaluator_rule_matched() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    let rule = Rule {
        id: "eval.1".to_string(),
        name: "Evaluator Rule".to_string(),
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        actions: vec![Action::Allow],
        priority: 5,
    };

    let result = evaluate_rule(&rule, &context);
    assert!(result.matched);
    assert_eq!(result.actions.len(), 1);
    assert_eq!(result.rule_id, "eval.1");
}

#[test]
fn test_evaluator_rule_not_matched() {
    let context = EvaluationContext::new();

    let rule = Rule {
        id: "eval.2".to_string(),
        name: "Fail Rule".to_string(),
        conditions: vec![Condition::Equals {
            field: "missing".to_string(),
            value: json!("val"),
        }],
        actions: vec![Action::Deny],
        priority: 1,
    };

    let result = evaluate_rule(&rule, &context);
    assert!(!result.matched);
    assert_eq!(result.actions.len(), 0);
    assert_eq!(result.rule_id, "eval.2");
}

#[test]
fn test_evaluator_empty_conditions_matches() {
    let context = EvaluationContext::new();

    let rule = Rule {
        id: "eval.3".to_string(),
        name: "No Conditions".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 0,
    };

    let result = evaluate_rule(&rule, &context);
    assert!(result.matched);
    assert_eq!(result.actions.len(), 1);
}

#[test]
fn test_evaluator_short_circuits_on_first_false() {
    let mut context = EvaluationContext::new();
    context.insert("a".to_string(), json!("wrong"));

    let rule = Rule {
        id: "eval.4".to_string(),
        name: "Short Circuit".to_string(),
        conditions: vec![
            Condition::Equals {
                field: "a".to_string(),
                value: json!("right"),
            },
            Condition::Equals {
                field: "b".to_string(),
                value: json!("val"),
            },
        ],
        actions: vec![Action::Allow],
        priority: 0,
    };

    let result = evaluate_rule(&rule, &context);
    assert!(!result.matched);
    assert_eq!(result.actions.len(), 0);
}
