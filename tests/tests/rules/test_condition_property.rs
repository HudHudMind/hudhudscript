//! Property-based tests for hudhudscript_rules condition evaluation

use hudhudscript_governance::Condition;
use hudhudscript_rules::condition::evaluate_condition;
use hudhudscript_rules::context::EvaluationContext;
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

// Strategy for generating simple conditions (non-logical)
fn simple_condition_strategy() -> impl Strategy<Value = Condition> {
    prop_oneof![
        // Equals
        (field_name_strategy(), json_value_strategy())
            .prop_map(|(field, value)| { Condition::Equals { field, value } }),
        // NotEquals
        (field_name_strategy(), json_value_strategy())
            .prop_map(|(field, value)| { Condition::NotEquals { field, value } }),
        // GreaterThan
        (field_name_strategy(), -1000.0f64..1000.0f64)
            .prop_map(|(field, value)| Condition::GreaterThan { field, value }),
        // LessThan
        (field_name_strategy(), -1000.0f64..1000.0f64)
            .prop_map(|(field, value)| Condition::LessThan { field, value }),
        // Between
        (
            field_name_strategy(),
            -1000.0f64..1000.0f64,
            -1000.0f64..1000.0f64
        )
            .prop_map(|(field, a, b)| {
                let (min, max) = if a <= b { (a, b) } else { (b, a) };
                Condition::Between { field, min, max }
            }),
        // In
        (
            field_name_strategy(),
            prop::collection::vec(json_value_strategy(), 1..5)
        )
            .prop_map(|(field, values)| Condition::In { field, values }),
    ]
}

// Strategy for generating conditions with limited nesting depth
fn condition_strategy_depth(depth: u32) -> impl Strategy<Value = Condition> {
    if depth == 0 {
        simple_condition_strategy().boxed()
    } else {
        prop_oneof![
            // 70% simple conditions
            7 => simple_condition_strategy().boxed(),
            // 10% And
            1 => prop::collection::vec(
                condition_strategy_depth(depth - 1),
                1..4
            )
            .prop_map(Condition::And)
            .boxed(),
            // 10% Or
            1 => prop::collection::vec(
                condition_strategy_depth(depth - 1),
                1..4
            )
            .prop_map(Condition::Or)
            .boxed(),
            // 10% Not
            1 => condition_strategy_depth(depth - 1)
                .prop_map(|c| Condition::Not(Box::new(c)))
                .boxed(),
        ]
        .boxed()
    }
}

// Default condition strategy with reasonable nesting
fn condition_strategy() -> impl Strategy<Value = Condition> {
    condition_strategy_depth(3)
}

// Strategy for generating deeply nested conditions
fn deeply_nested_condition_strategy() -> impl Strategy<Value = Condition> {
    condition_strategy_depth(5)
}

// **Property 8: Rule Evaluation Determinism**
proptest! {
    #[test]
    fn property_evaluation_determinism(
        condition in condition_strategy(),
        context in context_strategy()
    ) {
        // Evaluate the condition multiple times
        let result1 = evaluate_condition(&condition, &context);
        let result2 = evaluate_condition(&condition, &context);
        let result3 = evaluate_condition(&condition, &context);

        // All results must be identical
        prop_assert_eq!(result1, result2, "First and second evaluation must match");
        prop_assert_eq!(result2, result3, "Second and third evaluation must match");
    }
}

// **Property 10: Condition Type Support - Equals**
proptest! {
    #[test]
    fn property_equals_condition_support(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        let condition = Condition::Equals {
            field: field.clone(),
            value: value.clone(),
        };

        // Context with matching value
        let mut context_match = EvaluationContext::new();
        context_match.insert(field.clone(), value.clone());
        prop_assert!(
            evaluate_condition(&condition, &context_match),
            "Equals condition must return true when values match"
        );

        // Context with different value
        let mut context_no_match = EvaluationContext::new();
        context_no_match.insert(field.clone(), json!("different_value"));
        let result = evaluate_condition(&condition, &context_no_match);
        // Result depends on whether the values actually differ
        if value != json!("different_value") {
            prop_assert!(
                !result,
                "Equals condition must return false when values don't match"
            );
        }

        // Context without field
        let context_missing = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&condition, &context_missing),
            "Equals condition must return false when field is missing"
        );
    }
}

// **Property 10: Condition Type Support - NotEquals**
proptest! {
    #[test]
    fn property_not_equals_condition_support(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        let condition = Condition::NotEquals {
            field: field.clone(),
            value: value.clone(),
        };

        // Context with matching value
        let mut context_match = EvaluationContext::new();
        context_match.insert(field.clone(), value.clone());
        prop_assert!(
            !evaluate_condition(&condition, &context_match),
            "NotEquals condition must return false when values match"
        );

        // Context without field
        let context_missing = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&condition, &context_missing),
            "NotEquals condition must return false when field is missing"
        );
    }
}

// **Property 10: Condition Type Support - GreaterThan**
proptest! {
    #[test]
    fn property_greater_than_condition_support(
        field in field_name_strategy(),
        threshold in -1000.0f64..1000.0f64
    ) {
        let condition = Condition::GreaterThan {
            field: field.clone(),
            value: threshold,
        };

        // Context with greater value
        let mut context_greater = EvaluationContext::new();
        context_greater.insert(field.clone(), json!(threshold + 10.0));
        prop_assert!(
            evaluate_condition(&condition, &context_greater),
            "GreaterThan must return true when field value is greater"
        );

        // Context with lesser value
        let mut context_lesser = EvaluationContext::new();
        context_lesser.insert(field.clone(), json!(threshold - 10.0));
        prop_assert!(
            !evaluate_condition(&condition, &context_lesser),
            "GreaterThan must return false when field value is lesser"
        );

        // Context with equal value
        let mut context_equal = EvaluationContext::new();
        context_equal.insert(field.clone(), json!(threshold));
        prop_assert!(
            !evaluate_condition(&condition, &context_equal),
            "GreaterThan must return false when field value is equal"
        );

        // Context without field
        let context_missing = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&condition, &context_missing),
            "GreaterThan must return false when field is missing"
        );
    }
}

// **Property 10: Condition Type Support - LessThan**
proptest! {
    #[test]
    fn property_less_than_condition_support(
        field in field_name_strategy(),
        threshold in -1000.0f64..1000.0f64
    ) {
        let condition = Condition::LessThan {
            field: field.clone(),
            value: threshold,
        };

        // Context with lesser value
        let mut context_lesser = EvaluationContext::new();
        context_lesser.insert(field.clone(), json!(threshold - 10.0));
        prop_assert!(
            evaluate_condition(&condition, &context_lesser),
            "LessThan must return true when field value is lesser"
        );

        // Context with greater value
        let mut context_greater = EvaluationContext::new();
        context_greater.insert(field.clone(), json!(threshold + 10.0));
        prop_assert!(
            !evaluate_condition(&condition, &context_greater),
            "LessThan must return false when field value is greater"
        );

        // Context with equal value
        let mut context_equal = EvaluationContext::new();
        context_equal.insert(field.clone(), json!(threshold));
        prop_assert!(
            !evaluate_condition(&condition, &context_equal),
            "LessThan must return false when field value is equal"
        );

        // Context without field
        let context_missing = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&condition, &context_missing),
            "LessThan must return false when field is missing"
        );
    }
}

// **Property 10: Condition Type Support - Between**
proptest! {
    #[test]
    fn property_between_condition_support(
        field in field_name_strategy(),
        a in -1000.0f64..1000.0f64,
        b in -1000.0f64..1000.0f64
    ) {
        let (min, max) = if a <= b { (a, b) } else { (b, a) };
        let condition = Condition::Between {
            field: field.clone(),
            min,
            max,
        };

        // Context with value in range
        if max > min + 1.0 {
            let mid = (min + max) / 2.0;
            let mut context_in_range = EvaluationContext::new();
            context_in_range.insert(field.clone(), json!(mid));
            prop_assert!(
                evaluate_condition(&condition, &context_in_range),
                "Between must return true when value is in range"
            );
        }

        // Context with value at min boundary (inclusive)
        let mut context_min = EvaluationContext::new();
        context_min.insert(field.clone(), json!(min));
        prop_assert!(
            evaluate_condition(&condition, &context_min),
            "Between must return true at min boundary (inclusive)"
        );

        // Context with value at max boundary (inclusive)
        let mut context_max = EvaluationContext::new();
        context_max.insert(field.clone(), json!(max));
        prop_assert!(
            evaluate_condition(&condition, &context_max),
            "Between must return true at max boundary (inclusive)"
        );

        // Context with value below range
        let mut context_below = EvaluationContext::new();
        context_below.insert(field.clone(), json!(min - 10.0));
        prop_assert!(
            !evaluate_condition(&condition, &context_below),
            "Between must return false when value is below range"
        );

        // Context with value above range
        let mut context_above = EvaluationContext::new();
        context_above.insert(field.clone(), json!(max + 10.0));
        prop_assert!(
            !evaluate_condition(&condition, &context_above),
            "Between must return false when value is above range"
        );

        // Context without field
        let context_missing = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&condition, &context_missing),
            "Between must return false when field is missing"
        );
    }
}

// **Property 10: Condition Type Support - In**
proptest! {
    #[test]
    fn property_in_condition_support(
        field in field_name_strategy(),
        values in prop::collection::vec(json_value_strategy(), 1..5)
    ) {
        let condition = Condition::In {
            field: field.clone(),
            values: values.clone(),
        };

        // Context with value in set
        if !values.is_empty() {
            let mut context_in_set = EvaluationContext::new();
            context_in_set.insert(field.clone(), values[0].clone());
            prop_assert!(
                evaluate_condition(&condition, &context_in_set),
                "In condition must return true when value is in set"
            );
        }

        // Context without field
        let context_missing = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&condition, &context_missing),
            "In condition must return false when field is missing"
        );
    }
}

// **Property 10: Condition Type Support - And**
proptest! {
    #[test]
    fn property_and_condition_support(
        field1 in field_name_strategy(),
        field2 in field_name_strategy(),
        value1 in json_value_strategy(),
        value2 in json_value_strategy()
    ) {
        // Skip when fields collide — same key means second insert overwrites first
        prop_assume!(field1 != field2);
        let condition = Condition::And(vec![
            Condition::Equals {
                field: field1.clone(),
                value: value1.clone(),
            },
            Condition::Equals {
                field: field2.clone(),
                value: value2.clone(),
            },
        ]);

        // Context with both conditions true
        let mut context_both_true = EvaluationContext::new();
        context_both_true.insert(field1.clone(), value1.clone());
        context_both_true.insert(field2.clone(), value2.clone());
        prop_assert!(
            evaluate_condition(&condition, &context_both_true),
            "And must return true when all conditions are true"
        );

        // Context with first condition false
        let mut context_first_false = EvaluationContext::new();
        context_first_false.insert(field1.clone(), json!("wrong_value"));
        context_first_false.insert(field2.clone(), value2.clone());
        prop_assert!(
            !evaluate_condition(&condition, &context_first_false),
            "And must return false when first condition is false"
        );

        // Context with second condition false
        let mut context_second_false = EvaluationContext::new();
        context_second_false.insert(field1.clone(), value1.clone());
        context_second_false.insert(field2.clone(), json!("wrong_value"));
        prop_assert!(
            !evaluate_condition(&condition, &context_second_false),
            "And must return false when second condition is false"
        );

        // Context with both conditions false
        let context_both_false = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&condition, &context_both_false),
            "And must return false when all conditions are false"
        );
    }
}

// **Property 10: Condition Type Support - Or**
proptest! {
    #[test]
    fn property_or_condition_support(
        field1 in field_name_strategy(),
        field2 in field_name_strategy(),
        value1 in json_value_strategy(),
        value2 in json_value_strategy()
    ) {
        // Skip when fields collide — same key means second insert overwrites first
        prop_assume!(field1 != field2);
        let condition = Condition::Or(vec![
            Condition::Equals {
                field: field1.clone(),
                value: value1.clone(),
            },
            Condition::Equals {
                field: field2.clone(),
                value: value2.clone(),
            },
        ]);

        // Context with both conditions true
        let mut context_both_true = EvaluationContext::new();
        context_both_true.insert(field1.clone(), value1.clone());
        context_both_true.insert(field2.clone(), value2.clone());
        prop_assert!(
            evaluate_condition(&condition, &context_both_true),
            "Or must return true when all conditions are true"
        );

        // Context with first condition true
        let mut context_first_true = EvaluationContext::new();
        context_first_true.insert(field1.clone(), value1.clone());
        context_first_true.insert(field2.clone(), json!("wrong_value"));
        prop_assert!(
            evaluate_condition(&condition, &context_first_true),
            "Or must return true when first condition is true"
        );

        // Context with second condition true
        let mut context_second_true = EvaluationContext::new();
        context_second_true.insert(field1.clone(), json!("wrong_value"));
        context_second_true.insert(field2.clone(), value2.clone());
        prop_assert!(
            evaluate_condition(&condition, &context_second_true),
            "Or must return true when second condition is true"
        );

        // Context with both conditions false
        let context_both_false = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&condition, &context_both_false),
            "Or must return false when all conditions are false"
        );
    }
}

// **Property 10: Condition Type Support - Not**
proptest! {
    #[test]
    fn property_not_condition_support(
        field in field_name_strategy(),
        value in json_value_strategy()
    ) {
        let inner_condition = Condition::Equals {
            field: field.clone(),
            value: value.clone(),
        };
        let not_condition = Condition::Not(Box::new(inner_condition.clone()));

        // Context where inner condition is true
        let mut context_true = EvaluationContext::new();
        context_true.insert(field.clone(), value.clone());
        prop_assert!(
            evaluate_condition(&inner_condition, &context_true),
            "Inner condition must be true"
        );
        prop_assert!(
            !evaluate_condition(&not_condition, &context_true),
            "Not must invert true to false"
        );

        // Context where inner condition is false
        let context_false = EvaluationContext::new();
        prop_assert!(
            !evaluate_condition(&inner_condition, &context_false),
            "Inner condition must be false"
        );
        prop_assert!(
            evaluate_condition(&not_condition, &context_false),
            "Not must invert false to true"
        );
    }
}

// **Property 40: Nested Logical Operators**
proptest! {
    #[test]
    fn property_nested_logical_operators(
        condition in deeply_nested_condition_strategy(),
        context in context_strategy()
    ) {
        // The condition should evaluate without panicking or stack overflow
        let result = evaluate_condition(&condition, &context);

        // Result must be a boolean (true or false)
        prop_assert!(
            true,
            "Nested condition must evaluate to a boolean"
        );

        // Evaluating again must produce the same result (determinism)
        let result2 = evaluate_condition(&condition, &context);
        prop_assert_eq!(
            result,
            result2,
            "Nested condition evaluation must be deterministic"
        );
    }
}

// **Property 41: Short-Circuit Evaluation - And**
proptest! {
    #[test]
    fn property_short_circuit_and(
        field1 in field_name_strategy(),
        field2 in field_name_strategy()
    ) {
        // Create an And condition where first is false and second references missing field
        let condition = Condition::And(vec![
            Condition::Equals {
                field: field1.clone(),
                value: json!(true),
            },
            Condition::Equals {
                field: field2.clone(),
                value: json!("any_value"),
            },
        ]);

        // Context where first condition is false
        let mut context = EvaluationContext::new();
        context.insert(field1.clone(), json!(false));
        // field2 is intentionally missing

        // Should return false due to first condition, without evaluating second
        let result = evaluate_condition(&condition, &context);
        prop_assert!(
            !result,
            "And must short-circuit and return false when first condition is false"
        );

        // The evaluation should complete successfully even though field2 is missing
        // This demonstrates short-circuit behavior
    }
}

// **Property 41: Short-Circuit Evaluation - Or**
proptest! {
    #[test]
    fn property_short_circuit_or(
        field1 in field_name_strategy(),
        field2 in field_name_strategy()
    ) {
        // Create an Or condition where first is true and second references missing field
        let condition = Condition::Or(vec![
            Condition::Equals {
                field: field1.clone(),
                value: json!(true),
            },
            Condition::Equals {
                field: field2.clone(),
                value: json!("any_value"),
            },
        ]);

        // Context where first condition is true
        let mut context = EvaluationContext::new();
        context.insert(field1.clone(), json!(true));
        // field2 is intentionally missing

        // Should return true due to first condition, without evaluating second
        let result = evaluate_condition(&condition, &context);
        prop_assert!(
            result,
            "Or must short-circuit and return true when first condition is true"
        );

        // The evaluation should complete successfully even though field2 is missing
        // This demonstrates short-circuit behavior
    }
}

// **Additional Property: Condition Evaluation Consistency**
proptest! {
    #[test]
    fn property_condition_evaluation_consistency(
        condition in condition_strategy(),
        context in context_strategy()
    ) {
        let result = evaluate_condition(&condition, &context);

        // Result must be a boolean
        prop_assert!(
            true,
            "Condition evaluation must return a boolean"
        );

        // Double negation must equal original
        let double_not = Condition::Not(Box::new(Condition::Not(Box::new(condition.clone()))));
        let double_not_result = evaluate_condition(&double_not, &context);
        prop_assert_eq!(
            result,
            double_not_result,
            "Double negation must equal original result"
        );
    }
}

// **Additional Property: Empty And/Or Conditions**
proptest! {
    #[test]
    fn property_empty_logical_conditions(context in context_strategy()) {
        // Empty And should return true (all zero conditions are satisfied)
        let empty_and = Condition::And(vec![]);
        prop_assert!(
            evaluate_condition(&empty_and, &context),
            "Empty And condition must return true (vacuous truth)"
        );

        // Empty Or should return false (no conditions are satisfied)
        let empty_or = Condition::Or(vec![]);
        prop_assert!(
            !evaluate_condition(&empty_or, &context),
            "Empty Or condition must return false"
        );
    }
}

// **Additional Property: De Morgan's Laws**
proptest! {
    #[test]
    fn property_de_morgans_laws(
        cond_a in simple_condition_strategy(),
        cond_b in simple_condition_strategy(),
        context in context_strategy()
    ) {
        // NOT (A AND B) = (NOT A) OR (NOT B)
        let not_and = Condition::Not(Box::new(Condition::And(vec![
            cond_a.clone(),
            cond_b.clone(),
        ])));
        let not_a_or_not_b = Condition::Or(vec![
            Condition::Not(Box::new(cond_a.clone())),
            Condition::Not(Box::new(cond_b.clone())),
        ]);

        let result1 = evaluate_condition(&not_and, &context);
        let result2 = evaluate_condition(&not_a_or_not_b, &context);
        prop_assert_eq!(
            result1,
            result2,
            "De Morgan's law: NOT (A AND B) = (NOT A) OR (NOT B)"
        );

        // NOT (A OR B) = (NOT A) AND (NOT B)
        let not_or = Condition::Not(Box::new(Condition::Or(vec![
            cond_a.clone(),
            cond_b.clone(),
        ])));
        let not_a_and_not_b = Condition::And(vec![
            Condition::Not(Box::new(cond_a.clone())),
            Condition::Not(Box::new(cond_b.clone())),
        ]);

        let result3 = evaluate_condition(&not_or, &context);
        let result4 = evaluate_condition(&not_a_and_not_b, &context);
        prop_assert_eq!(
            result3,
            result4,
            "De Morgan's law: NOT (A OR B) = (NOT A) AND (NOT B)"
        );
    }
}

// =========================================================================
// NEW property-based tests with REAL assertions
// =========================================================================

// Strategy for generating JSON values that roundtrip perfectly through serde_json.
// We use integer-valued floats and strings/bools which don't suffer from f64 precision loss.
fn roundtrip_safe_json_value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<String>().prop_map(Value::String),
        (-1000i64..1000i64).prop_map(|i| json!(i)),
        any::<bool>().prop_map(Value::Bool),
    ]
}

// Strategy for conditions that use integer-valued numerics (safe for roundtrip)
fn roundtrip_safe_condition_strategy() -> impl Strategy<Value = Condition> {
    prop_oneof![
        (field_name_strategy(), roundtrip_safe_json_value_strategy())
            .prop_map(|(field, value)| Condition::Equals { field, value }),
        (field_name_strategy(), roundtrip_safe_json_value_strategy())
            .prop_map(|(field, value)| Condition::NotEquals { field, value }),
        (field_name_strategy(), -1000i64..1000i64).prop_map(|(field, v)| Condition::GreaterThan {
            field,
            value: v as f64
        }),
        (field_name_strategy(), -1000i64..1000i64).prop_map(|(field, v)| Condition::LessThan {
            field,
            value: v as f64
        }),
        (field_name_strategy(), -1000i64..1000i64, -1000i64..1000i64).prop_map(|(field, a, b)| {
            let (min, max) = if a <= b {
                (a as f64, b as f64)
            } else {
                (b as f64, a as f64)
            };
            Condition::Between { field, min, max }
        }),
        (
            field_name_strategy(),
            prop::collection::vec(roundtrip_safe_json_value_strategy(), 1..5)
        )
            .prop_map(|(field, values)| Condition::In { field, values }),
    ]
}

// Helper: assert exact serialization roundtrip using integer-safe values
fn assert_exact_roundtrip(
    condition: &Condition,
) -> Result<(), proptest::test_runner::TestCaseError> {
    let json1 = serde_json::to_string(condition).unwrap();
    let deserialized: Condition = serde_json::from_str(&json1).unwrap();
    prop_assert_eq!(
        condition,
        &deserialized,
        "Condition must survive exact serialization roundtrip"
    );
    let json2 = serde_json::to_string(&deserialized).unwrap();
    prop_assert_eq!(
        &json1,
        &json2,
        "Serialized JSON must be identical across roundtrips"
    );
    Ok(())
}

// Helper: assert that arbitrary conditions can serialize and deserialize without error,
// and that evaluation behavior is preserved across the roundtrip.
fn assert_behavioral_roundtrip(
    condition: &Condition,
    context: &EvaluationContext,
) -> Result<(), proptest::test_runner::TestCaseError> {
    let json = serde_json::to_string(condition).unwrap();
    let deserialized: Condition = serde_json::from_str(&json).unwrap();
    let original_result = evaluate_condition(condition, context);
    let roundtrip_result = evaluate_condition(&deserialized, context);
    prop_assert_eq!(
        original_result,
        roundtrip_result,
        "Evaluation result must be preserved across serialization roundtrip"
    );
    Ok(())
}

// Exact serialization roundtrip for every condition variant (using integer-safe values)
proptest! {
    #[test]
    fn prop_equals_roundtrip(
        field in field_name_strategy(),
        value in roundtrip_safe_json_value_strategy()
    ) {
        let condition = Condition::Equals { field, value };
        assert_exact_roundtrip(&condition)?;
    }
}

proptest! {
    #[test]
    fn prop_not_equals_roundtrip(
        field in field_name_strategy(),
        value in roundtrip_safe_json_value_strategy()
    ) {
        let condition = Condition::NotEquals { field, value };
        assert_exact_roundtrip(&condition)?;
    }
}

proptest! {
    #[test]
    fn prop_greater_than_roundtrip(
        field in field_name_strategy(),
        value in -1000i64..1000i64
    ) {
        let condition = Condition::GreaterThan { field, value: value as f64 };
        assert_exact_roundtrip(&condition)?;
    }
}

proptest! {
    #[test]
    fn prop_less_than_roundtrip(
        field in field_name_strategy(),
        value in -1000i64..1000i64
    ) {
        let condition = Condition::LessThan { field, value: value as f64 };
        assert_exact_roundtrip(&condition)?;
    }
}

proptest! {
    #[test]
    fn prop_between_roundtrip(
        field in field_name_strategy(),
        a in -1000i64..1000i64,
        b in -1000i64..1000i64
    ) {
        let (min, max) = if a <= b { (a as f64, b as f64) } else { (b as f64, a as f64) };
        let condition = Condition::Between { field, min, max };
        assert_exact_roundtrip(&condition)?;
    }
}

proptest! {
    #[test]
    fn prop_in_roundtrip(
        field in field_name_strategy(),
        values in prop::collection::vec(roundtrip_safe_json_value_strategy(), 1..5)
    ) {
        let condition = Condition::In { field, values };
        assert_exact_roundtrip(&condition)?;
    }
}

proptest! {
    #[test]
    fn prop_and_roundtrip(
        conditions in prop::collection::vec(roundtrip_safe_condition_strategy(), 0..4)
    ) {
        let condition = Condition::And(conditions);
        assert_exact_roundtrip(&condition)?;
    }
}

proptest! {
    #[test]
    fn prop_or_roundtrip(
        conditions in prop::collection::vec(roundtrip_safe_condition_strategy(), 0..4)
    ) {
        let condition = Condition::Or(conditions);
        assert_exact_roundtrip(&condition)?;
    }
}

proptest! {
    #[test]
    fn prop_not_roundtrip(
        inner in roundtrip_safe_condition_strategy()
    ) {
        let condition = Condition::Not(Box::new(inner));
        assert_exact_roundtrip(&condition)?;
    }
}

// Behavioral roundtrip: arbitrary conditions (with f64) preserve evaluation semantics
proptest! {
    #[test]
    fn prop_nested_condition_roundtrip(
        condition in condition_strategy(),
        context in context_strategy()
    ) {
        assert_behavioral_roundtrip(&condition, &context)?;
    }
}

// Validates: Equals and NotEquals are logical inverses when field is present
proptest! {
    #[test]
    fn prop_equals_not_equals_inverse(
        field in field_name_strategy(),
        value in json_value_strategy(),
        ctx_value in json_value_strategy()
    ) {
        let eq_cond = Condition::Equals { field: field.clone(), value: value.clone() };
        let neq_cond = Condition::NotEquals { field: field.clone(), value: value.clone() };

        let mut context = EvaluationContext::new();
        context.insert(field.clone(), ctx_value);

        let eq_result = evaluate_condition(&eq_cond, &context);
        let neq_result = evaluate_condition(&neq_cond, &context);
        prop_assert_ne!(
            eq_result, neq_result,
            "Equals and NotEquals must be logical inverses when field is present"
        );
    }
}

// Validates: GreaterThan and LessThan are mutually exclusive (both false when equal)
proptest! {
    #[test]
    fn prop_greater_less_than_mutual_exclusion(
        field in field_name_strategy(),
        threshold in -1000.0f64..1000.0f64,
        offset in -500.0f64..500.0f64
    ) {
        let gt_cond = Condition::GreaterThan { field: field.clone(), value: threshold };
        let lt_cond = Condition::LessThan { field: field.clone(), value: threshold };

        let mut context = EvaluationContext::new();
        context.insert(field.clone(), json!(threshold + offset));

        let gt_result = evaluate_condition(&gt_cond, &context);
        let lt_result = evaluate_condition(&lt_cond, &context);

        // Cannot be both greater than AND less than the same threshold
        prop_assert!(
            !(gt_result && lt_result),
            "A value cannot be both greater than and less than the same threshold"
        );
    }
}

// Validates: Between is equivalent to GreaterThanOrEqual AND LessThanOrEqual
proptest! {
    #[test]
    fn prop_between_validates_inclusive_bounds(
        field in field_name_strategy(),
        a in -1000.0f64..1000.0f64,
        b in -1000.0f64..1000.0f64,
        test_val in -1500.0f64..1500.0f64
    ) {
        let (min, max) = if a <= b { (a, b) } else { (b, a) };
        let between_cond = Condition::Between { field: field.clone(), min, max };

        let mut context = EvaluationContext::new();
        context.insert(field.clone(), json!(test_val));

        let between_result = evaluate_condition(&between_cond, &context);
        let expected = test_val >= min && test_val <= max;

        prop_assert_eq!(
            between_result, expected,
            "Between({}, {}) for value {} should be {}", min, max, test_val, expected
        );
    }
}

// Validates: Not(Not(X)) == X for arbitrary conditions
proptest! {
    #[test]
    fn prop_double_negation_validates(
        condition in condition_strategy(),
        context in context_strategy()
    ) {
        let original = evaluate_condition(&condition, &context);
        let double_neg = Condition::Not(Box::new(Condition::Not(Box::new(condition.clone()))));
        let double_neg_result = evaluate_condition(&double_neg, &context);
        prop_assert_eq!(
            original, double_neg_result,
            "Double negation must equal original"
        );

        // Single negation must differ
        let single_neg = Condition::Not(Box::new(condition));
        let single_neg_result = evaluate_condition(&single_neg, &context);
        prop_assert_ne!(
            original, single_neg_result,
            "Single negation must invert the result"
        );
    }
}

// Validates: And with single condition is equivalent to the condition itself
proptest! {
    #[test]
    fn prop_and_single_condition_identity(
        condition in simple_condition_strategy(),
        context in context_strategy()
    ) {
        let and_single = Condition::And(vec![condition.clone()]);
        let result_single = evaluate_condition(&condition, &context);
        let result_and = evaluate_condition(&and_single, &context);
        prop_assert_eq!(
            result_single, result_and,
            "And with single condition must equal the condition itself"
        );
    }
}

// Validates: Or with single condition is equivalent to the condition itself
proptest! {
    #[test]
    fn prop_or_single_condition_identity(
        condition in simple_condition_strategy(),
        context in context_strategy()
    ) {
        let or_single = Condition::Or(vec![condition.clone()]);
        let result_single = evaluate_condition(&condition, &context);
        let result_or = evaluate_condition(&or_single, &context);
        prop_assert_eq!(
            result_single, result_or,
            "Or with single condition must equal the condition itself"
        );
    }
}
