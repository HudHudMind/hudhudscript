//! Property-based tests for hudhudscript_rules rule evaluation

use hudhudscript_governance::{Action, Condition, Rule};
use hudhudscript_rules::context::EvaluationContext;
use hudhudscript_rules::rule::{evaluate_rule, evaluate_rules_with_priority, execute_actions};
use proptest::prelude::*;
use serde_json::{json, Value};

// Strategy for generating JSON values
fn json_value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<String>().prop_map(Value::String),
        (-1000.0f64..1000.0f64).prop_map(|f| json!(f)),
        any::<bool>().prop_map(Value::Bool),
    ]
}

// Strategy for generating field names
fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_map(|s| s.to_string())
}

// Strategy for generating evaluation contexts
fn context_strategy() -> impl Strategy<Value = EvaluationContext> {
    prop::collection::hash_map(field_name_strategy(), json_value_strategy(), 0..10)
        .prop_map(EvaluationContext::from_map)
}

// Strategy for generating simple conditions
fn simple_condition_strategy() -> impl Strategy<Value = Condition> {
    prop_oneof![
        (field_name_strategy(), json_value_strategy())
            .prop_map(|(field, value)| { Condition::Equals { field, value } }),
        (field_name_strategy(), json_value_strategy())
            .prop_map(|(field, value)| { Condition::NotEquals { field, value } }),
        (field_name_strategy(), -1000.0f64..1000.0f64)
            .prop_map(|(field, value)| Condition::GreaterThan { field, value }),
        (field_name_strategy(), -1000.0f64..1000.0f64)
            .prop_map(|(field, value)| Condition::LessThan { field, value }),
    ]
}

// Strategy for generating actions
fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Allow),
        Just(Action::Deny),
        "[a-z]{3,10}".prop_map(|s| Action::Require { permission: s }),
        "[a-z]{3,10}".prop_map(|s| Action::Execute { task: s }),
        "[a-z]{3,10}".prop_map(|s| Action::Notify { message: s }),
    ]
}

// Strategy for generating rule IDs
fn rule_id_strategy() -> impl Strategy<Value = String> {
    (1u32..1000).prop_map(|n| format!("rule.{}", n))
}

// Strategy for generating rule names
fn rule_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{3,15}( [A-Z][a-z]{3,15}){0,2}".prop_map(|s| s.to_string())
}

// Strategy for generating priorities
fn priority_strategy() -> impl Strategy<Value = u32> {
    0u32..100
}

// Strategy for generating rules
fn rule_strategy() -> impl Strategy<Value = Rule> {
    (
        rule_id_strategy(),
        rule_name_strategy(),
        prop::collection::vec(simple_condition_strategy(), 1..5),
        prop::collection::vec(action_strategy(), 1..5),
        priority_strategy(),
    )
        .prop_map(|(id, name, conditions, actions, priority)| Rule {
            id,
            name,
            conditions,
            actions,
            priority,
        })
}

// **Property 9: Rule Condition Matching**
proptest! {
    #[test]
    fn property_rule_condition_matching_all_met(
        field in field_name_strategy(),
        value in json_value_strategy(),
        actions in prop::collection::vec(action_strategy(), 1..5)
    ) {
        // Create a rule with a single condition
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Test Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: actions.clone(),
            priority: 10,
        };

        // Create context where condition is met
        let mut context = EvaluationContext::new();
        context.insert(field.clone(), value.clone());

        let result = evaluate_rule(&rule, &context);

        // When all conditions are met, rule must match and return actions
        prop_assert!(result.matched, "Rule must match when all conditions are met");
        prop_assert_eq!(
            result.actions.len(),
            actions.len(),
            "All actions must be returned when rule matches"
        );
        prop_assert_eq!(
            result.actions,
            actions,
            "Returned actions must match rule's actions"
        );
    }
}

proptest! {
    #[test]
    fn property_rule_condition_matching_not_met(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        // Create a rule with a condition
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Test Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![Action::Allow, Action::Notify { message: "test".to_string() }],
            priority: 10,
        };

        // Create context where condition is NOT met (field missing)
        let context = EvaluationContext::new();

        let result = evaluate_rule(&rule, &context);

        // When any condition fails, rule must not match and return no actions
        prop_assert!(!result.matched, "Rule must not match when conditions fail");
        prop_assert_eq!(
            result.actions.len(),
            0,
            "No actions must be returned when rule doesn't match"
        );
    }
}

proptest! {
    #[test]
    fn property_rule_condition_matching_multiple_conditions(
        field1 in field_name_strategy(),
        field2 in field_name_strategy(),
        value1 in json_value_strategy(),
        value2 in json_value_strategy()
    ) {
        // Create a rule with multiple conditions
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Multi Condition Rule".to_string(),
            conditions: vec![
                Condition::Equals {
                    field: field1.clone(),
                    value: value1.clone(),
                },
                Condition::Equals {
                    field: field2.clone(),
                    value: value2.clone(),
                },
            ],
            actions: vec![Action::Allow],
            priority: 10,
        };

        // Context where both conditions are met
        let mut context_both = EvaluationContext::new();
        context_both.insert(field1.clone(), value1.clone());
        context_both.insert(field2.clone(), value2.clone());

        let result_both = evaluate_rule(&rule, &context_both);
        prop_assert!(result_both.matched, "Rule must match when all conditions are met");
        prop_assert!(!result_both.actions.is_empty(), "Actions must be returned");

        // Context where only first condition is met
        let mut context_first = EvaluationContext::new();
        context_first.insert(field1.clone(), value1.clone());

        let result_first = evaluate_rule(&rule, &context_first);
        prop_assert!(!result_first.matched, "Rule must not match when any condition fails");
        prop_assert!(result_first.actions.is_empty(), "No actions when rule doesn't match");

        // Context where only second condition is met
        let mut context_second = EvaluationContext::new();
        context_second.insert(field2.clone(), value2.clone());

        let result_second = evaluate_rule(&rule, &context_second);
        prop_assert!(!result_second.matched, "Rule must not match when any condition fails");
        prop_assert!(result_second.actions.is_empty(), "No actions when rule doesn't match");
    }
}

// **Property 11: Action Type Support**
proptest! {
    #[test]
    fn property_action_type_support_allow(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Allow Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![Action::Allow],
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);
        prop_assert!(result.matched);
        prop_assert_eq!(result.actions.len(), 1);
        prop_assert!(matches!(result.actions[0], Action::Allow));

        // Execute the action
        let exec_results = execute_actions(&result);
        prop_assert_eq!(exec_results.len(), 1);
        prop_assert_eq!(&exec_results[0].action_type, "Allow");
        prop_assert!(exec_results[0].success);
    }
}

proptest! {
    #[test]
    fn property_action_type_support_deny(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Deny Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![Action::Deny],
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);
        prop_assert!(result.matched);
        prop_assert_eq!(result.actions.len(), 1);
        prop_assert!(matches!(result.actions[0], Action::Deny));

        // Execute the action
        let exec_results = execute_actions(&result);
        prop_assert_eq!(exec_results.len(), 1);
        prop_assert_eq!(&exec_results[0].action_type, "Deny");
        prop_assert!(exec_results[0].success);
    }
}

proptest! {
    #[test]
    fn property_action_type_support_require(
        field in field_name_strategy(),
        value in json_value_strategy(),
        permission in "[a-z]{3,10}"
    ) {
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Require Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![Action::Require { permission: permission.clone() }],
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);
        prop_assert!(result.matched);
        prop_assert_eq!(result.actions.len(), 1);
        if let Action::Require { .. } = &result.actions[0] {
            // Action type is correct
        } else {
            prop_assert!(false, "Expected Require action");
        }

        // Execute the action
        let exec_results = execute_actions(&result);
        prop_assert_eq!(exec_results.len(), 1);
        prop_assert_eq!(&exec_results[0].action_type, "Require");
        prop_assert!(exec_results[0].success);
        prop_assert!(exec_results[0].message.contains(&permission));
    }
}

proptest! {
    #[test]
    fn property_action_type_support_execute(
        field in field_name_strategy(),
        value in json_value_strategy(),
        task in "[a-z]{3,10}"
    ) {
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Execute Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![Action::Execute { task: task.clone() }],
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);
        prop_assert!(result.matched);
        prop_assert_eq!(result.actions.len(), 1);
        if let Action::Execute { .. } = &result.actions[0] {
            // Action type is correct
        } else {
            prop_assert!(false, "Expected Execute action");
        }

        // Execute the action
        let exec_results = execute_actions(&result);
        prop_assert_eq!(exec_results.len(), 1);
        prop_assert_eq!(&exec_results[0].action_type, "Execute");
        prop_assert!(exec_results[0].success);
        prop_assert!(exec_results[0].message.contains(&task));
    }
}

proptest! {
    #[test]
    fn property_action_type_support_notify(
        field in field_name_strategy(),
        value in json_value_strategy(),
        message in "[a-z]{3,10}"
    ) {
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Notify Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![Action::Notify { message: message.clone() }],
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);
        prop_assert!(result.matched);
        prop_assert_eq!(result.actions.len(), 1);
        if let Action::Notify { .. } = &result.actions[0] {
            // Action type is correct
        } else {
            prop_assert!(false, "Expected Notify action");
        }

        // Execute the action
        let exec_results = execute_actions(&result);
        prop_assert_eq!(exec_results.len(), 1);
        prop_assert_eq!(&exec_results[0].action_type, "Notify");
        prop_assert!(exec_results[0].success);
        prop_assert!(exec_results[0].message.contains(&message));
    }
}

proptest! {
    #[test]
    fn property_action_type_support_all_types(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        // Rule with all action types
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "All Actions Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![
                Action::Allow,
                Action::Deny,
                Action::Require { permission: "admin".to_string() },
                Action::Execute { task: "validate".to_string() },
                Action::Notify { message: "logged".to_string() },
            ],
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);
        prop_assert!(result.matched);
        prop_assert_eq!(result.actions.len(), 5);

        // Execute all actions
        let exec_results = execute_actions(&result);
        prop_assert_eq!(exec_results.len(), 5);

        // Verify all action types are executed successfully
        prop_assert_eq!(&exec_results[0].action_type, "Allow");
        prop_assert_eq!(&exec_results[1].action_type, "Deny");
        prop_assert_eq!(&exec_results[2].action_type, "Require");
        prop_assert_eq!(&exec_results[3].action_type, "Execute");
        prop_assert_eq!(&exec_results[4].action_type, "Notify");

        for exec_result in &exec_results {
            prop_assert!(exec_result.success, "All actions must execute successfully");
        }
    }
}

// **Property 42: Action Execution Order**
proptest! {
    #[test]
    fn property_action_execution_order(
        field in field_name_strategy(),
        value in json_value_strategy(),
        actions in prop::collection::vec(action_strategy(), 2..10)
    ) {
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Ordered Actions Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: actions.clone(),
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);
        prop_assert!(result.matched);

        // Execute actions
        let exec_results = execute_actions(&result);

        // Verify execution order matches definition order
        prop_assert_eq!(
            exec_results.len(),
            actions.len(),
            "Number of executed actions must match number of defined actions"
        );

        for (index, exec_result) in exec_results.iter().enumerate() {
            prop_assert_eq!(
                exec_result.action_index,
                index,
                "Action execution order must match definition order"
            );
        }
    }
}

proptest! {
    #[test]
    fn property_action_execution_order_specific_sequence(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        // Create rule with specific action sequence
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "Sequence Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![
                Action::Notify { message: "first".to_string() },
                Action::Notify { message: "second".to_string() },
                Action::Notify { message: "third".to_string() },
            ],
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);
        let exec_results = execute_actions(&result);

        // Verify specific order
        prop_assert_eq!(exec_results.len(), 3);
        prop_assert!(exec_results[0].message.contains("first"));
        prop_assert!(exec_results[1].message.contains("second"));
        prop_assert!(exec_results[2].message.contains("third"));

        // Verify indices are sequential
        prop_assert_eq!(exec_results[0].action_index, 0);
        prop_assert_eq!(exec_results[1].action_index, 1);
        prop_assert_eq!(exec_results[2].action_index, 2);
    }
}

// **Property 46: Rule Priority Ordering**
proptest! {
    #[test]
    fn property_rule_priority_ordering_different_priorities(
        field in field_name_strategy(),
        value in json_value_strategy(),
        priorities in prop::collection::vec(0u32..100, 3..10)
    ) {
        // Create rules with different priorities
        let rules: Vec<Rule> = priorities
            .iter()
            .enumerate()
            .map(|(idx, &priority)| Rule {
                id: format!("rule.{}", idx + 1),
                name: format!("Rule {}", idx + 1),
                conditions: vec![Condition::Equals {
                    field: field.clone(),
                    value: value.clone(),
                }],
                actions: vec![Action::Allow],
                priority,
            })
            .collect();

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let results = evaluate_rules_with_priority(&rules, &context);

        // All rules should match
        prop_assert_eq!(results.len(), rules.len());

        // Verify priority ordering (highest first)
        for i in 0..results.len() - 1 {
            let current_rule = rules.iter().find(|r| r.id == results[i].rule_id).unwrap();
            let next_rule = rules.iter().find(|r| r.id == results[i + 1].rule_id).unwrap();

            prop_assert!(
                current_rule.priority >= next_rule.priority,
                "Rules must be ordered by priority (highest first)"
            );
        }
    }
}

proptest! {
    #[test]
    fn property_rule_priority_ordering_equal_priorities(
        field in field_name_strategy(),
        value in json_value_strategy(),
        count in 3usize..10
    ) {
        // Create rules with equal priorities
        let rules: Vec<Rule> = (0..count)
            .map(|idx| Rule {
                id: format!("rule.{}", idx + 1),
                name: format!("Rule {}", idx + 1),
                conditions: vec![Condition::Equals {
                    field: field.clone(),
                    value: value.clone(),
                }],
                actions: vec![Action::Allow],
                priority: 10, // All have same priority
            })
            .collect();

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let results = evaluate_rules_with_priority(&rules, &context);

        // All rules should match
        prop_assert_eq!(results.len(), rules.len());

        // Verify definition order is maintained when priorities are equal
        for (idx, result) in results.iter().enumerate() {
            prop_assert_eq!(
                &result.rule_id,
                &format!("rule.{}", idx + 1),
                "Rules with equal priority must maintain definition order"
            );
        }
    }
}

proptest! {
    #[test]
    fn property_rule_priority_ordering_mixed(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        // Create rules with mixed priorities (some equal, some different)
        let rules = vec![
            Rule {
                id: "rule.1".to_string(),
                name: "Low Priority 1".to_string(),
                conditions: vec![Condition::Equals {
                    field: field.clone(),
                    value: value.clone(),
                }],
                actions: vec![Action::Allow],
                priority: 5,
            },
            Rule {
                id: "rule.2".to_string(),
                name: "High Priority".to_string(),
                conditions: vec![Condition::Equals {
                    field: field.clone(),
                    value: value.clone(),
                }],
                actions: vec![Action::Allow],
                priority: 20,
            },
            Rule {
                id: "rule.3".to_string(),
                name: "Low Priority 2".to_string(),
                conditions: vec![Condition::Equals {
                    field: field.clone(),
                    value: value.clone(),
                }],
                actions: vec![Action::Allow],
                priority: 5,
            },
            Rule {
                id: "rule.4".to_string(),
                name: "Medium Priority".to_string(),
                conditions: vec![Condition::Equals {
                    field: field.clone(),
                    value: value.clone(),
                }],
                actions: vec![Action::Allow],
                priority: 10,
            },
        ];

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let results = evaluate_rules_with_priority(&rules, &context);

        // All rules should match
        prop_assert_eq!(results.len(), 4);

        // Verify ordering: rule.2 (20), rule.4 (10), rule.1 (5), rule.3 (5)
        prop_assert_eq!(&results[0].rule_id, "rule.2", "Highest priority first");
        prop_assert_eq!(&results[1].rule_id, "rule.4", "Medium priority second");
        prop_assert_eq!(&results[2].rule_id, "rule.1", "Low priority, first in definition");
        prop_assert_eq!(&results[3].rule_id, "rule.3", "Low priority, second in definition");
    }
}

proptest! {
    #[test]
    fn property_rule_priority_ordering_only_matched_rules(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        // Create rules where some match and some don't
        let rules = vec![
            Rule {
                id: "rule.1".to_string(),
                name: "Matches".to_string(),
                conditions: vec![Condition::Equals {
                    field: field.clone(),
                    value: value.clone(),
                }],
                actions: vec![Action::Allow],
                priority: 10,
            },
            Rule {
                id: "rule.2".to_string(),
                name: "Does Not Match".to_string(),
                conditions: vec![Condition::Equals {
                    field: "nonexistent".to_string(),
                    value: json!("impossible"),
                }],
                actions: vec![Action::Deny],
                priority: 20,
            },
            Rule {
                id: "rule.3".to_string(),
                name: "Matches".to_string(),
                conditions: vec![Condition::Equals {
                    field: field.clone(),
                    value: value.clone(),
                }],
                actions: vec![Action::Allow],
                priority: 5,
            },
        ];

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let results = evaluate_rules_with_priority(&rules, &context);

        // Only matching rules should be in results
        prop_assert_eq!(results.len(), 2);
        prop_assert_eq!(&results[0].rule_id, "rule.1");
        prop_assert_eq!(&results[1].rule_id, "rule.3");

        // Verify they're in priority order
        prop_assert!(results[0].matched);
        prop_assert!(results[1].matched);
    }
}

// **Additional Property: Rule Evaluation Determinism**
proptest! {
    #[test]
    fn property_rule_evaluation_determinism(
        rule in rule_strategy(),
        context in context_strategy()
    ) {
        // Evaluate multiple times
        let result1 = evaluate_rule(&rule, &context);
        let result2 = evaluate_rule(&rule, &context);
        let result3 = evaluate_rule(&rule, &context);

        // All results must be identical
        prop_assert_eq!(&result1, &result2, "First and second evaluation must match");
        prop_assert_eq!(&result2, &result3, "Second and third evaluation must match");
    }
}

// **Additional Property: Empty Actions**
proptest! {
    #[test]
    fn property_empty_actions(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        let rule = Rule {
            id: "rule.1".to_string(),
            name: "No Actions Rule".to_string(),
            conditions: vec![Condition::Equals {
                field: field.clone(),
                value: value.clone(),
            }],
            actions: vec![],
            priority: 10,
        };

        let mut context = EvaluationContext::new();
        context.insert(field, value);

        let result = evaluate_rule(&rule, &context);

        prop_assert!(result.matched, "Rule should match");
        prop_assert_eq!(result.actions.len(), 0, "No actions should be returned");

        // Execute empty actions
        let exec_results = execute_actions(&result);
        prop_assert_eq!(exec_results.len(), 0, "No execution results for empty actions");
    }
}

// **Additional Property: Rule ID Preservation**
proptest! {
    #[test]
    fn property_rule_id_preservation(
        rule in rule_strategy(),
        context in context_strategy()
    ) {
        let result = evaluate_rule(&rule, &context);

        prop_assert_eq!(
            &result.rule_id,
            &rule.id,
            "Result must preserve the rule ID"
        );
    }
}
