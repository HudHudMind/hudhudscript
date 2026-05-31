use hudhudscript_types::contracts::*;
use hudhudscript_types::Type;
use std::collections::HashMap;

// ============================================================================
// TypeConstraint::Numeric
// ============================================================================

#[test]
fn numeric_constraint_passes_number() {
    assert!(TypeConstraint::Numeric.check(&Type::Number).is_ok());
}

#[test]
fn numeric_constraint_passes_any() {
    assert!(TypeConstraint::Numeric.check(&Type::Any).is_ok());
}

#[test]
fn numeric_constraint_fails_string() {
    let err = TypeConstraint::Numeric.check(&Type::String).unwrap_err();
    assert_eq!(err, "expected Numeric type, found String");
}

#[test]
fn numeric_constraint_fails_boolean() {
    assert!(TypeConstraint::Numeric.check(&Type::Boolean).is_err());
}

#[test]
fn numeric_constraint_fails_null() {
    assert!(TypeConstraint::Numeric.check(&Type::Null).is_err());
}

// ============================================================================
// TypeConstraint::Textual
// ============================================================================

#[test]
fn textual_constraint_passes_string() {
    assert!(TypeConstraint::Textual.check(&Type::String).is_ok());
}

#[test]
fn textual_constraint_passes_any() {
    assert!(TypeConstraint::Textual.check(&Type::Any).is_ok());
}

#[test]
fn textual_constraint_fails_number() {
    assert!(TypeConstraint::Textual.check(&Type::Number).is_err());
}

#[test]
fn textual_constraint_fails_boolean() {
    assert!(TypeConstraint::Textual.check(&Type::Boolean).is_err());
}

// ============================================================================
// TypeConstraint::Iterable
// ============================================================================

#[test]
fn iterable_constraint_passes_array() {
    let arr = Type::Array(Box::new(Type::Number));
    assert!(TypeConstraint::Iterable.check(&arr).is_ok());
}

#[test]
fn iterable_constraint_passes_any() {
    assert!(TypeConstraint::Iterable.check(&Type::Any).is_ok());
}

#[test]
fn iterable_constraint_fails_string() {
    assert!(TypeConstraint::Iterable.check(&Type::String).is_err());
}

#[test]
fn iterable_constraint_fails_number() {
    assert!(TypeConstraint::Iterable.check(&Type::Number).is_err());
}

// ============================================================================
// TypeConstraint::Callable
// ============================================================================

#[test]
fn callable_constraint_passes_function() {
    let f = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Any),
    };
    assert!(TypeConstraint::Callable.check(&f).is_ok());
}

#[test]
fn callable_constraint_passes_any() {
    assert!(TypeConstraint::Callable.check(&Type::Any).is_ok());
}

#[test]
fn callable_constraint_fails_number() {
    assert!(TypeConstraint::Callable.check(&Type::Number).is_err());
}

#[test]
fn callable_constraint_fails_string() {
    assert!(TypeConstraint::Callable.check(&Type::String).is_err());
}

// ============================================================================
// TypeConstraint::NonNull
// ============================================================================

#[test]
fn non_null_passes_string() {
    assert!(TypeConstraint::NonNull.check(&Type::String).is_ok());
}

#[test]
fn non_null_passes_number() {
    assert!(TypeConstraint::NonNull.check(&Type::Number).is_ok());
}

#[test]
fn non_null_passes_boolean() {
    assert!(TypeConstraint::NonNull.check(&Type::Boolean).is_ok());
}

#[test]
fn non_null_fails_null() {
    let err = TypeConstraint::NonNull.check(&Type::Null).unwrap_err();
    assert_eq!(err, "value must be non-null");
}

// ============================================================================
// TypeConstraint::Exact
// ============================================================================

#[test]
fn exact_constraint_passes() {
    let c = TypeConstraint::Exact(Type::Number);
    assert!(c.check(&Type::Number).is_ok());
}

#[test]
fn exact_constraint_fails() {
    let c = TypeConstraint::Exact(Type::Number);
    let err = c.check(&Type::String).unwrap_err();
    assert_eq!(err, "expected Number, found String");
}

#[test]
fn exact_constraint_passes_with_any() {
    let c = TypeConstraint::Exact(Type::Number);
    assert!(c.check(&Type::Any).is_ok());
}

#[test]
fn exact_constraint_array() {
    let c = TypeConstraint::Exact(Type::Array(Box::new(Type::Number)));
    assert!(c.check(&Type::Array(Box::new(Type::Number))).is_ok());
    assert!(c.check(&Type::Array(Box::new(Type::String))).is_err());
}

// ============================================================================
// TypeConstraint::All
// ============================================================================

#[test]
fn all_constraint_both_pass() {
    let c = TypeConstraint::All(vec![TypeConstraint::NonNull, TypeConstraint::Numeric]);
    assert!(c.check(&Type::Number).is_ok());
}

#[test]
fn all_constraint_one_fails() {
    let c = TypeConstraint::All(vec![TypeConstraint::NonNull, TypeConstraint::Numeric]);
    assert!(c.check(&Type::Null).is_err());
}

#[test]
fn all_constraint_second_fails() {
    let c = TypeConstraint::All(vec![TypeConstraint::NonNull, TypeConstraint::Numeric]);
    assert!(c.check(&Type::String).is_err());
}

#[test]
fn all_constraint_empty_passes() {
    let c = TypeConstraint::All(vec![]);
    assert!(c.check(&Type::Null).is_ok());
}

// ============================================================================
// TypeConstraint::Any
// ============================================================================

#[test]
fn any_constraint_first_passes() {
    let c = TypeConstraint::Any(vec![TypeConstraint::Numeric, TypeConstraint::Textual]);
    assert!(c.check(&Type::Number).is_ok());
}

#[test]
fn any_constraint_second_passes() {
    let c = TypeConstraint::Any(vec![TypeConstraint::Numeric, TypeConstraint::Textual]);
    assert!(c.check(&Type::String).is_ok());
}

#[test]
fn any_constraint_none_pass() {
    let c = TypeConstraint::Any(vec![TypeConstraint::Numeric, TypeConstraint::Textual]);
    let err = c.check(&Type::Boolean).unwrap_err();
    assert_eq!(
        err,
        "type Boolean does not satisfy any of the required constraints"
    );
}

#[test]
fn any_constraint_empty_fails() {
    let c = TypeConstraint::Any(vec![]);
    assert!(c.check(&Type::Number).is_err());
}

// ============================================================================
// TypeConstraint — Display
// ============================================================================

#[test]
fn constraint_display_numeric() {
    assert_eq!(format!("{}", TypeConstraint::Numeric), "Numeric");
}

#[test]
fn constraint_display_textual() {
    assert_eq!(format!("{}", TypeConstraint::Textual), "Textual");
}

#[test]
fn constraint_display_iterable() {
    assert_eq!(format!("{}", TypeConstraint::Iterable), "Iterable");
}

#[test]
fn constraint_display_callable() {
    assert_eq!(format!("{}", TypeConstraint::Callable), "Callable");
}

#[test]
fn constraint_display_non_null() {
    assert_eq!(format!("{}", TypeConstraint::NonNull), "NonNull");
}

#[test]
fn constraint_display_exact() {
    assert_eq!(
        format!("{}", TypeConstraint::Exact(Type::Number)),
        "Exact(Number)"
    );
}

#[test]
fn constraint_display_all() {
    let c = TypeConstraint::All(vec![TypeConstraint::Numeric, TypeConstraint::NonNull]);
    assert_eq!(format!("{}", c), "All(Numeric, NonNull)");
}

#[test]
fn constraint_display_any() {
    let c = TypeConstraint::Any(vec![TypeConstraint::Numeric, TypeConstraint::Textual]);
    assert_eq!(format!("{}", c), "Any(Numeric, Textual)");
}

// ============================================================================
// Precondition
// ============================================================================

#[test]
fn precondition_check_passes() {
    let pre = Precondition::new("amount", TypeConstraint::Numeric);
    assert!(pre.check(&Type::Number).is_ok());
}

#[test]
fn precondition_check_fails() {
    let pre = Precondition::new("amount", TypeConstraint::Numeric)
        .with_description("amount must be numeric");
    let err = pre.check(&Type::String).unwrap_err();
    assert!(matches!(err, ContractViolation::PreconditionFailed { .. }));
}

#[test]
fn precondition_check_fails_has_param_name() {
    let pre = Precondition::new("amount", TypeConstraint::Numeric);
    let err = pre.check(&Type::String).unwrap_err();
    match err {
        ContractViolation::PreconditionFailed { param, .. } => {
            assert_eq!(param, "amount");
        }
        _ => panic!("expected PreconditionFailed"),
    }
}

#[test]
fn precondition_display_with_description() {
    let pre = Precondition::new("x", TypeConstraint::Numeric).with_description("must be positive");
    assert_eq!(format!("{}", pre), "requires x:Numeric (must be positive)");
}

#[test]
fn precondition_display_without_description() {
    let pre = Precondition::new("x", TypeConstraint::Textual);
    assert_eq!(format!("{}", pre), "requires x:Textual");
}

#[test]
fn precondition_fields_accessible() {
    let pre =
        Precondition::new("val", TypeConstraint::NonNull).with_description("must not be null");
    assert_eq!(pre.param, "val");
    assert_eq!(pre.description, Some("must not be null".to_string()));
}

// ============================================================================
// Postcondition
// ============================================================================

#[test]
fn postcondition_check_passes() {
    let post = Postcondition::new(TypeConstraint::NonNull);
    assert!(post.check(&Type::String).is_ok());
}

#[test]
fn postcondition_check_fails() {
    let post = Postcondition::new(TypeConstraint::NonNull).with_description("never returns null");
    let err = post.check(&Type::Null).unwrap_err();
    assert!(matches!(err, ContractViolation::PostconditionFailed { .. }));
}

#[test]
fn postcondition_check_fails_has_reason() {
    let post = Postcondition::new(TypeConstraint::Numeric);
    let err = post.check(&Type::String).unwrap_err();
    match err {
        ContractViolation::PostconditionFailed { reason, .. } => {
            assert!(reason.contains("Numeric"));
        }
        _ => panic!("expected PostconditionFailed"),
    }
}

#[test]
fn postcondition_display_with_description() {
    let post = Postcondition::new(TypeConstraint::NonNull).with_description("never returns null");
    assert_eq!(
        format!("{}", post),
        "ensures result:NonNull (never returns null)"
    );
}

#[test]
fn postcondition_display_without_description() {
    let post = Postcondition::new(TypeConstraint::Numeric);
    assert_eq!(format!("{}", post), "ensures result:Numeric");
}

// ============================================================================
// ContractSignature
// ============================================================================

#[test]
fn contract_signature_is_empty_when_new() {
    let sig = ContractSignature::new();
    assert!(sig.is_empty());
}

#[test]
fn contract_signature_not_empty_with_precondition() {
    let sig =
        ContractSignature::new().with_precondition(Precondition::new("x", TypeConstraint::Numeric));
    assert!(!sig.is_empty());
}

#[test]
fn contract_signature_not_empty_with_postcondition() {
    let sig =
        ContractSignature::new().with_postcondition(Postcondition::new(TypeConstraint::NonNull));
    assert!(!sig.is_empty());
}

#[test]
fn contract_signature_check_preconditions_pass() {
    let sig =
        ContractSignature::new().with_precondition(Precondition::new("x", TypeConstraint::Numeric));
    let args: HashMap<String, Type> = [("x".to_string(), Type::Number)].into_iter().collect();
    let violations = sig.check_preconditions(&args);
    assert_eq!(violations.len(), 0);
}

#[test]
fn contract_signature_check_preconditions_fail() {
    let sig =
        ContractSignature::new().with_precondition(Precondition::new("x", TypeConstraint::Numeric));
    let args: HashMap<String, Type> = [("x".to_string(), Type::String)].into_iter().collect();
    let violations = sig.check_preconditions(&args);
    assert_eq!(violations.len(), 1);
}

#[test]
fn contract_signature_check_preconditions_missing_param() {
    let sig =
        ContractSignature::new().with_precondition(Precondition::new("x", TypeConstraint::Numeric));
    let args: HashMap<String, Type> = HashMap::new();
    let violations = sig.check_preconditions(&args);
    // Missing param is not a contract violation (separate type error)
    assert_eq!(violations.len(), 0);
}

#[test]
fn contract_signature_multiple_preconditions() {
    let sig = ContractSignature::new()
        .with_precondition(Precondition::new("x", TypeConstraint::Numeric))
        .with_precondition(Precondition::new("y", TypeConstraint::Textual));
    let args: HashMap<String, Type> = [
        ("x".to_string(), Type::Number),
        ("y".to_string(), Type::String),
    ]
    .into_iter()
    .collect();
    assert_eq!(sig.check_preconditions(&args).len(), 0);
}

#[test]
fn contract_signature_multiple_preconditions_one_fails() {
    let sig = ContractSignature::new()
        .with_precondition(Precondition::new("x", TypeConstraint::Numeric))
        .with_precondition(Precondition::new("y", TypeConstraint::Textual));
    let args: HashMap<String, Type> = [
        ("x".to_string(), Type::Number),
        ("y".to_string(), Type::Number), // wrong
    ]
    .into_iter()
    .collect();
    assert_eq!(sig.check_preconditions(&args).len(), 1);
}

#[test]
fn contract_signature_check_postconditions_pass() {
    let sig =
        ContractSignature::new().with_postcondition(Postcondition::new(TypeConstraint::NonNull));
    let violations = sig.check_postconditions(&Type::Number);
    assert_eq!(violations.len(), 0);
}

#[test]
fn contract_signature_check_postconditions_fail() {
    let sig =
        ContractSignature::new().with_postcondition(Postcondition::new(TypeConstraint::NonNull));
    let violations = sig.check_postconditions(&Type::Null);
    assert_eq!(violations.len(), 1);
}

#[test]
fn contract_signature_multiple_postconditions() {
    let sig = ContractSignature::new()
        .with_postcondition(Postcondition::new(TypeConstraint::NonNull))
        .with_postcondition(Postcondition::new(TypeConstraint::Numeric));
    let violations = sig.check_postconditions(&Type::Number);
    assert_eq!(violations.len(), 0);
}

#[test]
fn contract_signature_display() {
    let sig = ContractSignature::new()
        .with_precondition(Precondition::new("x", TypeConstraint::Numeric))
        .with_postcondition(Postcondition::new(TypeConstraint::NonNull));
    let display = format!("{}", sig);
    assert_eq!(display, "  requires x:Numeric\n  ensures result:NonNull\n");
}

#[test]
fn contract_signature_display_empty() {
    let sig = ContractSignature::new();
    assert_eq!(format!("{}", sig), "");
}

// ============================================================================
// ContractViolation — Display
// ============================================================================

#[test]
fn contract_violation_precondition_display() {
    let v = ContractViolation::PreconditionFailed {
        param: "amount".to_string(),
        reason: "expected Numeric type, found String".to_string(),
        description: None,
    };
    let s = v.to_string();
    assert!(s.contains("amount"));
    assert!(s.contains("Numeric"));
}

#[test]
fn contract_violation_precondition_with_desc_display() {
    let v = ContractViolation::PreconditionFailed {
        param: "x".to_string(),
        reason: "bad type".to_string(),
        description: Some("must be a number".to_string()),
    };
    let s = v.to_string();
    assert!(s.contains("must be a number"));
}

#[test]
fn contract_violation_postcondition_display() {
    let v = ContractViolation::PostconditionFailed {
        reason: "value must be non-null".to_string(),
        description: None,
    };
    let s = v.to_string();
    assert!(s.contains("non-null"));
}

#[test]
fn contract_violation_equality() {
    let v1 = ContractViolation::PreconditionFailed {
        param: "x".to_string(),
        reason: "bad".to_string(),
        description: None,
    };
    let v2 = ContractViolation::PreconditionFailed {
        param: "x".to_string(),
        reason: "bad".to_string(),
        description: None,
    };
    assert_eq!(v1, v2);
}

// ============================================================================
// Additional coverage — moved from inline #[cfg(test)] blocks
// ============================================================================

#[test]
fn numeric_constraint_passes_for_number() {
    assert!(TypeConstraint::Numeric.check(&Type::Number).is_ok());
    assert!(TypeConstraint::Numeric.check(&Type::Any).is_ok());
}

#[test]
fn numeric_constraint_fails_for_string() {
    assert!(TypeConstraint::Numeric.check(&Type::String).is_err());
}

#[test]
fn textual_constraint() {
    assert!(TypeConstraint::Textual.check(&Type::String).is_ok());
    assert!(TypeConstraint::Textual.check(&Type::Number).is_err());
}

#[test]
fn iterable_constraint() {
    let arr = Type::Array(Box::new(Type::Number));
    assert!(TypeConstraint::Iterable.check(&arr).is_ok());
    assert!(TypeConstraint::Iterable.check(&Type::String).is_err());
}

#[test]
fn callable_constraint() {
    let f = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Any),
    };
    assert!(TypeConstraint::Callable.check(&f).is_ok());
    assert!(TypeConstraint::Callable.check(&Type::Number).is_err());
}

#[test]
fn non_null_constraint() {
    assert!(TypeConstraint::NonNull.check(&Type::String).is_ok());
    assert!(TypeConstraint::NonNull.check(&Type::Null).is_err());
}

#[test]
fn all_constraint() {
    let c = TypeConstraint::All(vec![TypeConstraint::NonNull, TypeConstraint::Numeric]);
    assert!(c.check(&Type::Number).is_ok());
    assert!(c.check(&Type::Null).is_err());
}

#[test]
fn any_constraint() {
    let c = TypeConstraint::Any(vec![TypeConstraint::Numeric, TypeConstraint::Textual]);
    assert!(c.check(&Type::Number).is_ok());
    assert!(c.check(&Type::String).is_ok());
    assert!(c.check(&Type::Boolean).is_err());
}

#[test]
fn precondition_check() {
    let pre = Precondition::new("amount", TypeConstraint::Numeric)
        .with_description("amount must be a number");
    assert!(pre.check(&Type::Number).is_ok());
    let err = pre.check(&Type::String).unwrap_err();
    assert!(matches!(err, ContractViolation::PreconditionFailed { .. }));
}

#[test]
fn postcondition_check() {
    let post = Postcondition::new(TypeConstraint::NonNull);
    assert!(post.check(&Type::String).is_ok());
    assert!(post.check(&Type::Null).is_err());
}

#[test]
fn contract_signature_full() {
    let sig = ContractSignature::new()
        .with_precondition(Precondition::new("x", TypeConstraint::Numeric))
        .with_precondition(Precondition::new("label", TypeConstraint::Textual))
        .with_postcondition(Postcondition::new(TypeConstraint::NonNull));

    // Passing args
    let args: HashMap<String, Type> = [
        ("x".to_string(), Type::Number),
        ("label".to_string(), Type::String),
    ]
    .into_iter()
    .collect();

    let violations = sig.check_preconditions(&args);
    assert!(violations.is_empty());

    let post_violations = sig.check_postconditions(&Type::Number);
    assert!(post_violations.is_empty());

    // Failing args
    let bad_args: HashMap<String, Type> = [
        ("x".to_string(), Type::String), // wrong type
        ("label".to_string(), Type::String),
    ]
    .into_iter()
    .collect();

    let violations = sig.check_preconditions(&bad_args);
    assert_eq!(violations.len(), 1);
}

#[test]
fn type_constraint_display_exact() {
    let c = TypeConstraint::Exact(Type::Number);
    assert_eq!(format!("{}", c), "Exact(Number)");
}

#[test]
fn type_constraint_display_numeric() {
    assert_eq!(format!("{}", TypeConstraint::Numeric), "Numeric");
}

#[test]
fn type_constraint_display_textual() {
    assert_eq!(format!("{}", TypeConstraint::Textual), "Textual");
}

#[test]
fn type_constraint_display_iterable() {
    assert_eq!(format!("{}", TypeConstraint::Iterable), "Iterable");
}

#[test]
fn type_constraint_display_callable() {
    assert_eq!(format!("{}", TypeConstraint::Callable), "Callable");
}

#[test]
fn type_constraint_display_nonnull() {
    assert_eq!(format!("{}", TypeConstraint::NonNull), "NonNull");
}

#[test]
fn type_constraint_display_all() {
    let c = TypeConstraint::All(vec![TypeConstraint::Numeric, TypeConstraint::NonNull]);
    assert_eq!(format!("{}", c), "All(Numeric, NonNull)");
}

#[test]
fn type_constraint_display_any() {
    let c = TypeConstraint::Any(vec![TypeConstraint::Textual, TypeConstraint::Callable]);
    assert_eq!(format!("{}", c), "Any(Textual, Callable)");
}

#[test]
fn exact_constraint_passes_compatible() {
    let c = TypeConstraint::Exact(Type::Number);
    assert!(c.check(&Type::Number).is_ok());
}

#[test]
fn exact_constraint_fails_incompatible() {
    let c = TypeConstraint::Exact(Type::Number);
    let err = c.check(&Type::String).unwrap_err();
    assert_eq!(err, "expected Number, found String");
}

#[test]
fn iterable_allows_any() {
    assert!(TypeConstraint::Iterable.check(&Type::Any).is_ok());
}

#[test]
fn callable_allows_any() {
    assert!(TypeConstraint::Callable.check(&Type::Any).is_ok());
}

#[test]
fn textual_allows_any() {
    assert!(TypeConstraint::Textual.check(&Type::Any).is_ok());
}

#[test]
fn non_null_error_message() {
    let err = TypeConstraint::NonNull.check(&Type::Null).unwrap_err();
    assert_eq!(err, "value must be non-null");
}

#[test]
fn all_constraint_fails_on_first_violated() {
    let c = TypeConstraint::All(vec![TypeConstraint::Numeric, TypeConstraint::Textual]);
    // String is not Numeric, so fails on first
    let err = c.check(&Type::String).unwrap_err();
    assert_eq!(err, "expected Numeric type, found String");
}

#[test]
fn any_constraint_error_message() {
    let c = TypeConstraint::Any(vec![TypeConstraint::Numeric, TypeConstraint::Textual]);
    let err = c.check(&Type::Boolean).unwrap_err();
    assert_eq!(
        err,
        "type Boolean does not satisfy any of the required constraints"
    );
}

#[test]
fn postcondition_check_returns_postcondition_failed() {
    let post = Postcondition::new(TypeConstraint::NonNull).with_description("result is non-null");
    let err = post.check(&Type::Null).unwrap_err();
    match err {
        ContractViolation::PostconditionFailed {
            reason,
            description,
        } => {
            assert_eq!(reason, "value must be non-null");
            assert_eq!(description.unwrap(), "result is non-null");
        }
        other => panic!("Expected PostconditionFailed, got: {:?}", other),
    }
}

#[test]
fn precondition_failed_display_with_description() {
    let v = ContractViolation::PreconditionFailed {
        param: "x".to_string(),
        reason: "expected Numeric type, found String".to_string(),
        description: Some("x must be a number".to_string()),
    };
    let msg = format!("{}", v);
    assert_eq!(
        msg,
        "precondition violated for parameter 'x': expected Numeric type, found String (x must be a number)"
    );
}

#[test]
fn precondition_failed_display_without_description() {
    let v = ContractViolation::PreconditionFailed {
        param: "y".to_string(),
        reason: "value must be non-null".to_string(),
        description: None,
    };
    let msg = format!("{}", v);
    assert_eq!(
        msg,
        "precondition violated for parameter 'y': value must be non-null"
    );
}

#[test]
fn postcondition_failed_display_with_description() {
    let v = ContractViolation::PostconditionFailed {
        reason: "value must be non-null".to_string(),
        description: Some("always returns a value".to_string()),
    };
    let msg = format!("{}", v);
    assert_eq!(
        msg,
        "postcondition violated: value must be non-null (always returns a value)"
    );
}

#[test]
fn postcondition_failed_display_without_description() {
    let v = ContractViolation::PostconditionFailed {
        reason: "expected Numeric type, found String".to_string(),
        description: None,
    };
    let msg = format!("{}", v);
    assert_eq!(
        msg,
        "postcondition violated: expected Numeric type, found String"
    );
}

#[test]
fn contract_signature_postcondition_violation() {
    let sig =
        ContractSignature::new().with_postcondition(Postcondition::new(TypeConstraint::NonNull));
    let violations = sig.check_postconditions(&Type::Null);
    assert_eq!(violations.len(), 1);
    assert!(matches!(
        &violations[0],
        ContractViolation::PostconditionFailed { .. }
    ));
}

#[test]
fn contract_signature_multiple_postconditions_both_fail() {
    let sig = ContractSignature::new()
        .with_postcondition(Postcondition::new(TypeConstraint::NonNull))
        .with_postcondition(Postcondition::new(TypeConstraint::Numeric));

    let violations = sig.check_postconditions(&Type::Null);
    assert_eq!(violations.len(), 2);
}

#[test]
fn precondition_check_without_description() {
    let pre = Precondition::new("x", TypeConstraint::Textual);
    let err = pre.check(&Type::Number).unwrap_err();
    match err {
        ContractViolation::PreconditionFailed {
            param, description, ..
        } => {
            assert_eq!(param, "x");
            assert!(description.is_none());
        }
        other => panic!("Expected PreconditionFailed, got: {:?}", other),
    }
}

#[test]
fn postcondition_check_without_description() {
    let post = Postcondition::new(TypeConstraint::Numeric);
    let err = post.check(&Type::String).unwrap_err();
    match err {
        ContractViolation::PostconditionFailed { description, .. } => {
            assert!(description.is_none());
        }
        other => panic!("Expected PostconditionFailed, got: {:?}", other),
    }
}

#[test]
fn nested_all_any_constraint() {
    let c = TypeConstraint::All(vec![
        TypeConstraint::NonNull,
        TypeConstraint::Any(vec![TypeConstraint::Numeric, TypeConstraint::Textual]),
    ]);
    assert!(c.check(&Type::Number).is_ok());
    assert!(c.check(&Type::String).is_ok());
    assert!(c.check(&Type::Null).is_err()); // fails NonNull
    assert!(c.check(&Type::Boolean).is_err()); // fails Any(Numeric, Textual)
}
