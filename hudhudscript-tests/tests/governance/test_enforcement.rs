use hudhudscript_governance::audit::{AuditEventType, AuditLogger};
use hudhudscript_governance::enforcement::{
    check_law_compliance, enforce_constitution, EnforcementResult, EvaluationContext,
};
use hudhudscript_governance::*;
use serde_json::json;
use std::collections::HashMap;

fn make_constitution() -> Constitution {
    let mut laws = HashMap::new();
    laws.insert(
        "cons1.law1".to_string(),
        Law {
            id: "cons1.law1".to_string(),
            constitution_id: "cons.1".to_string(),
            name: "Size Limit".to_string(),
            description: "data_size < 1000".to_string(),
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
            name: "Priority".to_string(),
            description: "priority == high".to_string(),
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
            name: "Optional Meta".to_string(),
            description: "has_metadata == true".to_string(),
            enforcement_level: EnforcementLevel::Optional,
            conditions: vec![Condition::Equals {
                field: "has_metadata".to_string(),
                value: json!(true),
            }],
        },
    );
    Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws,
        created_at: chrono::Utc::now(),
        version: 1,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// enforce_constitution — all compliant
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enforce_all_compliant_allowed() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(500));
    ctx.insert("priority".to_string(), json!("high"));
    ctx.insert("has_metadata".to_string(), json!(true));
    let result = enforce_constitution(&c, &ctx, None);
    assert!(result.allowed);
}

#[test]
fn enforce_all_compliant_no_violations() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(500));
    ctx.insert("priority".to_string(), json!("high"));
    ctx.insert("has_metadata".to_string(), json!(true));
    let result = enforce_constitution(&c, &ctx, None);
    assert_eq!(result.violations.len(), 0);
    assert_eq!(result.advisory_violations.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// enforce_constitution — mandatory violation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enforce_mandatory_violation_denies() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(1500));
    ctx.insert("priority".to_string(), json!("high"));
    let result = enforce_constitution(&c, &ctx, None);
    assert!(!result.allowed);
}

#[test]
fn enforce_mandatory_violation_lists_law() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(1500));
    ctx.insert("priority".to_string(), json!("high"));
    let result = enforce_constitution(&c, &ctx, None);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0], "cons1.law1");
}

// ═══════════════════════════════════════════════════════════════════════════════
// enforce_constitution — advisory violation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enforce_advisory_violation_still_allowed() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(500));
    ctx.insert("priority".to_string(), json!("low"));
    let result = enforce_constitution(&c, &ctx, None);
    assert!(result.allowed);
}

#[test]
fn enforce_advisory_violation_listed() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(500));
    ctx.insert("priority".to_string(), json!("low"));
    let result = enforce_constitution(&c, &ctx, None);
    assert_eq!(result.advisory_violations.len(), 1);
    assert_eq!(result.advisory_violations[0], "cons1.law2");
}

#[test]
fn enforce_advisory_no_mandatory_violations() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(500));
    ctx.insert("priority".to_string(), json!("low"));
    let result = enforce_constitution(&c, &ctx, None);
    assert_eq!(result.violations.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// enforce_constitution — optional violation ignored
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enforce_optional_violation_ignored() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(500));
    ctx.insert("priority".to_string(), json!("high"));
    // has_metadata missing -> optional law violated but ignored
    let result = enforce_constitution(&c, &ctx, None);
    assert!(result.allowed);
    assert!(result.violations.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// enforce_constitution — empty constitution
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enforce_empty_constitution_allowed() {
    let c = Constitution {
        id: "empty".to_string(),
        name: "Empty".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: chrono::Utc::now(),
        version: 1,
    };
    let ctx = EvaluationContext::new();
    let result = enforce_constitution(&c, &ctx, None);
    assert!(result.allowed);
    assert!(result.violations.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// enforce_constitution — with audit logger
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enforce_with_audit_logger_logs_event() {
    let c = make_constitution();
    let logger = AuditLogger::new();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(500));
    ctx.insert("priority".to_string(), json!("high"));
    ctx.insert("has_metadata".to_string(), json!(true));
    let result = enforce_constitution(&c, &ctx, Some(&logger));
    assert!(result.allowed);
    assert_eq!(logger.count(), 1);
}

#[test]
fn enforce_with_audit_logger_on_denial() {
    let c = make_constitution();
    let logger = AuditLogger::new();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(1500));
    let result = enforce_constitution(&c, &ctx, Some(&logger));
    assert!(!result.allowed);
    assert_eq!(logger.count(), 1);
}

#[test]
fn enforce_without_audit_logger_no_crash() {
    let c = make_constitution();
    let mut ctx = EvaluationContext::new();
    ctx.insert("data_size".to_string(), json!(500));
    let result = enforce_constitution(&c, &ctx, None);
    assert!(result.allowed);
}

// ═══════════════════════════════════════════════════════════════════════════════
// check_law_compliance
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn check_law_no_conditions_complies() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Empty".to_string(),
        description: "no conditions".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };
    let ctx = EvaluationContext::new();
    assert!(check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_equals_pass() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Active".to_string(),
        description: "status active".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    assert!(check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_equals_fail() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Active".to_string(),
        description: "status active".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("inactive"));
    assert!(!check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_less_than_pass() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Limit".to_string(),
        description: "size < 100".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::LessThan {
            field: "size".to_string(),
            value: 100.0,
        }],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("size".to_string(), json!(50));
    assert!(check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_less_than_fail() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Limit".to_string(),
        description: "size < 100".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::LessThan {
            field: "size".to_string(),
            value: 100.0,
        }],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("size".to_string(), json!(200));
    assert!(!check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_greater_than_pass() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Min".to_string(),
        description: "count > 5".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::GreaterThan {
            field: "count".to_string(),
            value: 5.0,
        }],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(10));
    assert!(check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_missing_field_fails() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Active".to_string(),
        description: "status active".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
    };
    let ctx = EvaluationContext::new();
    assert!(!check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_between_pass() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Range".to_string(),
        description: "10 <= x <= 20".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::Between {
            field: "x".to_string(),
            min: 10.0,
            max: 20.0,
        }],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("x".to_string(), json!(15));
    assert!(check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_between_boundary_min() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Range".to_string(),
        description: "10 <= x <= 20".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::Between {
            field: "x".to_string(),
            min: 10.0,
            max: 20.0,
        }],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("x".to_string(), json!(10));
    assert!(check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_between_boundary_max() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Range".to_string(),
        description: "10 <= x <= 20".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::Between {
            field: "x".to_string(),
            min: 10.0,
            max: 20.0,
        }],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("x".to_string(), json!(20));
    assert!(check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_multiple_conditions_all_must_pass() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Multi".to_string(),
        description: "status=active AND count>5".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![
            Condition::Equals {
                field: "status".to_string(),
                value: json!("active"),
            },
            Condition::GreaterThan {
                field: "count".to_string(),
                value: 5.0,
            },
        ],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    ctx.insert("count".to_string(), json!(10));
    assert!(check_law_compliance(&law, &ctx));
}

#[test]
fn check_law_multiple_conditions_one_fails() {
    let law = Law {
        id: "l1".to_string(),
        constitution_id: "c1".to_string(),
        name: "Multi".to_string(),
        description: "status=active AND count>5".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![
            Condition::Equals {
                field: "status".to_string(),
                value: json!("active"),
            },
            Condition::GreaterThan {
                field: "count".to_string(),
                value: 5.0,
            },
        ],
    };
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    ctx.insert("count".to_string(), json!(3));
    assert!(!check_law_compliance(&law, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EnforcementResult constructors
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enforcement_result_allowed_constructor() {
    let r = EnforcementResult::allowed();
    assert!(r.allowed);
    assert!(r.violations.is_empty());
    assert_eq!(r.message, "Action complies with constitution");
}

#[test]
fn enforcement_result_denied_constructor() {
    let r = EnforcementResult::denied(vec!["law.1".to_string()]);
    assert!(!r.allowed);
    assert_eq!(r.violations, vec!["law.1".to_string()]);
    assert_eq!(r.message, "Action violates mandatory laws");
}

#[test]
fn enforcement_result_denied_multiple_violations() {
    let r = EnforcementResult::denied(vec!["law.1".to_string(), "law.2".to_string()]);
    assert!(!r.allowed);
    assert_eq!(r.violations.len(), 2);
}

#[test]
fn enforcement_result_with_advisory() {
    let r = EnforcementResult::allowed().with_advisory_violations(vec!["adv.1".to_string()]);
    assert_eq!(r.advisory_violations.len(), 1);
    assert_eq!(r.advisory_violations[0], "adv.1");
}

#[test]
fn enforcement_result_with_message() {
    let r = EnforcementResult::allowed().with_message("custom".to_string());
    assert_eq!(r.message, "custom");
}

#[test]
fn enforcement_result_chained() {
    let r = EnforcementResult::denied(vec!["law.1".to_string()])
        .with_advisory_violations(vec!["adv.1".to_string()])
        .with_message("combined".to_string());
    assert!(!r.allowed);
    assert_eq!(r.violations.len(), 1);
    assert_eq!(r.advisory_violations.len(), 1);
    assert_eq!(r.message, "combined");
}

#[test]
fn enforcement_result_eq() {
    let r1 = EnforcementResult::allowed();
    let r2 = EnforcementResult::allowed();
    assert_eq!(r1, r2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// AuditLogger
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn audit_logger_new_empty() {
    let logger = AuditLogger::new();
    assert_eq!(logger.count(), 0);
}

#[test]
fn audit_logger_default_empty() {
    let logger = AuditLogger::default();
    assert_eq!(logger.count(), 0);
}

#[test]
fn audit_logger_log_enforcement_decision() {
    let logger = AuditLogger::new();
    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "test action".to_string(),
        true,
        vec![],
        vec![],
    );
    assert_eq!(logger.count(), 1);
}

#[test]
fn audit_logger_log_constitution_modification() {
    let logger = AuditLogger::new();
    logger.log_constitution_modification(
        "cons.1".to_string(),
        "add_law".to_string(),
        1,
        2,
        "Added law".to_string(),
    );
    assert_eq!(logger.count(), 1);
}

#[test]
fn audit_logger_log_agent_role_change() {
    let logger = AuditLogger::new();
    logger.log_agent_role_change(
        "agent1".to_string(),
        Some(AgentRole::Member),
        AgentRole::Judge,
        "c1".to_string(),
    );
    assert_eq!(logger.count(), 1);
}

#[test]
fn audit_logger_log_cache_access() {
    let logger = AuditLogger::new();
    logger.log_cache_access("resolve".to_string(), "cons.1".to_string(), true);
    assert_eq!(logger.count(), 1);
}

#[test]
fn audit_logger_get_events() {
    let logger = AuditLogger::new();
    logger.log_enforcement_decision("c1".to_string(), "test".to_string(), true, vec![], vec![]);
    let events = logger.get_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, AuditEventType::EnforcementDecision);
}

#[test]
fn audit_logger_get_events_by_type() {
    let logger = AuditLogger::new();
    logger.log_enforcement_decision("c1".to_string(), "test".to_string(), true, vec![], vec![]);
    logger.log_cache_access("resolve".to_string(), "key".to_string(), true);
    let enforcement_events = logger.get_events_by_type(AuditEventType::EnforcementDecision);
    assert_eq!(enforcement_events.len(), 1);
    let cache_events = logger.get_events_by_type(AuditEventType::CacheAccess);
    assert_eq!(cache_events.len(), 1);
}

#[test]
fn audit_logger_clear() {
    let logger = AuditLogger::new();
    logger.log_enforcement_decision("c1".to_string(), "test".to_string(), true, vec![], vec![]);
    assert_eq!(logger.count(), 1);
    logger.clear();
    assert_eq!(logger.count(), 0);
}

#[test]
fn audit_logger_export_to_json() {
    let logger = AuditLogger::new();
    logger.log_enforcement_decision("c1".to_string(), "test".to_string(), true, vec![], vec![]);
    let json = logger.export_to_json().unwrap();
    assert!(json.contains("EnforcementDecision"));
}

#[test]
fn audit_logger_export_to_json_compact() {
    let logger = AuditLogger::new();
    logger.log_enforcement_decision("c1".to_string(), "test".to_string(), true, vec![], vec![]);
    let json = logger.export_to_json_compact().unwrap();
    assert!(json.contains("EnforcementDecision"));
    // Compact should not have pretty-print newlines in the same way
    assert!(!json.contains("\n  "));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvaluationContext (enforcement module's version)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enforcement_eval_context_new_empty() {
    let ctx = EvaluationContext::new();
    // No direct is_empty, but get returns None
    assert!(ctx.get("missing").is_none());
}

#[test]
fn enforcement_eval_context_insert_and_get() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("key".to_string(), json!(42));
    assert_eq!(ctx.get("key"), Some(&json!(42)));
}

#[test]
fn enforcement_eval_context_default() {
    let ctx = EvaluationContext::default();
    assert!(ctx.get("anything").is_none());
}
