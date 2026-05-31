use chrono::Utc;
use hudhudscript_governance::audit::AuditLogger;
use hudhudscript_governance::enforcement::{
    check_law_compliance, enforce_constitution, enforce_with_model, evaluate_condition,
    EnforcementResult, EvaluationContext,
};
use hudhudscript_governance::*;
use serde_json::json;
use std::collections::HashMap;

fn make_test_constitution() -> Constitution {
    let mut laws = HashMap::new();

    laws.insert(
        "cons1.law1".to_string(),
        Law {
            id: "cons1.law1".to_string(),
            constitution_id: "cons.1".to_string(),
            name: "Data Size Limit".to_string(),
            description: "Data must be under 1000 bytes".to_string(),
            enforcement_level: EnforcementLevel::Mandatory,
            conditions: vec![Condition::LessThan {
                field: "data_size".to_string(),
                value: 1000.0,
            }],
        },
    );

    laws.insert(
        "cons1.law2".to_string(),
        Law {
            id: "cons1.law2".to_string(),
            constitution_id: "cons.1".to_string(),
            name: "High Priority Recommended".to_string(),
            description: "Actions should have high priority".to_string(),
            enforcement_level: EnforcementLevel::Advisory,
            conditions: vec![Condition::Equals {
                field: "priority".to_string(),
                value: json!("high"),
            }],
        },
    );

    laws.insert(
        "cons1.law3".to_string(),
        Law {
            id: "cons1.law3".to_string(),
            constitution_id: "cons.1".to_string(),
            name: "Metadata Optional".to_string(),
            description: "Actions may include metadata".to_string(),
            enforcement_level: EnforcementLevel::Optional,
            conditions: vec![Condition::Equals {
                field: "has_metadata".to_string(),
                value: json!(true),
            }],
        },
    );

    Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: Some("A test constitution".to_string()),
        laws,
        created_at: Utc::now(),
        version: 1,
    }
}

// ─── EvaluationContext tests ──────────────────────────────────────────────────

#[test]
fn test_evaluation_context_new() {
    let ctx = EvaluationContext::new();
    assert!(ctx.get("anything").is_none());
}

#[test]
fn test_evaluation_context_default() {
    let ctx = EvaluationContext::default();
    assert!(ctx.get("anything").is_none());
}

#[test]
fn test_insert_and_get() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("key".to_string(), json!(42));
    assert_eq!(ctx.get("key"), Some(&json!(42)));
}

#[test]
fn test_insert_overwrite() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("key".to_string(), json!(1));
    ctx.insert("key".to_string(), json!(2));
    assert_eq!(ctx.get("key"), Some(&json!(2)));
}

#[test]
fn test_get_missing_returns_none() {
    let ctx = EvaluationContext::new();
    assert!(ctx.get("missing").is_none());
}

#[test]
fn test_multiple_fields() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!("hello"));
    ctx.insert("b".to_string(), json!(true));
    ctx.insert("c".to_string(), json!(3.14));

    assert_eq!(ctx.get("a"), Some(&json!("hello")));
    assert_eq!(ctx.get("b"), Some(&json!(true)));
    assert_eq!(ctx.get("c"), Some(&json!(3.14)));
}

#[test]
fn test_context_equality() {
    let mut ctx1 = EvaluationContext::new();
    ctx1.insert("x".to_string(), json!(1));

    let mut ctx2 = EvaluationContext::new();
    ctx2.insert("x".to_string(), json!(1));

    assert_eq!(ctx1, ctx2);
}

#[test]
fn test_context_clone() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("x".to_string(), json!(42));

    let cloned = ctx.clone();
    assert_eq!(cloned.get("x"), Some(&json!(42)));
}

// ─── EnforcementResult tests ──────────────────────────────────────────────────

#[test]
fn test_enforcement_result_allowed() {
    let result = EnforcementResult::allowed();
    assert!(result.allowed);
    assert!(result.violations.is_empty());
    assert!(result.advisory_violations.is_empty());
    assert_eq!(result.message, "Action complies with constitution");
}

#[test]
fn test_enforcement_result_denied() {
    let violations = vec!["law.1".to_string(), "law.2".to_string()];
    let result = EnforcementResult::denied(violations.clone());
    assert!(!result.allowed);
    assert_eq!(result.violations, violations);
    assert!(result.advisory_violations.is_empty());
    assert_eq!(result.message, "Action violates mandatory laws");
}

#[test]
fn test_with_advisory_violations() {
    let result =
        EnforcementResult::allowed().with_advisory_violations(vec!["advisory.1".to_string()]);
    assert!(result.allowed);
    assert_eq!(result.advisory_violations.len(), 1);
    assert_eq!(result.advisory_violations[0], "advisory.1");
}

#[test]
fn test_with_message() {
    let result = EnforcementResult::allowed().with_message("Custom message".to_string());
    assert_eq!(result.message, "Custom message");
}

#[test]
fn test_denied_with_advisory_and_message() {
    let result = EnforcementResult::denied(vec!["law.1".to_string()])
        .with_advisory_violations(vec!["adv.1".to_string()])
        .with_message("Custom denial".to_string());
    assert!(!result.allowed);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.advisory_violations.len(), 1);
    assert_eq!(result.message, "Custom denial");
}

#[test]
fn test_enforcement_result_equality() {
    let r1 = EnforcementResult::allowed();
    let r2 = EnforcementResult::allowed();
    assert_eq!(r1, r2);

    let r3 = EnforcementResult::denied(vec!["law.1".to_string()]);
    assert_ne!(r1, r3);
}

// ─── evaluate_condition tests ─────────────────────────────────────────────────

#[test]
fn test_equals_match() {
    let cond = Condition::Equals {
        field: "status".to_string(),
        value: json!("active"),
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_equals_mismatch() {
    let cond = Condition::Equals {
        field: "status".to_string(),
        value: json!("active"),
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("inactive"));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_equals_missing_field() {
    let cond = Condition::Equals {
        field: "status".to_string(),
        value: json!("active"),
    };
    let ctx = EvaluationContext::new();
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_not_equals_match() {
    let cond = Condition::NotEquals {
        field: "status".to_string(),
        value: json!("active"),
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("inactive"));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_not_equals_same_value() {
    let cond = Condition::NotEquals {
        field: "status".to_string(),
        value: json!("active"),
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_not_equals_missing_field() {
    let cond = Condition::NotEquals {
        field: "status".to_string(),
        value: json!("active"),
    };
    let ctx = EvaluationContext::new();
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_greater_than_true() {
    let cond = Condition::GreaterThan {
        field: "count".to_string(),
        value: 10.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(15));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_greater_than_false() {
    let cond = Condition::GreaterThan {
        field: "count".to_string(),
        value: 10.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(5));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_greater_than_equal_returns_false() {
    let cond = Condition::GreaterThan {
        field: "count".to_string(),
        value: 10.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(10));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_greater_than_non_numeric() {
    let cond = Condition::GreaterThan {
        field: "count".to_string(),
        value: 10.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!("not_a_number"));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_greater_than_missing_field() {
    let cond = Condition::GreaterThan {
        field: "count".to_string(),
        value: 10.0,
    };
    let ctx = EvaluationContext::new();
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_less_than_true() {
    let cond = Condition::LessThan {
        field: "size".to_string(),
        value: 100.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("size".to_string(), json!(50));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_less_than_false() {
    let cond = Condition::LessThan {
        field: "size".to_string(),
        value: 100.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("size".to_string(), json!(150));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_less_than_non_numeric() {
    let cond = Condition::LessThan {
        field: "size".to_string(),
        value: 100.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("size".to_string(), json!("big"));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_less_than_missing_field() {
    let cond = Condition::LessThan {
        field: "size".to_string(),
        value: 100.0,
    };
    let ctx = EvaluationContext::new();
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_between_inside() {
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!(30));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_between_at_boundaries() {
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!(18));
    assert!(evaluate_condition(&cond, &ctx));

    let mut ctx2 = EvaluationContext::new();
    ctx2.insert("age".to_string(), json!(65));
    assert!(evaluate_condition(&cond, &ctx2));
}

#[test]
fn test_between_outside() {
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!(10));
    assert!(!evaluate_condition(&cond, &ctx));

    let mut ctx2 = EvaluationContext::new();
    ctx2.insert("age".to_string(), json!(70));
    assert!(!evaluate_condition(&cond, &ctx2));
}

#[test]
fn test_between_non_numeric() {
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!("thirty"));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_between_missing_field() {
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    let ctx = EvaluationContext::new();
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_in_match() {
    let cond = Condition::In {
        field: "color".to_string(),
        values: vec![json!("red"), json!("green"), json!("blue")],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("color".to_string(), json!("green"));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_in_no_match() {
    let cond = Condition::In {
        field: "color".to_string(),
        values: vec![json!("red"), json!("green"), json!("blue")],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("color".to_string(), json!("yellow"));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_in_missing_field() {
    let cond = Condition::In {
        field: "color".to_string(),
        values: vec![json!("red")],
    };
    let ctx = EvaluationContext::new();
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_and_all_true() {
    let cond = Condition::And(vec![
        Condition::Equals {
            field: "a".to_string(),
            value: json!(1),
        },
        Condition::Equals {
            field: "b".to_string(),
            value: json!(2),
        },
    ]);
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(1));
    ctx.insert("b".to_string(), json!(2));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_and_one_false() {
    let cond = Condition::And(vec![
        Condition::Equals {
            field: "a".to_string(),
            value: json!(1),
        },
        Condition::Equals {
            field: "b".to_string(),
            value: json!(2),
        },
    ]);
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(1));
    ctx.insert("b".to_string(), json!(99));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_and_empty() {
    let cond = Condition::And(vec![]);
    let ctx = EvaluationContext::new();
    // Empty And is vacuously true
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_or_one_true() {
    let cond = Condition::Or(vec![
        Condition::Equals {
            field: "a".to_string(),
            value: json!(1),
        },
        Condition::Equals {
            field: "b".to_string(),
            value: json!(2),
        },
    ]);
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(99));
    ctx.insert("b".to_string(), json!(2));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn test_or_all_false() {
    let cond = Condition::Or(vec![
        Condition::Equals {
            field: "a".to_string(),
            value: json!(1),
        },
        Condition::Equals {
            field: "b".to_string(),
            value: json!(2),
        },
    ]);
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(99));
    ctx.insert("b".to_string(), json!(99));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_or_empty() {
    let cond = Condition::Or(vec![]);
    let ctx = EvaluationContext::new();
    // Empty Or is vacuously false
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_not_true_becomes_false() {
    let cond = Condition::Not(Box::new(Condition::Equals {
        field: "a".to_string(),
        value: json!(1),
    }));
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(1));
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn test_not_false_becomes_true() {
    let cond = Condition::Not(Box::new(Condition::Equals {
        field: "a".to_string(),
        value: json!(1),
    }));
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(2));
    assert!(evaluate_condition(&cond, &ctx));
}

// ─── enforce_constitution tests ───────────────────────────────────────────────

#[test]
fn test_enforce_constitution_all_compliant() {
    let constitution = make_test_constitution();

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(500));
    context.insert("priority".to_string(), json!("high"));
    context.insert("has_metadata".to_string(), json!(true));

    let result = enforce_constitution(&constitution, &context, None);

    assert!(result.allowed);
    assert_eq!(result.violations.len(), 0);
    assert_eq!(result.advisory_violations.len(), 0);
}

#[test]
fn test_enforce_constitution_mandatory_violation() {
    let constitution = make_test_constitution();

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(1500)); // Violates mandatory law
    context.insert("priority".to_string(), json!("high"));

    let result = enforce_constitution(&constitution, &context, None);

    assert!(!result.allowed);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0], "cons1.law1");
    assert_eq!(result.advisory_violations.len(), 0);
}

#[test]
fn test_enforce_constitution_advisory_violation_does_not_block() {
    let constitution = make_test_constitution();

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(500)); // Complies with mandatory
    context.insert("priority".to_string(), json!("low")); // Violates advisory

    let result = enforce_constitution(&constitution, &context, None);

    // Action should be allowed despite advisory violation
    assert!(result.allowed);
    assert_eq!(result.violations.len(), 0);
    assert_eq!(result.advisory_violations.len(), 1);
    assert_eq!(result.advisory_violations[0], "cons1.law2");
}

#[test]
fn test_enforce_constitution_with_audit_logging() {
    let constitution = make_test_constitution();
    let audit_logger = AuditLogger::new();

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(500));
    context.insert("priority".to_string(), json!("high"));
    context.insert("has_metadata".to_string(), json!(true));

    let result = enforce_constitution(&constitution, &context, Some(&audit_logger));
    assert!(result.allowed);
    assert_eq!(audit_logger.count(), 1);
}

#[test]
fn test_enforce_constitution_audit_logging_on_denial() {
    let constitution = make_test_constitution();
    let audit_logger = AuditLogger::new();

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(1500)); // violates mandatory

    let result = enforce_constitution(&constitution, &context, Some(&audit_logger));
    assert!(!result.allowed);
    assert_eq!(audit_logger.count(), 1);
}

#[test]
fn test_check_law_compliance_no_conditions() {
    let law = Law {
        id: "cons1.law1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Empty Law".to_string(),
        description: "A law with no conditions".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };

    let context = EvaluationContext::new();
    assert!(check_law_compliance(&law, &context));
}

#[test]
fn test_enforce_constitution_optional_violation_ignored() {
    let constitution = make_test_constitution();

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(500)); // complies with mandatory
    context.insert("priority".to_string(), json!("high")); // complies with advisory
                                                           // has_metadata not set — violates optional law

    let result = enforce_constitution(&constitution, &context, None);

    // Optional violations don't appear in advisory_violations
    assert!(result.allowed);
    assert!(result.violations.is_empty());
}

#[test]
fn test_enforce_constitution_empty_laws() {
    let constitution = Constitution {
        id: "cons.empty".to_string(),
        name: "Empty".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let context = EvaluationContext::new();
    let result = enforce_constitution(&constitution, &context, None);
    assert!(result.allowed);
    assert!(result.violations.is_empty());
}

#[test]
fn test_check_law_compliance_with_conditions() {
    let law = Law {
        id: "cons1.law1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Test Law".to_string(),
        description: "A test law".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
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
    };

    // Compliant context
    let mut context1 = EvaluationContext::new();
    context1.insert("status".to_string(), json!("active"));
    context1.insert("count".to_string(), json!(15));
    assert!(check_law_compliance(&law, &context1));

    // Non-compliant context (first condition fails)
    let mut context2 = EvaluationContext::new();
    context2.insert("status".to_string(), json!("inactive"));
    context2.insert("count".to_string(), json!(15));
    assert!(!check_law_compliance(&law, &context2));
}

// ─── enforce_with_model tests (from models.rs) ────────────────────────────────

fn make_single_law_constitution() -> Constitution {
    let mut laws = HashMap::new();
    laws.insert(
        "cons1.law1".to_string(),
        Law {
            id: "cons1.law1".to_string(),
            constitution_id: "cons.1".to_string(),
            name: "Data Size Limit".to_string(),
            description: "Data must be under 1000 bytes".to_string(),
            enforcement_level: EnforcementLevel::Mandatory,
            conditions: vec![Condition::LessThan {
                field: "data_size".to_string(),
                value: 1000.0,
            }],
        },
    );
    Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: Some("A test constitution".to_string()),
        laws,
        created_at: Utc::now(),
        version: 1,
    }
}

#[test]
fn test_enforce_with_democracy_model() {
    let constitution = make_single_law_constitution();
    let democracy = GovernanceModel::democracy();

    // Compliant action should be allowed
    let mut context1 = EvaluationContext::new();
    context1.insert("data_size".to_string(), json!(500));

    let result1 = enforce_with_model(
        &constitution,
        &context1,
        &democracy,
        &AgentRole::Member,
        None,
    );
    assert!(result1.allowed);

    // Non-compliant action should be denied (strict enforcement)
    let mut context2 = EvaluationContext::new();
    context2.insert("data_size".to_string(), json!(1500));

    let result2 = enforce_with_model(
        &constitution,
        &context2,
        &democracy,
        &AgentRole::Member,
        None,
    );
    assert!(!result2.allowed);
    assert_eq!(result2.violations.len(), 1);
}

#[test]
fn test_enforce_with_monarchy_model() {
    let constitution = make_single_law_constitution();
    let monarchy = GovernanceModel::monarchy("king".to_string());

    // King role bypasses constitution
    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(1500)); // Violates law

    let result = enforce_with_model(
        &constitution,
        &context,
        &monarchy,
        &AgentRole::Custom("king".to_string()),
        None,
    );
    assert!(result.allowed);
    assert!(result.message.contains("bypass privilege"));

    // Non-king role should also be allowed because monarchy has 0.0 constitution compliance
    let result2 = enforce_with_model(&constitution, &context, &monarchy, &AgentRole::Member, None);
    assert!(result2.allowed);
    assert!(result2.message.contains("compliance not required"));
}

#[test]
fn test_enforce_with_anarchy_model() {
    let constitution = make_single_law_constitution();
    let anarchy = GovernanceModel::anarchy();

    // Even violations are allowed in anarchy
    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(1500)); // Violates law

    let result = enforce_with_model(&constitution, &context, &anarchy, &AgentRole::Member, None);
    assert!(result.allowed);
    assert!(result.message.contains("compliance not required"));
}

#[test]
fn test_enforce_with_high_law_flexibility() {
    let constitution = make_single_law_constitution();

    // Create model with high law flexibility (>0.7)
    let flexible_model = GovernanceModel::new(
        GovernanceModelType::Hybrid,
        1.0, // Full constitution compliance
        0.8, // High law flexibility
        1.0, // Full rule enforcement
        DecisionStrategy::Majority,
    );

    // Violation should be downgraded to advisory
    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(1500)); // Violates mandatory law

    let result = enforce_with_model(
        &constitution,
        &context,
        &flexible_model,
        &AgentRole::Member,
        None,
    );

    // Should be allowed because mandatory law was downgraded to advisory
    assert!(result.allowed);
    assert_eq!(result.violations.len(), 0);
    assert_eq!(result.advisory_violations.len(), 1);
}

#[test]
fn test_enforce_with_low_enforcement_threshold() {
    let constitution = make_single_law_constitution();

    // Create model with low enforcement threshold (<0.5)
    let lenient_model = GovernanceModel::new(
        GovernanceModelType::Hybrid,
        1.0, // Full constitution compliance
        0.0, // No law flexibility
        0.3, // Low enforcement threshold
        DecisionStrategy::Majority,
    );

    // Violations should be ignored due to low enforcement
    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(1500)); // Violates mandatory law

    let result = enforce_with_model(
        &constitution,
        &context,
        &lenient_model,
        &AgentRole::Member,
        None,
    );

    // Should be allowed despite violations
    assert!(result.allowed);
    assert!(result.message.contains("low enforcement threshold"));
}

#[test]
fn test_enforce_with_special_role_privileges() {
    let constitution = make_single_law_constitution();

    // Create model with special role
    let mut model = GovernanceModel::new(
        GovernanceModelType::Hybrid,
        1.0,
        0.0,
        1.0,
        DecisionStrategy::Majority,
    );

    model.add_special_role(
        "admin".to_string(),
        RolePrivileges {
            bypass_constitution: true,
            modify_laws: true,
            override_rules: true,
            voting_weight: 5.0,
        },
    );

    // Admin role bypasses constitution
    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(1500)); // Violates law

    let result = enforce_with_model(
        &constitution,
        &context,
        &model,
        &AgentRole::Custom("admin".to_string()),
        None,
    );

    assert!(result.allowed);
    assert!(result.message.contains("bypass privilege"));
}

#[test]
fn test_enforce_with_model_audit_logging() {
    use hudhudscript_governance::audit::AuditEventData;

    let constitution = make_single_law_constitution();
    let democracy = GovernanceModel::democracy();
    let audit_logger = AuditLogger::new();

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(500));

    let result = enforce_with_model(
        &constitution,
        &context,
        &democracy,
        &AgentRole::Member,
        Some(&audit_logger),
    );

    assert!(result.allowed);
    assert_eq!(audit_logger.count(), 1);

    // Verify audit log contains governance model info
    let events = audit_logger.get_events();
    match &events[0].data {
        AuditEventData::Enforcement {
            action_description, ..
        } => {
            assert!(action_description.contains("Democracy"));
        }
        other => unreachable!("Expected Enforcement event, got: {:?}", other),
    }
}
