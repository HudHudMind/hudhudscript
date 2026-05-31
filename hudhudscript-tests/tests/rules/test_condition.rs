//! External tests for hudhudscript_rules::condition

use hudhudscript_governance::Condition;
use hudhudscript_rules::condition::evaluate_condition;
use hudhudscript_rules::context::EvaluationContext;
use serde_json::json;

#[test]
fn test_equals_condition() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    let condition = Condition::Equals {
        field: "status".to_string(),
        value: json!("active"),
    };

    assert!(evaluate_condition(&condition, &context));

    let condition_false = Condition::Equals {
        field: "status".to_string(),
        value: json!("inactive"),
    };

    assert!(!evaluate_condition(&condition_false, &context));
}

#[test]
fn test_not_equals_condition() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    let condition = Condition::NotEquals {
        field: "status".to_string(),
        value: json!("inactive"),
    };

    assert!(evaluate_condition(&condition, &context));

    let condition_false = Condition::NotEquals {
        field: "status".to_string(),
        value: json!("active"),
    };

    assert!(!evaluate_condition(&condition_false, &context));
}

#[test]
fn test_greater_than_condition() {
    let mut context = EvaluationContext::new();
    context.insert("count".to_string(), json!(15));

    let condition = Condition::GreaterThan {
        field: "count".to_string(),
        value: 10.0,
    };

    assert!(evaluate_condition(&condition, &context));

    let condition_false = Condition::GreaterThan {
        field: "count".to_string(),
        value: 20.0,
    };

    assert!(!evaluate_condition(&condition_false, &context));
}

#[test]
fn test_less_than_condition() {
    let mut context = EvaluationContext::new();
    context.insert("count".to_string(), json!(5));

    let condition = Condition::LessThan {
        field: "count".to_string(),
        value: 10.0,
    };

    assert!(evaluate_condition(&condition, &context));

    let condition_false = Condition::LessThan {
        field: "count".to_string(),
        value: 3.0,
    };

    assert!(!evaluate_condition(&condition_false, &context));
}

#[test]
fn test_between_condition() {
    let mut context = EvaluationContext::new();
    context.insert("age".to_string(), json!(25));

    let condition = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };

    assert!(evaluate_condition(&condition, &context));

    context.insert("age".to_string(), json!(18));
    assert!(evaluate_condition(&condition, &context));

    context.insert("age".to_string(), json!(65));
    assert!(evaluate_condition(&condition, &context));

    context.insert("age".to_string(), json!(17));
    assert!(!evaluate_condition(&condition, &context));

    context.insert("age".to_string(), json!(66));
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_in_condition() {
    let mut context = EvaluationContext::new();
    context.insert("role".to_string(), json!("admin"));

    let condition = Condition::In {
        field: "role".to_string(),
        values: vec![json!("admin"), json!("moderator"), json!("user")],
    };

    assert!(evaluate_condition(&condition, &context));

    context.insert("role".to_string(), json!("guest"));
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_and_condition() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));
    context.insert("count".to_string(), json!(15));

    let condition = Condition::And(vec![
        Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        },
        Condition::GreaterThan {
            field: "count".to_string(),
            value: 10.0,
        },
    ]);

    assert!(evaluate_condition(&condition, &context));

    let condition_false = Condition::And(vec![
        Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        },
        Condition::GreaterThan {
            field: "count".to_string(),
            value: 20.0,
        },
    ]);

    assert!(!evaluate_condition(&condition_false, &context));
}

#[test]
fn test_or_condition() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("inactive"));
    context.insert("count".to_string(), json!(15));

    let condition = Condition::Or(vec![
        Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        },
        Condition::GreaterThan {
            field: "count".to_string(),
            value: 10.0,
        },
    ]);

    assert!(evaluate_condition(&condition, &context));

    let condition_false = Condition::Or(vec![
        Condition::Equals {
            field: "status".to_string(),
            value: json!("active"),
        },
        Condition::GreaterThan {
            field: "count".to_string(),
            value: 20.0,
        },
    ]);

    assert!(!evaluate_condition(&condition_false, &context));
}

#[test]
fn test_not_condition() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));

    let condition = Condition::Not(Box::new(Condition::Equals {
        field: "status".to_string(),
        value: json!("inactive"),
    }));

    assert!(evaluate_condition(&condition, &context));

    let condition_false = Condition::Not(Box::new(Condition::Equals {
        field: "status".to_string(),
        value: json!("active"),
    }));

    assert!(!evaluate_condition(&condition_false, &context));
}

#[test]
fn test_nested_logical_operators() {
    let mut context = EvaluationContext::new();
    context.insert("status".to_string(), json!("active"));
    context.insert("count".to_string(), json!(15));
    context.insert("role".to_string(), json!("admin"));

    let condition = Condition::Or(vec![
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

    assert!(evaluate_condition(&condition, &context));

    let condition_nested_not = Condition::Not(Box::new(Condition::Or(vec![
        Condition::Equals {
            field: "status".to_string(),
            value: json!("inactive"),
        },
        Condition::LessThan {
            field: "count".to_string(),
            value: 10.0,
        },
    ])));

    assert!(evaluate_condition(&condition_nested_not, &context));
}

#[test]
fn test_missing_field() {
    let context = EvaluationContext::new();

    let condition = Condition::Equals {
        field: "nonexistent".to_string(),
        value: json!("value"),
    };

    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_short_circuit_and() {
    let mut context = EvaluationContext::new();
    context.insert("first".to_string(), json!(false));

    let condition = Condition::And(vec![
        Condition::Equals {
            field: "first".to_string(),
            value: json!(true),
        },
        Condition::Equals {
            field: "nonexistent".to_string(),
            value: json!("value"),
        },
    ]);

    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_short_circuit_or() {
    let mut context = EvaluationContext::new();
    context.insert("first".to_string(), json!(true));

    let condition = Condition::Or(vec![
        Condition::Equals {
            field: "first".to_string(),
            value: json!(true),
        },
        Condition::Equals {
            field: "nonexistent".to_string(),
            value: json!("value"),
        },
    ]);

    assert!(evaluate_condition(&condition, &context));
}

#[test]
fn test_greater_than_non_numeric_returns_false() {
    let mut context = EvaluationContext::new();
    context.insert("name".to_string(), json!("alice"));

    let condition = Condition::GreaterThan {
        field: "name".to_string(),
        value: 10.0,
    };
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_less_than_non_numeric_returns_false() {
    let mut context = EvaluationContext::new();
    context.insert("name".to_string(), json!("alice"));

    let condition = Condition::LessThan {
        field: "name".to_string(),
        value: 10.0,
    };
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_between_non_numeric_returns_false() {
    let mut context = EvaluationContext::new();
    context.insert("name".to_string(), json!("alice"));

    let condition = Condition::Between {
        field: "name".to_string(),
        min: 0.0,
        max: 100.0,
    };
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_greater_than_missing_field_returns_false() {
    let context = EvaluationContext::new();
    let condition = Condition::GreaterThan {
        field: "missing".to_string(),
        value: 10.0,
    };
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_less_than_missing_field_returns_false() {
    let context = EvaluationContext::new();
    let condition = Condition::LessThan {
        field: "missing".to_string(),
        value: 10.0,
    };
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_between_missing_field_returns_false() {
    let context = EvaluationContext::new();
    let condition = Condition::Between {
        field: "missing".to_string(),
        min: 0.0,
        max: 100.0,
    };
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_in_missing_field_returns_false() {
    let context = EvaluationContext::new();
    let condition = Condition::In {
        field: "missing".to_string(),
        values: vec![json!("a"), json!("b")],
    };
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_not_equals_missing_field_returns_false() {
    let context = EvaluationContext::new();
    let condition = Condition::NotEquals {
        field: "missing".to_string(),
        value: json!("val"),
    };
    assert!(!evaluate_condition(&condition, &context));
}

#[test]
fn test_deeply_nested_operators() {
    let mut context = EvaluationContext::new();
    context.insert("a".to_string(), json!(true));
    context.insert("b".to_string(), json!(true));
    context.insert("c".to_string(), json!(true));
    context.insert("d".to_string(), json!(false));

    let condition = Condition::And(vec![
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

    assert!(evaluate_condition(&condition, &context));
}
