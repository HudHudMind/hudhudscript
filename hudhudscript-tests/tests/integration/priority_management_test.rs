//! Integration test for rule priority management
//!
//! This test demonstrates the complete priority management functionality:
//! - Creating rules with priorities
//! - Modifying priorities
//! - Evaluating rules in priority order
//!
//! **Validates Requirements:** 23.1, 23.2, 23.3, 23.4, 23.5

use hudhudscript_governance::{Action, Condition, Rule};
use hudhudscript_rules::context::EvaluationContext;
use hudhudscript_rules::rule::evaluate_rules_with_priority;
use serde_json::json;

#[test]
fn test_priority_management_workflow() {
    // Create rules with different priorities
    let mut rule1 = Rule::new(
        "rule.1".to_string(),
        "Low Priority Rule".to_string(),
        vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        vec![Action::Notify {
            message: "Low priority action".to_string(),
        }],
        5, // Low priority
    );

    let mut rule2 = Rule::new(
        "rule.2".to_string(),
        "High Priority Rule".to_string(),
        vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        vec![Action::Notify {
            message: "High priority action".to_string(),
        }],
        20, // High priority
    );

    let mut rule3 = Rule::new(
        "rule.3".to_string(),
        "Medium Priority Rule".to_string(),
        vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        vec![Action::Notify {
            message: "Medium priority action".to_string(),
        }],
        10, // Medium priority
    );

    // Verify initial priorities
    assert_eq!(rule1.get_priority(), 5);
    assert_eq!(rule2.get_priority(), 20);
    assert_eq!(rule3.get_priority(), 10);

    // Create evaluation context
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    // Evaluate rules - should be in priority order: rule2, rule3, rule1
    let rules = vec![rule1.clone(), rule2.clone(), rule3.clone()];
    let results = evaluate_rules_with_priority(&rules, &context);

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].rule_id, "rule.2"); // Highest priority (20)
    assert_eq!(results[1].rule_id, "rule.3"); // Medium priority (10)
    assert_eq!(results[2].rule_id, "rule.1"); // Lowest priority (5)

    // Modify priorities
    rule1.set_priority(25); // Make rule1 highest priority
    rule2.adjust_priority(-10); // Reduce rule2 priority to 10
    rule3.adjust_priority(5); // Increase rule3 priority to 15

    // Verify modified priorities
    assert_eq!(rule1.get_priority(), 25);
    assert_eq!(rule2.get_priority(), 10);
    assert_eq!(rule3.get_priority(), 15);

    // Evaluate again with modified priorities
    let rules = vec![rule1.clone(), rule2.clone(), rule3.clone()];
    let results = evaluate_rules_with_priority(&rules, &context);

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].rule_id, "rule.1"); // Now highest priority (25)
    assert_eq!(results[1].rule_id, "rule.3"); // Medium priority (15)
    assert_eq!(results[2].rule_id, "rule.2"); // Now lowest priority (10)
}

#[test]
fn test_priority_validation() {
    let rule = Rule::new(
        "rule.1".to_string(),
        "Test Rule".to_string(),
        vec![],
        vec![Action::Allow],
        10,
    );

    // Priority validation always succeeds for u32
    assert!(rule.validate_priority());

    // Test with zero priority
    let rule_zero = Rule::new(
        "rule.2".to_string(),
        "Zero Priority".to_string(),
        vec![],
        vec![Action::Allow],
        0,
    );
    assert!(rule_zero.validate_priority());

    // Test with maximum priority
    let rule_max = Rule::new(
        "rule.3".to_string(),
        "Max Priority".to_string(),
        vec![],
        vec![Action::Allow],
        u32::MAX,
    );
    assert!(rule_max.validate_priority());
}

#[test]
fn test_priority_modification_safety() {
    let mut rule = Rule::new(
        "rule.1".to_string(),
        "Test Rule".to_string(),
        vec![],
        vec![Action::Allow],
        10,
    );

    // Test underflow protection
    rule.adjust_priority(-20);
    assert_eq!(rule.get_priority(), 0); // Should saturate at 0

    // Test overflow protection
    rule.set_priority(u32::MAX - 5);
    rule.adjust_priority(10);
    assert_eq!(rule.get_priority(), u32::MAX); // Should saturate at MAX
}

#[test]
fn test_equal_priority_definition_order() {
    // Create three rules with the same priority
    let rule1 = Rule::new(
        "rule.1".to_string(),
        "First Rule".to_string(),
        vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        vec![Action::Allow],
        10,
    );

    let rule2 = Rule::new(
        "rule.2".to_string(),
        "Second Rule".to_string(),
        vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        vec![Action::Allow],
        10,
    );

    let rule3 = Rule::new(
        "rule.3".to_string(),
        "Third Rule".to_string(),
        vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        vec![Action::Allow],
        10,
    );

    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    // Evaluate rules - should maintain definition order when priorities are equal
    let rules = vec![rule1, rule2, rule3];
    let results = evaluate_rules_with_priority(&rules, &context);

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].rule_id, "rule.1"); // First in definition order
    assert_eq!(results[1].rule_id, "rule.2"); // Second in definition order
    assert_eq!(results[2].rule_id, "rule.3"); // Third in definition order
}
