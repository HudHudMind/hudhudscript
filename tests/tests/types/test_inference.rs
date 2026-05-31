//! External tests for hudhudscript-types — inference module (TypeInference, TypeVar).

use hudhudscript_types::inference::{TypeInference, TypeVar};
use hudhudscript_types::Type;

#[test]
fn test_unify_same_types() {
    let mut inference = TypeInference::new();
    assert!(inference.unify(&Type::String, &Type::String).is_ok());
    assert!(inference.unify(&Type::Number, &Type::Number).is_ok());
}

#[test]
fn test_unify_different_types() {
    let mut inference = TypeInference::new();
    assert!(inference.unify(&Type::String, &Type::Number).is_err());
}

#[test]
fn test_unify_arrays() {
    let mut inference = TypeInference::new();
    let arr1 = Type::Array(Box::new(Type::Number));
    let arr2 = Type::Array(Box::new(Type::Number));
    assert!(inference.unify(&arr1, &arr2).is_ok());
}

#[test]
fn test_infer_general_type() {
    let mut inference = TypeInference::new();
    let types = vec![Type::Number, Type::Number, Type::Number];
    let result = inference.infer_general_type(&types);
    assert_eq!(result, Type::Number);
}

#[test]
fn test_infer_union_type() {
    let mut inference = TypeInference::new();
    let types = vec![Type::Number, Type::String];
    let result = inference.infer_general_type(&types);
    assert_eq!(result, Type::Union(vec![Type::Number, Type::String]));
}

#[test]
fn test_fresh_var_increments() {
    let mut inference = TypeInference::new();
    let v0 = inference.fresh_var();
    let v1 = inference.fresh_var();
    assert_eq!(v0, TypeVar(0));
    assert_eq!(v1, TypeVar(1));
}

#[test]
fn test_type_inference_default() {
    let inference = TypeInference::default();
    let _ = inference; // just ensure it compiles and doesn't panic
}

#[test]
fn test_unify_boolean() {
    let mut inference = TypeInference::new();
    assert!(inference.unify(&Type::Boolean, &Type::Boolean).is_ok());
}

#[test]
fn test_unify_null() {
    let mut inference = TypeInference::new();
    assert!(inference.unify(&Type::Null, &Type::Null).is_ok());
}

#[test]
fn test_unify_server() {
    let mut inference = TypeInference::new();
    assert!(inference.unify(&Type::Server, &Type::Server).is_ok());
}

#[test]
fn test_unify_any_with_anything() {
    let mut inference = TypeInference::new();
    assert!(inference.unify(&Type::Any, &Type::Number).is_ok());
    assert!(inference.unify(&Type::String, &Type::Any).is_ok());
}

#[test]
fn test_unify_arrays_incompatible() {
    let mut inference = TypeInference::new();
    let a1 = Type::Array(Box::new(Type::Number));
    let a2 = Type::Array(Box::new(Type::String));
    assert!(inference.unify(&a1, &a2).is_err());
}

#[test]
fn test_unify_functions_ok() {
    let mut inference = TypeInference::new();
    let f1 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    assert!(inference.unify(&f1, &f2).is_ok());
}

#[test]
fn test_unify_functions_arity_mismatch() {
    let mut inference = TypeInference::new();
    let f1 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![Type::Number, Type::Boolean],
        return_type: Box::new(Type::String),
    };
    let err = inference.unify(&f1, &f2).unwrap_err();
    assert_eq!(err, "Function arity mismatch: 1 vs 2");
}

#[test]
fn test_unify_functions_param_mismatch() {
    let mut inference = TypeInference::new();
    let f1 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![Type::String],
        return_type: Box::new(Type::String),
    };
    assert!(inference.unify(&f1, &f2).is_err());
}

#[test]
fn test_unify_functions_return_mismatch() {
    let mut inference = TypeInference::new();
    let f1 = Type::Function {
        params: vec![],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Number),
    };
    assert!(inference.unify(&f1, &f2).is_err());
}

#[test]
fn test_unify_promises_ok() {
    let mut inference = TypeInference::new();
    let p1 = Type::Promise(Box::new(Type::Number));
    let p2 = Type::Promise(Box::new(Type::Number));
    assert!(inference.unify(&p1, &p2).is_ok());
}

#[test]
fn test_unify_promises_incompatible() {
    let mut inference = TypeInference::new();
    let p1 = Type::Promise(Box::new(Type::Number));
    let p2 = Type::Promise(Box::new(Type::String));
    assert!(inference.unify(&p1, &p2).is_err());
}

#[test]
fn test_unify_objects_ok() {
    let mut inference = TypeInference::new();
    let o1 = Type::Object([("x".to_string(), Type::Number)].into_iter().collect());
    let o2 = Type::Object([("x".to_string(), Type::Number)].into_iter().collect());
    assert!(inference.unify(&o1, &o2).is_ok());
}

#[test]
fn test_unify_objects_missing_property() {
    let mut inference = TypeInference::new();
    let o1 = Type::Object(
        [
            ("x".to_string(), Type::Number),
            ("y".to_string(), Type::String),
        ]
        .into_iter()
        .collect(),
    );
    let o2 = Type::Object([("x".to_string(), Type::Number)].into_iter().collect());
    // o1 has "y" but o2 does not
    assert!(inference.unify(&o1, &o2).is_err());
}

#[test]
fn test_unify_objects_property_type_mismatch() {
    let mut inference = TypeInference::new();
    let o1 = Type::Object([("x".to_string(), Type::Number)].into_iter().collect());
    let o2 = Type::Object([("x".to_string(), Type::String)].into_iter().collect());
    assert!(inference.unify(&o1, &o2).is_err());
}

#[test]
fn test_unify_incompatible_catch_all() {
    let mut inference = TypeInference::new();
    assert!(inference.unify(&Type::Boolean, &Type::Number).is_err());
}

#[test]
fn test_apply_array() {
    let inference = TypeInference::new();
    let arr = Type::Array(Box::new(Type::Number));
    let result = inference.apply(&arr);
    assert_eq!(result, Type::Array(Box::new(Type::Number)));
}

#[test]
fn test_apply_function() {
    let inference = TypeInference::new();
    let f = Type::Function {
        params: vec![Type::String],
        return_type: Box::new(Type::Boolean),
    };
    let result = inference.apply(&f);
    assert_eq!(
        result,
        Type::Function {
            params: vec![Type::String],
            return_type: Box::new(Type::Boolean),
        }
    );
}

#[test]
fn test_apply_promise() {
    let inference = TypeInference::new();
    let p = Type::Promise(Box::new(Type::String));
    let result = inference.apply(&p);
    assert_eq!(result, Type::Promise(Box::new(Type::String)));
}

#[test]
fn test_apply_object() {
    let inference = TypeInference::new();
    let obj = Type::Object([("a".to_string(), Type::Number)].into_iter().collect());
    let result = inference.apply(&obj);
    assert_eq!(
        result,
        Type::Object([("a".to_string(), Type::Number)].into_iter().collect())
    );
}

#[test]
fn test_apply_primitive_passthrough() {
    let inference = TypeInference::new();
    assert_eq!(inference.apply(&Type::String), Type::String);
    assert_eq!(inference.apply(&Type::Number), Type::Number);
    assert_eq!(inference.apply(&Type::Boolean), Type::Boolean);
    assert_eq!(inference.apply(&Type::Null), Type::Null);
    assert_eq!(inference.apply(&Type::Any), Type::Any);
}

#[test]
fn test_infer_general_type_empty() {
    let mut inference = TypeInference::new();
    let result = inference.infer_general_type(&[]);
    assert_eq!(result, Type::Any);
}

#[test]
fn test_infer_general_type_single() {
    let mut inference = TypeInference::new();
    let result = inference.infer_general_type(&[Type::Boolean]);
    assert_eq!(result, Type::Boolean);
}

#[test]
fn test_apply_nested_array() {
    let inference = TypeInference::new();
    let nested = Type::Array(Box::new(Type::Array(Box::new(Type::String))));
    let result = inference.apply(&nested);
    assert_eq!(
        result,
        Type::Array(Box::new(Type::Array(Box::new(Type::String))))
    );
}

#[test]
fn test_apply_nested_function() {
    let inference = TypeInference::new();
    let f = Type::Function {
        params: vec![Type::Array(Box::new(Type::Number))],
        return_type: Box::new(Type::Promise(Box::new(Type::String))),
    };
    let result = inference.apply(&f);
    assert_eq!(
        result,
        Type::Function {
            params: vec![Type::Array(Box::new(Type::Number))],
            return_type: Box::new(Type::Promise(Box::new(Type::String))),
        }
    );
}

#[test]
fn test_unify_nested_arrays() {
    let mut inference = TypeInference::new();
    let a1 = Type::Array(Box::new(Type::Array(Box::new(Type::Number))));
    let a2 = Type::Array(Box::new(Type::Array(Box::new(Type::Number))));
    assert!(inference.unify(&a1, &a2).is_ok());
}

#[test]
fn test_unify_nested_arrays_mismatch() {
    let mut inference = TypeInference::new();
    let a1 = Type::Array(Box::new(Type::Array(Box::new(Type::Number))));
    let a2 = Type::Array(Box::new(Type::Array(Box::new(Type::String))));
    assert!(inference.unify(&a1, &a2).is_err());
}

#[test]
fn test_infer_general_type_all_same_string() {
    let mut inference = TypeInference::new();
    let result = inference.infer_general_type(&[Type::String, Type::String, Type::String]);
    assert_eq!(result, Type::String);
}

#[test]
fn test_infer_general_type_mixed_three() {
    let mut inference = TypeInference::new();
    let result = inference.infer_general_type(&[Type::Number, Type::String, Type::Boolean]);
    assert_eq!(
        result,
        Type::Union(vec![Type::Number, Type::String, Type::Boolean])
    );
}

#[test]
fn test_fresh_var_monotonic_ids() {
    let mut inference = TypeInference::new();
    let v0 = inference.fresh_var();
    let v1 = inference.fresh_var();
    let v2 = inference.fresh_var();
    assert_eq!(v0.0, 0);
    assert_eq!(v1.0, 1);
    assert_eq!(v2.0, 2);
}

#[test]
fn test_type_var_eq_and_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(TypeVar(0));
    set.insert(TypeVar(1));
    set.insert(TypeVar(0)); // duplicate
    assert_eq!(set.len(), 2);
}
