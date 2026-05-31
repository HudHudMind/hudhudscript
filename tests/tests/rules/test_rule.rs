//! External tests for hudhudscript_rules::rule

use hudhudscript_governance::{Action, Condition, Rule};
use hudhudscript_rules::context::EvaluationContext;
use hudhudscript_rules::rule::{
    evaluate_rule, evaluate_rules_with_priority, execute_actions, RuleResult,
};
use serde_json::json;

#[test]
fn test_evaluate_rule_matched() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));
    context.insert("count".to_string(), json!(15));

    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test Rule".to_string(),
        conditions: vec![
            Condition::Equals {
                field: "status".to_string(),
                value: json!("active"),
            },
            Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            },
        ],
        actions: vec![
            Action::Allow,
            Action::Notify {
                message: "Rule matched".to_string(),
            },
        ],
        priority: 10,
    };

    let result = evaluate_rule(&rule, &context);
    assert!(result.matched);
    assert_eq!(result.actions.len(), 2);
    assert_eq!(result.rule_id, "rule.1");
}

#[test]
fn test_evaluate_rule_not_matched() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("inactive"));

    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test Rule".to_string(),
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        actions: vec![Action::Allow],
        priority: 10,
    };

    let result = evaluate_rule(&rule, &context);
    assert!(!result.matched);
    assert_eq!(result.actions.len(), 0);
    assert_eq!(result.rule_id, "rule.1");
}

#[test]
fn test_evaluate_rule_all_action_types() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    let rule = Rule {
        id: "rule.1".to_string(),
        name: "All Actions Rule".to_string(),
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        actions: vec![
            Action::Allow,
            Action::Deny,
            Action::Require {
                permission: "admin".to_string(),
            },
            Action::Execute {
                task: "validate".to_string(),
            },
            Action::Notify {
                message: "Action logged".to_string(),
            },
        ],
        priority: 10,
    };

    let result = evaluate_rule(&rule, &context);
    assert!(result.matched);
    assert_eq!(result.actions.len(), 5);
}

#[test]
fn test_evaluate_rules_with_priority_ordering() {
    let mut context = EvaluationContext::new();
    context.insert("count".to_string(), json!(15));

    let rules = vec![
        Rule {
            id: "rule.1".to_string(),
            name: "Low Priority".to_string(),
            conditions: vec![Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            }],
            actions: vec![Action::Allow],
            priority: 5,
        },
        Rule {
            id: "rule.2".to_string(),
            name: "High Priority".to_string(),
            conditions: vec![Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            }],
            actions: vec![Action::Notify {
                message: "High priority".to_string(),
            }],
            priority: 10,
        },
        Rule {
            id: "rule.3".to_string(),
            name: "Medium Priority".to_string(),
            conditions: vec![Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            }],
            actions: vec![Action::Allow],
            priority: 7,
        },
    ];

    let results = evaluate_rules_with_priority(&rules, &context);

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].rule_id, "rule.2");
    assert_eq!(results[1].rule_id, "rule.3");
    assert_eq!(results[2].rule_id, "rule.1");
}

#[test]
fn test_evaluate_rules_equal_priority_definition_order() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    let rules = vec![
        Rule {
            id: "rule.1".to_string(),
            name: "First".to_string(),
            conditions: vec![Condition::Equals {
                field: "status".to_string(),
                value: json!("active"),
            }],
            actions: vec![Action::Allow],
            priority: 10,
        },
        Rule {
            id: "rule.2".to_string(),
            name: "Second".to_string(),
            conditions: vec![Condition::Equals {
                field: "status".to_string(),
                value: json!("active"),
            }],
            actions: vec![Action::Allow],
            priority: 10,
        },
        Rule {
            id: "rule.3".to_string(),
            name: "Third".to_string(),
            conditions: vec![Condition::Equals {
                field: "status".to_string(),
                value: json!("active"),
            }],
            actions: vec![Action::Allow],
            priority: 10,
        },
    ];

    let results = evaluate_rules_with_priority(&rules, &context);

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].rule_id, "rule.1");
    assert_eq!(results[1].rule_id, "rule.2");
    assert_eq!(results[2].rule_id, "rule.3");
}

#[test]
fn test_evaluate_rules_mixed_matching() {
    let mut context = EvaluationContext::new();
    context.insert("count".to_string(), json!(15));

    let rules = vec![
        Rule {
            id: "rule.1".to_string(),
            name: "Matches".to_string(),
            conditions: vec![Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            }],
            actions: vec![Action::Allow],
            priority: 10,
        },
        Rule {
            id: "rule.2".to_string(),
            name: "Does Not Match".to_string(),
            conditions: vec![Condition::LessThan {
                field: "count".to_string(),
                value: 10.0,
            }],
            actions: vec![Action::Deny],
            priority: 20,
        },
        Rule {
            id: "rule.3".to_string(),
            name: "Matches".to_string(),
            conditions: vec![Condition::Between {
                field: "count".to_string(),
                min: 10.0,
                max: 20.0,
            }],
            actions: vec![Action::Allow],
            priority: 5,
        },
    ];

    let results = evaluate_rules_with_priority(&rules, &context);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].rule_id, "rule.1");
    assert_eq!(results[1].rule_id, "rule.3");
}

#[test]
fn test_execute_actions_all_types() {
    let result = RuleResult {
        matched: true,
        actions: vec![
            Action::Allow,
            Action::Deny,
            Action::Require {
                permission: "admin".to_string(),
            },
            Action::Execute {
                task: "validate".to_string(),
            },
            Action::Notify {
                message: "Action logged".to_string(),
            },
        ],
        rule_id: "rule.1".to_string(),
    };

    let execution_results = execute_actions(&result);

    assert_eq!(execution_results.len(), 5);

    assert_eq!(execution_results[0].action_index, 0);
    assert_eq!(execution_results[0].action_type, "Allow");
    assert!(execution_results[0].success);

    assert_eq!(execution_results[1].action_index, 1);
    assert_eq!(execution_results[1].action_type, "Deny");

    assert_eq!(execution_results[2].action_index, 2);
    assert_eq!(execution_results[2].action_type, "Require");
    assert!(execution_results[2].message.contains("admin"));

    assert_eq!(execution_results[3].action_index, 3);
    assert_eq!(execution_results[3].action_type, "Execute");
    assert!(execution_results[3].message.contains("validate"));

    assert_eq!(execution_results[4].action_index, 4);
    assert_eq!(execution_results[4].action_type, "Notify");
    assert!(execution_results[4].message.contains("Action logged"));
}

#[test]
fn test_execute_actions_empty() {
    let result = RuleResult {
        matched: false,
        actions: vec![],
        rule_id: "rule.1".to_string(),
    };

    let execution_results = execute_actions(&result);
    assert_eq!(execution_results.len(), 0);
}

#[test]
fn test_rule_evaluation_determinism() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));
    context.insert("count".to_string(), json!(15));

    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Deterministic Rule".to_string(),
        conditions: vec![
            Condition::Equals {
                field: "status".to_string(),
                value: json!("active"),
            },
            Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            },
        ],
        actions: vec![Action::Allow],
        priority: 10,
    };

    let result1 = evaluate_rule(&rule, &context);
    let result2 = evaluate_rule(&rule, &context);
    let result3 = evaluate_rule(&rule, &context);

    assert_eq!(result1, result2);
    assert_eq!(result2, result3);
}

#[test]
fn test_action_execution_order() {
    let result = RuleResult {
        matched: true,
        actions: vec![
            Action::Notify {
                message: "First".to_string(),
            },
            Action::Notify {
                message: "Second".to_string(),
            },
            Action::Notify {
                message: "Third".to_string(),
            },
        ],
        rule_id: "rule.1".to_string(),
    };

    let execution_results = execute_actions(&result);

    assert_eq!(execution_results[0].action_index, 0);
    assert!(execution_results[0].message.contains("First"));

    assert_eq!(execution_results[1].action_index, 1);
    assert!(execution_results[1].message.contains("Second"));

    assert_eq!(execution_results[2].action_index, 2);
    assert!(execution_results[2].message.contains("Third"));
}
