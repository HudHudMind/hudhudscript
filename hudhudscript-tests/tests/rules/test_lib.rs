use hudhudscript_governance::{Action, Condition, Rule};
use hudhudscript_rules::*;
use serde_json::json;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// EvaluationContext — construction
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn context_new_is_empty() {
    let ctx = EvaluationContext::new();
    assert!(ctx.is_empty());
    assert_eq!(ctx.len(), 0);
}

#[test]
fn context_default_is_empty() {
    let ctx = EvaluationContext::default();
    assert!(ctx.is_empty());
}

#[test]
fn context_insert_and_get() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    assert_eq!(ctx.get("status"), Some(&json!("active")));
    assert!(ctx.contains_key("status"));
    assert_eq!(ctx.len(), 1);
}

#[test]
fn context_from_map() {
    let mut fields = HashMap::new();
    fields.insert("a".to_string(), json!(1));
    fields.insert("b".to_string(), json!(2));
    let ctx = EvaluationContext::from_map(fields);
    assert_eq!(ctx.len(), 2);
}

#[test]
fn context_from_hashmap_trait() {
    let mut fields = HashMap::new();
    fields.insert("x".to_string(), json!("y"));
    let ctx: EvaluationContext = fields.into();
    assert_eq!(ctx.len(), 1);
}

#[test]
fn context_into_map() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("key".to_string(), json!("val"));
    let map = ctx.into_map();
    assert_eq!(map.len(), 1);
    assert_eq!(map.get("key"), Some(&json!("val")));
}

#[test]
fn context_to_map_ref() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("key".to_string(), json!("val"));
    let map = ctx.to_map();
    assert_eq!(map.len(), 1);
}

#[test]
fn context_into_hashmap_trait() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    let map: HashMap<String, serde_json::Value> = ctx.into();
    assert_eq!(map.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvaluationContext — typed accessors
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn context_get_string() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("s".to_string(), json!("hello"));
    assert_eq!(ctx.get_string("s"), Some("hello"));
}

#[test]
fn context_get_string_wrong_type() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("n".to_string(), json!(42));
    assert_eq!(ctx.get_string("n"), None);
}

#[test]
fn context_get_number() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("n".to_string(), json!(42));
    assert_eq!(ctx.get_number("n"), Some(42.0));
}

#[test]
fn context_get_number_float() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("price".to_string(), json!(19.99));
    assert_eq!(ctx.get_number("price"), Some(19.99));
}

#[test]
fn context_get_number_wrong_type() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("s".to_string(), json!("text"));
    assert_eq!(ctx.get_number("s"), None);
}

#[test]
fn context_get_bool() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("b".to_string(), json!(true));
    assert_eq!(ctx.get_bool("b"), Some(true));
}

#[test]
fn context_get_bool_false() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("b".to_string(), json!(false));
    assert_eq!(ctx.get_bool("b"), Some(false));
}

#[test]
fn context_get_bool_wrong_type() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("s".to_string(), json!("true"));
    assert_eq!(ctx.get_bool("s"), None);
}

#[test]
fn context_get_array() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("arr".to_string(), json!([1, 2, 3]));
    assert_eq!(ctx.get_array("arr").map(|a| a.len()), Some(3));
}

#[test]
fn context_get_object() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("obj".to_string(), json!({"key": "val"}));
    assert!(ctx.get_object("obj").is_some());
}

#[test]
fn context_get_missing() {
    let ctx = EvaluationContext::new();
    assert!(ctx.get("missing").is_none());
    assert_eq!(ctx.get_string("missing"), None);
    assert_eq!(ctx.get_number("missing"), None);
    assert_eq!(ctx.get_bool("missing"), None);
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvaluationContext — validate_field_type
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn context_validate_field_type_string_ok() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("s".to_string(), json!("hello"));
    assert!(ctx.validate_field_type("s", "string").is_ok());
}

#[test]
fn context_validate_field_type_number_ok() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("n".to_string(), json!(42));
    assert!(ctx.validate_field_type("n", "number").is_ok());
}

#[test]
fn context_validate_field_type_boolean_ok() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("b".to_string(), json!(true));
    assert!(ctx.validate_field_type("b", "boolean").is_ok());
}

#[test]
fn context_validate_field_type_array_ok() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("arr".to_string(), json!([1, 2]));
    assert!(ctx.validate_field_type("arr", "array").is_ok());
}

#[test]
fn context_validate_field_type_object_ok() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("obj".to_string(), json!({"k": "v"}));
    assert!(ctx.validate_field_type("obj", "object").is_ok());
}

#[test]
fn context_validate_field_type_null_ok() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("nothing".to_string(), serde_json::Value::Null);
    assert!(ctx.validate_field_type("nothing", "null").is_ok());
}

#[test]
fn context_validate_field_type_wrong() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("s".to_string(), json!("hello"));
    let err = ctx.validate_field_type("s", "number").unwrap_err();
    assert!(err.contains("string"));
    assert!(err.contains("number"));
}

#[test]
fn context_validate_field_type_missing() {
    let ctx = EvaluationContext::new();
    let err = ctx.validate_field_type("missing", "string").unwrap_err();
    assert!(err.contains("not found"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// EvaluationContext — iterators
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn context_keys_iterator() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(1));
    ctx.insert("b".to_string(), json!(2));
    let keys: Vec<&String> = ctx.keys().collect();
    assert_eq!(keys.len(), 2);
}

#[test]
fn context_values_iterator() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(1));
    let values: Vec<&serde_json::Value> = ctx.values().collect();
    assert_eq!(values.len(), 1);
}

#[test]
fn context_iter() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(1));
    let entries: Vec<_> = ctx.iter().collect();
    assert_eq!(entries.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — Equals
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_equals_pass() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    let cond = Condition::Equals {
        field: "status".to_string(),
        value: json!("active"),
    };
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_equals_fail() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("inactive"));
    let cond = Condition::Equals {
        field: "status".to_string(),
        value: json!("active"),
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_equals_numeric() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(42));
    let cond = Condition::Equals {
        field: "count".to_string(),
        value: json!(42),
    };
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_equals_boolean() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("active".to_string(), json!(true));
    let cond = Condition::Equals {
        field: "active".to_string(),
        value: json!(true),
    };
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_equals_missing_field() {
    let ctx = EvaluationContext::new();
    let cond = Condition::Equals {
        field: "missing".to_string(),
        value: json!("val"),
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — NotEquals
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_not_equals_pass() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    let cond = Condition::NotEquals {
        field: "status".to_string(),
        value: json!("inactive"),
    };
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_not_equals_fail() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    let cond = Condition::NotEquals {
        field: "status".to_string(),
        value: json!("active"),
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_not_equals_missing_field() {
    let ctx = EvaluationContext::new();
    let cond = Condition::NotEquals {
        field: "missing".to_string(),
        value: json!("val"),
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — GreaterThan
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_greater_than_pass() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(15));
    assert!(evaluate_condition(
        &Condition::GreaterThan {
            field: "count".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

#[test]
fn condition_greater_than_fail() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(5));
    assert!(!evaluate_condition(
        &Condition::GreaterThan {
            field: "count".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

#[test]
fn condition_greater_than_equal_is_false() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(10));
    assert!(!evaluate_condition(
        &Condition::GreaterThan {
            field: "count".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

#[test]
fn condition_greater_than_non_numeric() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("name".to_string(), json!("alice"));
    assert!(!evaluate_condition(
        &Condition::GreaterThan {
            field: "name".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

#[test]
fn condition_greater_than_missing_field() {
    let ctx = EvaluationContext::new();
    assert!(!evaluate_condition(
        &Condition::GreaterThan {
            field: "missing".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — LessThan
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_less_than_pass() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(5));
    assert!(evaluate_condition(
        &Condition::LessThan {
            field: "count".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

#[test]
fn condition_less_than_fail() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(15));
    assert!(!evaluate_condition(
        &Condition::LessThan {
            field: "count".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

#[test]
fn condition_less_than_equal_is_false() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(10));
    assert!(!evaluate_condition(
        &Condition::LessThan {
            field: "count".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

#[test]
fn condition_less_than_non_numeric() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("name".to_string(), json!("alice"));
    assert!(!evaluate_condition(
        &Condition::LessThan {
            field: "name".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

#[test]
fn condition_less_than_missing_field() {
    let ctx = EvaluationContext::new();
    assert!(!evaluate_condition(
        &Condition::LessThan {
            field: "missing".to_string(),
            value: 10.0
        },
        &ctx
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — Between
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_between_inclusive_min() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!(18));
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_between_inclusive_max() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!(65));
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_between_middle() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!(30));
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_between_below_min() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!(17));
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_between_above_max() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("age".to_string(), json!(66));
    let cond = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_between_non_numeric() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("name".to_string(), json!("alice"));
    let cond = Condition::Between {
        field: "name".to_string(),
        min: 0.0,
        max: 100.0,
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_between_missing_field() {
    let ctx = EvaluationContext::new();
    let cond = Condition::Between {
        field: "missing".to_string(),
        min: 0.0,
        max: 100.0,
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — In
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_in_pass() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("role".to_string(), json!("admin"));
    let cond = Condition::In {
        field: "role".to_string(),
        values: vec![json!("admin"), json!("mod")],
    };
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_in_fail() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("role".to_string(), json!("guest"));
    let cond = Condition::In {
        field: "role".to_string(),
        values: vec![json!("admin"), json!("mod")],
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_in_empty_list() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("role".to_string(), json!("admin"));
    let cond = Condition::In {
        field: "role".to_string(),
        values: vec![],
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_in_missing_field() {
    let ctx = EvaluationContext::new();
    let cond = Condition::In {
        field: "missing".to_string(),
        values: vec![json!("a"), json!("b")],
    };
    assert!(!evaluate_condition(&cond, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — And
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_and_both_true() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(true));
    ctx.insert("b".to_string(), json!(true));
    let cond = Condition::And(vec![
        Condition::Equals {
            field: "a".to_string(),
            value: json!(true),
        },
        Condition::Equals {
            field: "b".to_string(),
            value: json!(true),
        },
    ]);
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_and_one_false() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(true));
    ctx.insert("b".to_string(), json!(false));
    let cond = Condition::And(vec![
        Condition::Equals {
            field: "a".to_string(),
            value: json!(true),
        },
        Condition::Equals {
            field: "b".to_string(),
            value: json!(true),
        },
    ]);
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_and_empty_is_true() {
    let ctx = EvaluationContext::new();
    let cond = Condition::And(vec![]);
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_and_short_circuits() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("first".to_string(), json!(false));
    // Second condition references missing field but should not be evaluated
    let cond = Condition::And(vec![
        Condition::Equals {
            field: "first".to_string(),
            value: json!(true),
        },
        Condition::Equals {
            field: "nonexistent".to_string(),
            value: json!("val"),
        },
    ]);
    assert!(!evaluate_condition(&cond, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — Or
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_or_one_true() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("x".to_string(), json!(false));
    ctx.insert("y".to_string(), json!(true));
    let cond = Condition::Or(vec![
        Condition::Equals {
            field: "x".to_string(),
            value: json!(true),
        },
        Condition::Equals {
            field: "y".to_string(),
            value: json!(true),
        },
    ]);
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_or_none_true() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("x".to_string(), json!(false));
    ctx.insert("y".to_string(), json!(false));
    let cond = Condition::Or(vec![
        Condition::Equals {
            field: "x".to_string(),
            value: json!(true),
        },
        Condition::Equals {
            field: "y".to_string(),
            value: json!(true),
        },
    ]);
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_or_empty_is_false() {
    let ctx = EvaluationContext::new();
    let cond = Condition::Or(vec![]);
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_or_short_circuits() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("first".to_string(), json!(true));
    let cond = Condition::Or(vec![
        Condition::Equals {
            field: "first".to_string(),
            value: json!(true),
        },
        Condition::Equals {
            field: "nonexistent".to_string(),
            value: json!("val"),
        },
    ]);
    assert!(evaluate_condition(&cond, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — Not
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_not_inverts_true() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    let cond = Condition::Not(Box::new(Condition::Equals {
        field: "status".to_string(),
        value: json!("inactive"),
    }));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_not_inverts_false() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    let cond = Condition::Not(Box::new(Condition::Equals {
        field: "status".to_string(),
        value: json!("active"),
    }));
    assert!(!evaluate_condition(&cond, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Condition evaluation — nested / complex
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn condition_nested_and_or() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    ctx.insert("count".to_string(), json!(15));
    ctx.insert("role".to_string(), json!("admin"));

    // (status == "active" AND count > 10) OR role == "admin"
    let cond = Condition::Or(vec![
        Condition::And(vec![
            Condition::Equals {
                field: "status".to_string(),
                value: json!("active"),
            },
            Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            },
        ]),
        Condition::Equals {
            field: "role".to_string(),
            value: json!("admin"),
        },
    ]);
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_nested_not_or() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    ctx.insert("count".to_string(), json!(15));

    // NOT (status == "inactive" OR count < 10)
    let cond = Condition::Not(Box::new(Condition::Or(vec![
        Condition::Equals {
            field: "status".to_string(),
            value: json!("inactive"),
        },
        Condition::LessThan {
            field: "count".to_string(),
            value: 10.0,
        },
    ])));
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn condition_deeply_nested() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("a".to_string(), json!(true));
    ctx.insert("b".to_string(), json!(true));
    ctx.insert("c".to_string(), json!(true));
    ctx.insert("d".to_string(), json!(false));

    // ((a AND b) OR (c AND d)) AND NOT d
    let cond = Condition::And(vec![
        Condition::Or(vec![
            Condition::And(vec![
                Condition::Equals {
                    field: "a".to_string(),
                    value: json!(true),
                },
                Condition::Equals {
                    field: "b".to_string(),
                    value: json!(true),
                },
            ]),
            Condition::And(vec![
                Condition::Equals {
                    field: "c".to_string(),
                    value: json!(true),
                },
                Condition::Equals {
                    field: "d".to_string(),
                    value: json!(true),
                },
            ]),
        ]),
        Condition::Not(Box::new(Condition::Equals {
            field: "d".to_string(),
            value: json!(true),
        })),
    ]);
    assert!(evaluate_condition(&cond, &ctx));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rule evaluation — evaluate_rule
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn rule_matched() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test".to_string(),
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        actions: vec![Action::Allow],
        priority: 10,
    };
    let result = evaluate_rule(&rule, &ctx);
    assert!(result.matched);
    assert_eq!(result.actions.len(), 1);
    assert_eq!(result.rule_id, "rule.1");
}

#[test]
fn rule_not_matched() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("inactive"));
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test".to_string(),
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        actions: vec![Action::Allow],
        priority: 10,
    };
    let result = evaluate_rule(&rule, &ctx);
    assert!(!result.matched);
    assert_eq!(result.actions.len(), 0);
}

#[test]
fn rule_no_conditions_always_matches() {
    let ctx = EvaluationContext::new();
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    };
    let result = evaluate_rule(&rule, &ctx);
    assert!(result.matched);
}

#[test]
fn rule_multiple_conditions_all_must_pass() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    ctx.insert("count".to_string(), json!(15));
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test".to_string(),
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
                message: "ok".to_string(),
            },
        ],
        priority: 10,
    };
    let result = evaluate_rule(&rule, &ctx);
    assert!(result.matched);
    assert_eq!(result.actions.len(), 2);
}

#[test]
fn rule_evaluation_determinism() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test".to_string(),
        conditions: vec![Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        }],
        actions: vec![Action::Allow],
        priority: 10,
    };
    let r1 = evaluate_rule(&rule, &ctx);
    let r2 = evaluate_rule(&rule, &ctx);
    assert_eq!(r1, r2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rule evaluation — evaluate_rules_with_priority
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn rules_priority_ordering() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(15));
    let rules = vec![
        Rule {
            id: "rule.1".to_string(),
            name: "Low".to_string(),
            conditions: vec![Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            }],
            actions: vec![Action::Allow],
            priority: 5,
        },
        Rule {
            id: "rule.2".to_string(),
            name: "High".to_string(),
            conditions: vec![Condition::GreaterThan {
                field: "count".to_string(),
                value: 10.0,
            }],
            actions: vec![Action::Notify {
                message: "high".to_string(),
            }],
            priority: 10,
        },
    ];
    let results = evaluate_rules_with_priority(&rules, &ctx);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].rule_id, "rule.2"); // higher priority first
    assert_eq!(results[1].rule_id, "rule.1");
}

#[test]
fn rules_priority_only_matched() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("count".to_string(), json!(5));
    let rules = vec![
        Rule {
            id: "rule.1".to_string(),
            name: "Matches".to_string(),
            conditions: vec![Condition::LessThan {
                field: "count".to_string(),
                value: 10.0,
            }],
            actions: vec![Action::Allow],
            priority: 5,
        },
        Rule {
            id: "rule.2".to_string(),
            name: "No match".to_string(),
            conditions: vec![Condition::GreaterThan {
                field: "count".to_string(),
                value: 100.0,
            }],
            actions: vec![Action::Deny],
            priority: 10,
        },
    ];
    let results = evaluate_rules_with_priority(&rules, &ctx);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule_id, "rule.1");
}

#[test]
fn rules_equal_priority_definition_order() {
    let mut ctx = EvaluationContext::new();
    ctx.insert("status".to_string(), json!("active"));
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
    let results = evaluate_rules_with_priority(&rules, &ctx);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].rule_id, "rule.1");
    assert_eq!(results[1].rule_id, "rule.2");
    assert_eq!(results[2].rule_id, "rule.3");
}

#[test]
fn rules_empty_list() {
    let ctx = EvaluationContext::new();
    let results = evaluate_rules_with_priority(&[], &ctx);
    assert!(results.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Action execution — execute_actions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn execute_actions_allow() {
    let result = RuleResult {
        matched: true,
        actions: vec![Action::Allow],
        rule_id: "rule.1".to_string(),
    };
    let exec = execute_actions(&result);
    assert_eq!(exec.len(), 1);
    assert_eq!(exec[0].action_type, "Allow");
    assert!(exec[0].success);
    assert_eq!(exec[0].action_index, 0);
}

#[test]
fn execute_actions_deny() {
    let result = RuleResult {
        matched: true,
        actions: vec![Action::Deny],
        rule_id: "rule.1".to_string(),
    };
    let exec = execute_actions(&result);
    assert_eq!(exec[0].action_type, "Deny");
    assert!(exec[0].success);
}

#[test]
fn execute_actions_require() {
    let result = RuleResult {
        matched: true,
        actions: vec![Action::Require {
            permission: "admin".to_string(),
        }],
        rule_id: "rule.1".to_string(),
    };
    let exec = execute_actions(&result);
    assert_eq!(exec[0].action_type, "Require");
    assert!(exec[0].message.contains("admin"));
}

#[test]
fn execute_actions_execute() {
    let result = RuleResult {
        matched: true,
        actions: vec![Action::Execute {
            task: "validate".to_string(),
        }],
        rule_id: "rule.1".to_string(),
    };
    let exec = execute_actions(&result);
    assert_eq!(exec[0].action_type, "Execute");
    assert!(exec[0].message.contains("validate"));
}

#[test]
fn execute_actions_notify() {
    let result = RuleResult {
        matched: true,
        actions: vec![Action::Notify {
            message: "logged".to_string(),
        }],
        rule_id: "rule.1".to_string(),
    };
    let exec = execute_actions(&result);
    assert_eq!(exec[0].action_type, "Notify");
    assert!(exec[0].message.contains("logged"));
}

#[test]
fn execute_actions_all_types() {
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
                message: "logged".to_string(),
            },
        ],
        rule_id: "rule.1".to_string(),
    };
    let exec = execute_actions(&result);
    assert_eq!(exec.len(), 5);
    assert_eq!(exec[0].action_type, "Allow");
    assert_eq!(exec[1].action_type, "Deny");
    assert_eq!(exec[2].action_type, "Require");
    assert_eq!(exec[3].action_type, "Execute");
    assert_eq!(exec[4].action_type, "Notify");
}

#[test]
fn execute_actions_preserves_order() {
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
    let exec = execute_actions(&result);
    assert_eq!(exec[0].action_index, 0);
    assert!(exec[0].message.contains("First"));
    assert_eq!(exec[1].action_index, 1);
    assert!(exec[1].message.contains("Second"));
    assert_eq!(exec[2].action_index, 2);
    assert!(exec[2].message.contains("Third"));
}

#[test]
fn execute_actions_empty() {
    let result = RuleResult {
        matched: false,
        actions: vec![],
        rule_id: "rule.1".to_string(),
    };
    assert_eq!(execute_actions(&result).len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SkillEngine
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn skill_engine_new_empty() {
    let engine = SkillEngine::new();
    assert!(engine.is_empty());
    assert_eq!(engine.len(), 0);
}

#[test]
fn skill_engine_default_empty() {
    let engine = SkillEngine::default();
    assert!(engine.is_empty());
}

#[test]
fn skill_engine_load_yaml() {
    let yaml = r#"
name: test-skill
triggers:
  - event: "file.changed"
actions:
  - tool: backup
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();
    assert_eq!(engine.len(), 1);
    assert!(engine.get("test-skill").is_some());
}

#[test]
fn skill_engine_load_yaml_invalid() {
    let mut engine = SkillEngine::new();
    let result = engine.load_yaml("{{{{invalid");
    assert!(result.is_err());
}

#[test]
fn skill_engine_load_yaml_many() {
    let yaml = r#"
- name: a
  triggers:
    - event: "x"
  actions:
    - tool: t1
- name: b
  triggers:
    - event: "y"
  actions:
    - tool: t2
"#;
    let mut engine = SkillEngine::new();
    let count = engine.load_yaml_many(yaml).unwrap();
    assert_eq!(count, 2);
    assert_eq!(engine.len(), 2);
}

#[test]
fn skill_engine_register_overwrites() {
    let yaml1 = r#"
name: skill1
triggers:
  - event: "a"
actions:
  - tool: t1
"#;
    let yaml2 = r#"
name: skill1
triggers:
  - event: "b"
actions:
  - tool: t2
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml1).unwrap();
    engine.load_yaml(yaml2).unwrap();
    assert_eq!(engine.len(), 1);
    let skill = engine.get("skill1").unwrap();
    assert_eq!(skill.actions[0].tool, "t2");
}

#[test]
fn skill_engine_unregister() {
    let yaml = r#"
name: removable
triggers:
  - event: "x"
actions:
  - tool: t
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();
    let removed = engine.unregister("removable");
    assert!(removed.is_some());
    assert!(engine.is_empty());
}

#[test]
fn skill_engine_unregister_nonexistent() {
    let mut engine = SkillEngine::new();
    assert!(engine.unregister("missing").is_none());
}

#[test]
fn skill_engine_skill_names() {
    let yaml = r#"
name: alpha
triggers:
  - event: "x"
actions:
  - tool: t
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();
    let names = engine.skill_names();
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"alpha"));
}

#[test]
fn skill_engine_process_event_match() {
    let yaml = r#"
name: auto-backup
triggers:
  - event: "file.changed"
    pattern: "/home/*/documents/*"
actions:
  - tool: backup
    args:
      source: "{{event.payload}}"
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: Some("/home/alice/documents/report.txt".to_string()),
    };
    let reports = engine.process_event(&event, &NoopExecutor);
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].skill_name, "auto-backup");
    assert_eq!(reports[0].action_results.len(), 1);
}

#[test]
fn skill_engine_process_event_no_match() {
    let yaml = r#"
name: auto-backup
triggers:
  - event: "file.changed"
actions:
  - tool: backup
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();
    let event = BusEvent {
        event_type: "file.deleted".to_string(),
        payload: None,
    };
    let reports = engine.process_event(&event, &NoopExecutor);
    assert!(reports.is_empty());
}

#[test]
fn skill_engine_process_event_pattern_mismatch() {
    let yaml = r#"
name: auto-backup
triggers:
  - event: "file.changed"
    pattern: "/home/*/documents/*"
actions:
  - tool: backup
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: Some("/var/log/syslog".to_string()),
    };
    let reports = engine.process_event(&event, &NoopExecutor);
    assert!(reports.is_empty());
}

#[test]
fn skill_engine_process_event_no_skills() {
    let engine = SkillEngine::new();
    let event = BusEvent {
        event_type: "anything".to_string(),
        payload: None,
    };
    let reports = engine.process_event(&event, &NoopExecutor);
    assert!(reports.is_empty());
}

#[test]
fn skill_engine_process_event_no_payload() {
    let yaml = r#"
name: pinger
triggers:
  - event: "ping"
actions:
  - tool: pong
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();
    let event = BusEvent {
        event_type: "ping".to_string(),
        payload: None,
    };
    let reports = engine.process_event(&event, &NoopExecutor);
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].skill_name, "pinger");
}

// ═══════════════════════════════════════════════════════════════════════════════
// BusEvent / TriggerEvaluator
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bus_event_construction() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: Some("/path/to/file".to_string()),
    };
    assert_eq!(event.event_type, "file.changed");
    assert_eq!(event.payload, Some("/path/to/file".to_string()));
}

#[test]
fn bus_event_no_payload() {
    let event = BusEvent {
        event_type: "timer.tick".to_string(),
        payload: None,
    };
    assert!(event.payload.is_none());
}

#[test]
fn trigger_evaluator_cron_never_matches() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: None,
    };
    let trigger = hudhudscript_rules::skill::SkillTrigger::Cron {
        cron: "0 * * * *".to_string(),
    };
    assert!(!TriggerEvaluator::matches(&event, &trigger));
}

#[test]
fn trigger_evaluator_manual_never_matches() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: None,
    };
    let trigger = hudhudscript_rules::skill::SkillTrigger::Manual { manual: true };
    assert!(!TriggerEvaluator::matches(&event, &trigger));
}

// ═══════════════════════════════════════════════════════════════════════════════
// SkillParser / SkillParseError
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn skill_parse_error_yaml_display() {
    let err = SkillParseError::YamlError("bad yaml".to_string());
    assert_eq!(err.to_string(), "YAML parse error: bad yaml");
}

#[test]
fn skill_parse_error_validation_display() {
    let err = SkillParseError::ValidationError("empty name".to_string());
    assert_eq!(err.to_string(), "Validation error: empty name");
}

#[test]
fn skill_parse_error_is_std_error() {
    let err = SkillParseError::YamlError("test".to_string());
    let _: &dyn std::error::Error = &err;
}

#[test]
fn skill_parse_error_eq() {
    assert_eq!(
        SkillParseError::YamlError("a".to_string()),
        SkillParseError::YamlError("a".to_string())
    );
    assert_ne!(
        SkillParseError::YamlError("a".to_string()),
        SkillParseError::ValidationError("a".to_string())
    );
}

#[test]
fn skill_parser_valid() {
    let yaml = r#"
name: ping
triggers:
  - event: "health.check"
actions:
  - tool: echo
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    assert_eq!(skill.name, "ping");
}

#[test]
fn skill_parser_empty_name_rejected() {
    let yaml = r#"
name: ""
triggers:
  - event: "x"
actions:
  - tool: y
"#;
    assert!(SkillParser::parse(yaml).is_err());
}

#[test]
fn skill_parser_no_triggers_rejected() {
    let yaml = r#"
name: bad
triggers: []
actions:
  - tool: y
"#;
    assert!(SkillParser::parse(yaml).is_err());
}

#[test]
fn skill_parser_no_actions_rejected() {
    let yaml = r#"
name: bad
triggers:
  - event: "x"
actions: []
"#;
    assert!(SkillParser::parse(yaml).is_err());
}

#[test]
fn skill_parser_empty_tool_rejected() {
    let yaml = r#"
name: bad
triggers:
  - event: "x"
actions:
  - tool: ""
"#;
    assert!(SkillParser::parse(yaml).is_err());
}

#[test]
fn skill_parser_invalid_yaml() {
    let err = SkillParser::parse("{{{{invalid yaml").unwrap_err();
    assert!(matches!(err, SkillParseError::YamlError(_)));
}

#[test]
fn skill_parser_parse_many() {
    let yaml = r#"
- name: a
  triggers:
    - event: "x"
  actions:
    - tool: t1
- name: b
  triggers:
    - event: "y"
  actions:
    - tool: t2
"#;
    let skills = SkillParser::parse_many(yaml).unwrap();
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0].name, "a");
    assert_eq!(skills[1].name, "b");
}

#[test]
fn skill_parser_empty_event_rejected() {
    let yaml = r#"
name: bad-trigger
triggers:
  - event: ""
actions:
  - tool: y
"#;
    let err = SkillParser::parse(yaml).unwrap_err();
    assert!(matches!(err, SkillParseError::ValidationError(_)));
}

#[test]
fn skill_parser_empty_cron_rejected() {
    let yaml = r#"
name: bad-cron
triggers:
  - cron: ""
actions:
  - tool: y
"#;
    let err = SkillParser::parse(yaml).unwrap_err();
    assert!(matches!(err, SkillParseError::ValidationError(_)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ActionChain / NoopExecutor
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn noop_executor_returns_success() {
    let action = hudhudscript_rules::skill::SkillAction {
        tool: "test_tool".to_string(),
        args: HashMap::new(),
        timeout: None,
    };
    let result = NoopExecutor.execute(&action, &HashMap::new());
    assert_eq!(result.status, ActionStatus::Success);
    assert_eq!(result.tool_name, "test_tool");
}

#[test]
fn noop_executor_echoes_args() {
    let mut args = HashMap::new();
    args.insert("key".to_string(), "val".to_string());
    let action = hudhudscript_rules::skill::SkillAction {
        tool: "tool".to_string(),
        args: args.clone(),
        timeout: None,
    };
    let result = NoopExecutor.execute(&action, &HashMap::new());
    assert_eq!(result.output, args);
}

#[test]
fn action_chain_runs_all() {
    let actions = vec![
        hudhudscript_rules::skill::SkillAction {
            tool: "t1".to_string(),
            args: HashMap::new(),
            timeout: None,
        },
        hudhudscript_rules::skill::SkillAction {
            tool: "t2".to_string(),
            args: HashMap::new(),
            timeout: None,
        },
    ];
    let chain = ActionChain::new(&NoopExecutor);
    let results = chain.run(&actions, &HashMap::new(), false);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].tool_name, "t1");
    assert_eq!(results[1].tool_name, "t2");
}

#[test]
fn action_chain_empty_actions() {
    let chain = ActionChain::new(&NoopExecutor);
    let results = chain.run(&[], &HashMap::new(), false);
    assert!(results.is_empty());
}

#[test]
fn action_chain_template_piping() {
    let actions = vec![
        hudhudscript_rules::skill::SkillAction {
            tool: "step1".to_string(),
            args: {
                let mut m = HashMap::new();
                m.insert("out".to_string(), "value1".to_string());
                m
            },
            timeout: None,
        },
        hudhudscript_rules::skill::SkillAction {
            tool: "step2".to_string(),
            args: {
                let mut m = HashMap::new();
                m.insert("input".to_string(), "{{out}}".to_string());
                m
            },
            timeout: None,
        },
    ];
    let chain = ActionChain::new(&NoopExecutor);
    let results = chain.run(&actions, &HashMap::new(), false);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].status, ActionStatus::Success);
    assert_eq!(results[1].status, ActionStatus::Success);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Skill / SkillTrigger / SkillAction types
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn skill_action_with_timeout() {
    let yaml = r#"
name: slow
triggers:
  - event: "build.started"
actions:
  - tool: build
    timeout: 300
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    assert_eq!(skill.actions[0].timeout, Some(300));
}

#[test]
fn skill_action_without_timeout() {
    let yaml = r#"
name: fast
triggers:
  - event: "ping"
actions:
  - tool: pong
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    assert_eq!(skill.actions[0].timeout, None);
}

#[test]
fn skill_with_description() {
    let yaml = r#"
name: described
description: A skill with a description
triggers:
  - event: "ping"
actions:
  - tool: pong
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    assert_eq!(skill.description, "A skill with a description");
}

#[test]
fn skill_without_description() {
    let yaml = r#"
name: minimal
triggers:
  - event: "ping"
actions:
  - tool: pong
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    assert!(skill.description.is_empty());
}

#[test]
fn skill_with_conditions() {
    let yaml = r#"
name: conditional
triggers:
  - event: "check"
conditions:
  - "file.size > 0"
  - "file.type == 'text'"
actions:
  - tool: process
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    assert_eq!(skill.conditions.len(), 2);
    assert_eq!(skill.conditions[0], "file.size > 0");
}

#[test]
fn skill_trigger_event_with_pattern() {
    let yaml = r#"
name: patterned
triggers:
  - event: "file.changed"
    pattern: "/home/*/documents/*"
actions:
  - tool: backup
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    if let hudhudscript_rules::skill::SkillTrigger::Event { event, pattern } = &skill.triggers[0] {
        assert_eq!(event, "file.changed");
        assert_eq!(pattern.as_deref(), Some("/home/*/documents/*"));
    } else {
        panic!("Expected Event trigger");
    }
}

#[test]
fn skill_trigger_cron() {
    let yaml = r#"
name: scheduled
triggers:
  - cron: "0 * * * *"
actions:
  - tool: cleanup
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    assert!(matches!(
        &skill.triggers[0],
        hudhudscript_rules::skill::SkillTrigger::Cron { cron } if cron == "0 * * * *"
    ));
}

#[test]
fn skill_trigger_manual() {
    let yaml = r#"
name: deploy
triggers:
  - manual: true
actions:
  - tool: deploy
"#;
    let skill = SkillParser::parse(yaml).unwrap();
    assert!(matches!(
        &skill.triggers[0],
        hudhudscript_rules::skill::SkillTrigger::Manual { manual: true }
    ));
}
