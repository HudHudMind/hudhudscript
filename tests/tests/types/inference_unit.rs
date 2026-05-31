#[cfg(test)]
mod tests {
    use hudhudscript_types::inference::resolve_constraint_name;
    use hudhudscript_types::{Type, TypeInference};

    /// Issue #1009: Generic constraint enforcement — T: Number should reject String
    #[test]
    fn test_generic_constraint_rejects_incompatible_type() {
        let mut engine = TypeInference::new();
        engine.add_constraint("T".to_string(), Type::Number);

        let result = engine.unify(&Type::Generic("T".to_string()), &Type::String);
        assert!(result.is_err(), "T: Number should reject String");
        let err = result.unwrap_err();
        assert!(
            err.contains("Generic constraint violated"),
            "Error should mention constraint violation: {}",
            err
        );
    }

    /// Issue #1009: Generic constraint enforcement — T: Number should accept Number
    #[test]
    fn test_generic_constraint_accepts_compatible_type() {
        let mut engine = TypeInference::new();
        engine.add_constraint("T".to_string(), Type::Number);

        let result = engine.unify(&Type::Generic("T".to_string()), &Type::Number);
        assert!(result.is_ok(), "T: Number should accept Number");
    }

    /// Issue #1009: Generic constraint — Any always satisfies any constraint
    #[test]
    fn test_generic_constraint_any_passes() {
        let mut engine = TypeInference::new();
        engine.add_constraint("T".to_string(), Type::Number);

        let result = engine.unify(&Type::Generic("T".to_string()), &Type::Any);
        assert!(result.is_ok(), "Any should satisfy any constraint");
    }

    /// Issue #1009: Generic without constraint — should accept any type
    #[test]
    fn test_generic_without_constraint_accepts_any() {
        let mut engine = TypeInference::new();
        // No constraint added for T

        let result = engine.unify(&Type::Generic("T".to_string()), &Type::String);
        assert!(
            result.is_ok(),
            "Unconstrained generic should accept any type"
        );
    }

    /// Issue #1009: Generic constraint — T: String should reject Number
    #[test]
    fn test_generic_string_constraint_rejects_number() {
        let mut engine = TypeInference::new();
        engine.add_constraint("T".to_string(), Type::String);

        let result = engine.unify(&Type::Generic("T".to_string()), &Type::Number);
        assert!(result.is_err(), "T: String should reject Number");
    }

    /// Issue #1009: Generic constraint — T: Boolean should accept Boolean
    #[test]
    fn test_generic_boolean_constraint() {
        let mut engine = TypeInference::new();
        engine.add_constraint("T".to_string(), Type::Boolean);

        assert!(
            engine
                .unify(&Type::Generic("T".to_string()), &Type::Boolean)
                .is_ok(),
            "T: Boolean should accept Boolean"
        );
    }

    /// Issue #1009: resolve_constraint_name — built-in types
    #[test]
    fn test_resolve_constraint_name() {
        assert_eq!(resolve_constraint_name("Number"), Some(Type::Number));
        assert_eq!(resolve_constraint_name("number"), Some(Type::Number));
        assert_eq!(resolve_constraint_name("String"), Some(Type::String));
        assert_eq!(resolve_constraint_name("Boolean"), Some(Type::Boolean));
        assert_eq!(resolve_constraint_name("Bool"), Some(Type::Boolean));
        assert_eq!(resolve_constraint_name("Null"), Some(Type::Null));
        assert_eq!(resolve_constraint_name("Any"), Some(Type::Any));
        assert_eq!(
            resolve_constraint_name("UnknownClass"),
            None,
            "Unknown names return None"
        );
    }

    /// Issue #1009: remove_constraint should allow previously-constrained generic to accept any type
    #[test]
    fn test_remove_constraint() {
        let mut engine = TypeInference::new();
        engine.add_constraint("T".to_string(), Type::Number);
        engine.remove_constraint("T");

        let result = engine.unify(&Type::Generic("T".to_string()), &Type::String);
        assert!(
            result.is_ok(),
            "After removing constraint, T should accept any type"
        );
    }
}
