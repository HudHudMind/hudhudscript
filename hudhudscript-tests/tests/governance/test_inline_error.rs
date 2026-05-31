use hudhudscript_governance::error::{
    CacheIdCollisionError, CircularDependencyError, ConstitutionNotFoundError,
    FormatValidationError, GovernanceError, InvalidRoleError,
};

#[test]
fn test_constitution_not_found_error() {
    let error = ConstitutionNotFoundError::new("cons.123".to_string());
    assert_eq!(error.constitution_id, "cons.123");
    assert!(error.context.is_none());

    let display = format!("{}", error);
    assert!(display.contains("Constitution not found"));
    assert!(display.contains("cons.123"));
}

#[test]
fn test_constitution_not_found_with_context() {
    let error = ConstitutionNotFoundError::with_context(
        "cons.456".to_string(),
        "during enforcement".to_string(),
    );
    assert_eq!(error.constitution_id, "cons.456");
    assert_eq!(error.context, Some("during enforcement".to_string()));

    let display = format!("{}", error);
    assert!(display.contains("during enforcement"));
}

#[test]
fn test_cache_id_collision_error() {
    let error = CacheIdCollisionError::new("cons.123".to_string(), "Constitution".to_string());
    assert_eq!(error.id, "cons.123");
    assert_eq!(error.item_type, "Constitution");

    let display = format!("{}", error);
    assert!(display.contains("Cache ID collision"));
    assert!(display.contains("cons.123"));
    assert!(display.contains("Constitution"));
}

#[test]
fn test_circular_dependency_error() {
    let chain = vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "A".to_string(),
    ];
    let error = CircularDependencyError::new(chain.clone());
    assert_eq!(error.dependency_chain, chain);
    assert_eq!(error.cycle_string(), "A -> B -> C -> A");

    let display = format!("{}", error);
    assert!(display.contains("Circular dependency"));
    assert!(display.contains("A -> B -> C -> A"));
}

#[test]
fn test_invalid_role_error() {
    let error = InvalidRoleError::new("InvalidRole".to_string());
    assert_eq!(error.role_name, "InvalidRole");
    assert!(error.valid_roles.is_none());

    let display = format!("{}", error);
    assert!(display.contains("Invalid role"));
    assert!(display.contains("InvalidRole"));
}

#[test]
fn test_invalid_role_with_valid_roles() {
    let valid = vec!["Judge".to_string(), "Prosecutor".to_string()];
    let error = InvalidRoleError::with_valid_roles("BadRole".to_string(), valid.clone());
    assert_eq!(error.role_name, "BadRole");
    assert_eq!(error.valid_roles, Some(valid));

    let display = format!("{}", error);
    assert!(display.contains("Valid roles"));
    assert!(display.contains("Judge"));
    assert!(display.contains("Prosecutor"));
}

#[test]
fn test_format_validation_error() {
    let error = FormatValidationError::new("invalid-id".to_string(), "cons.\\d+".to_string());
    assert_eq!(error.value, "invalid-id");
    assert_eq!(error.expected_format, "cons.\\d+");

    let display = format!("{}", error);
    assert!(display.contains("Format validation error"));
    assert!(display.contains("invalid-id"));
    assert!(display.contains("cons.\\d+"));
}

#[test]
fn test_format_validation_with_field() {
    let error = FormatValidationError::with_field(
        "bad".to_string(),
        "cons.\\d+".to_string(),
        "constitution_id".to_string(),
    );
    assert_eq!(error.field_name, Some("constitution_id".to_string()));

    let display = format!("{}", error);
    assert!(display.contains("constitution_id"));
}

#[test]
fn test_governance_error_conversion() {
    let const_error = ConstitutionNotFoundError::new("cons.123".to_string());
    let gov_error: GovernanceError = const_error.clone().into();

    match gov_error {
        GovernanceError::ConstitutionNotFound(e) => assert_eq!(e, const_error),
        other => unreachable!("Expected ConstitutionNotFound, got: {:?}", other),
    }
}

#[test]
fn test_governance_error_display() {
    let error = GovernanceError::AgentNotFound("agent123".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Agent not found"));
    assert!(display.contains("agent123"));
}

#[test]
fn test_error_trait_implementation() {
    let error: Box<dyn std::error::Error> =
        Box::new(ConstitutionNotFoundError::new("cons.123".to_string()));
    assert!(error.to_string().contains("Constitution not found"));
}

#[test]
fn test_cache_id_collision_with_context() {
    let error = CacheIdCollisionError::with_context(
        "cons.1".to_string(),
        "Constitution".to_string(),
        "during merge".to_string(),
    );
    assert_eq!(error.context, Some("during merge".to_string()));
    let display = format!("{}", error);
    assert!(display.contains("during merge"));
}

#[test]
fn test_circular_dependency_with_context() {
    let chain = vec!["A".to_string(), "B".to_string(), "A".to_string()];
    let error = CircularDependencyError::with_context(chain, "in resolver".to_string());
    assert_eq!(error.context, Some("in resolver".to_string()));
    let display = format!("{}", error);
    assert!(display.contains("in resolver"));
    assert!(display.contains("A -> B -> A"));
}

#[test]
fn test_invalid_role_with_context() {
    let error =
        InvalidRoleError::with_context("BadRole".to_string(), "during assignment".to_string());
    assert_eq!(error.context, Some("during assignment".to_string()));
    assert!(error.valid_roles.is_none());
    let display = format!("{}", error);
    assert!(display.contains("during assignment"));
    assert!(display.contains("BadRole"));
}

#[test]
fn test_format_validation_with_context() {
    let error = FormatValidationError::with_context(
        "bad-id".to_string(),
        "cons.\\d+".to_string(),
        "in parser".to_string(),
    );
    assert_eq!(error.context, Some("in parser".to_string()));
    assert!(error.field_name.is_none());
    let display = format!("{}", error);
    assert!(display.contains("in parser"));
    assert!(display.contains("bad-id"));
}

#[test]
fn test_governance_error_display_all_variants() {
    let e1 = GovernanceError::ResourceNotFound("res.1".to_string());
    assert!(format!("{}", e1).contains("Resource not found"));
    assert!(format!("{}", e1).contains("res.1"));

    let e2 = GovernanceError::InvalidConfiguration("bad config".to_string());
    assert!(format!("{}", e2).contains("Invalid configuration"));
    assert!(format!("{}", e2).contains("bad config"));

    let e3 = GovernanceError::SerializationError("json error".to_string());
    assert!(format!("{}", e3).contains("Serialization error"));
    assert!(format!("{}", e3).contains("json error"));
}

#[test]
fn test_governance_error_conversions_all_variants() {
    // CacheIdCollision
    let cache_err = CacheIdCollisionError::new("id1".to_string(), "Law".to_string());
    let gov: GovernanceError = cache_err.into();
    assert!(matches!(gov, GovernanceError::CacheIdCollision(_)));

    // CircularDependency
    let circ_err = CircularDependencyError::new(vec!["A".to_string()]);
    let gov2: GovernanceError = circ_err.into();
    assert!(matches!(gov2, GovernanceError::CircularDependency(_)));

    // InvalidRole
    let role_err = InvalidRoleError::new("Bad".to_string());
    let gov3: GovernanceError = role_err.into();
    assert!(matches!(gov3, GovernanceError::InvalidRole(_)));

    // FormatValidation
    let fmt_err = FormatValidationError::new("val".to_string(), "pat".to_string());
    let gov4: GovernanceError = fmt_err.into();
    assert!(matches!(gov4, GovernanceError::FormatValidation(_)));
}

#[test]
fn test_governance_error_display_format_validation() {
    let err = GovernanceError::FormatValidation(FormatValidationError::with_field(
        "bad".to_string(),
        "cons.\\d+".to_string(),
        "id_field".to_string(),
    ));
    let display = format!("{}", err);
    assert!(display.contains("id_field"));
    assert!(display.contains("bad"));
}

#[test]
fn test_governance_error_display_circular_dependency() {
    let err = GovernanceError::CircularDependency(CircularDependencyError::new(vec![
        "X".to_string(),
        "Y".to_string(),
        "X".to_string(),
    ]));
    let display = format!("{}", err);
    assert!(display.contains("X -> Y -> X"));
}

#[test]
fn test_governance_error_display_invalid_role() {
    let err = GovernanceError::InvalidRole(InvalidRoleError::with_valid_roles(
        "Typo".to_string(),
        vec!["Judge".to_string(), "Member".to_string()],
    ));
    let display = format!("{}", err);
    assert!(display.contains("Typo"));
    assert!(display.contains("Judge"));
}

#[test]
fn test_governance_error_display_cache_collision() {
    let err = GovernanceError::CacheIdCollision(CacheIdCollisionError::new(
        "dup_id".to_string(),
        "Rule".to_string(),
    ));
    let display = format!("{}", err);
    assert!(display.contains("dup_id"));
    assert!(display.contains("Rule"));
}

#[test]
fn test_error_trait_for_all_types() {
    // Verify std::error::Error is implemented
    let e1: Box<dyn std::error::Error> =
        Box::new(CacheIdCollisionError::new("a".to_string(), "b".to_string()));
    assert!(!e1.to_string().is_empty());

    let e2: Box<dyn std::error::Error> =
        Box::new(CircularDependencyError::new(vec!["A".to_string()]));
    assert!(!e2.to_string().is_empty());

    let e3: Box<dyn std::error::Error> = Box::new(InvalidRoleError::new("X".to_string()));
    assert!(!e3.to_string().is_empty());

    let e4: Box<dyn std::error::Error> =
        Box::new(FormatValidationError::new("v".to_string(), "p".to_string()));
    assert!(!e4.to_string().is_empty());

    let e5: Box<dyn std::error::Error> = Box::new(GovernanceError::AgentNotFound("a".to_string()));
    assert!(!e5.to_string().is_empty());
}
