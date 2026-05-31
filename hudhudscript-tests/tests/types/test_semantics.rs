//! External tests for hudhudscript-types — semantics module (Ownership, OwnedType, default_ownership).

use hudhudscript_types::semantics::{default_ownership, OwnedType, Ownership};
use hudhudscript_types::Type;
use std::collections::HashMap;

#[test]
fn primitives_are_value_semantic() {
    assert_eq!(default_ownership(&Type::Number), Ownership::OwnedValue);
    assert_eq!(default_ownership(&Type::String), Ownership::OwnedValue);
    assert_eq!(default_ownership(&Type::Boolean), Ownership::OwnedValue);
}

#[test]
fn collections_are_reference_semantic_by_default() {
    let arr = Type::Array(Box::new(Type::Number));
    assert_eq!(default_ownership(&arr), Ownership::SharedRef);
}

#[test]
fn owned_type_display() {
    let ot = OwnedType::value(Type::Number);
    assert_eq!(format!("{}", ot), "Number");

    let ot = OwnedType::shared_ref(Type::String);
    assert_eq!(format!("{}", ot), "&String");

    let ot = OwnedType::shared_mut_ref(Type::Boolean);
    assert_eq!(format!("{}", ot), "&mut Boolean");
}

#[test]
fn owned_type_predicates() {
    assert!(OwnedType::value(Type::Number).is_value_semantic());
    assert!(!OwnedType::value(Type::Number).is_mutable());

    assert!(OwnedType::shared_ref(Type::String).is_reference_semantic());
    assert!(!OwnedType::shared_ref(Type::String).is_mutable());

    assert!(OwnedType::shared_mut_ref(Type::Boolean).is_mutable());
    assert!(OwnedType::shared_mut_ref(Type::Boolean).is_reference_semantic());
}

#[test]
fn ownership_display_value() {
    assert_eq!(format!("{}", Ownership::OwnedValue), "value");
}

#[test]
fn ownership_display_shared_ref() {
    assert_eq!(format!("{}", Ownership::SharedRef), "&T");
}

#[test]
fn ownership_display_shared_mut_ref() {
    assert_eq!(format!("{}", Ownership::SharedMutRef), "&mut T");
}

#[test]
fn null_is_value_semantic() {
    assert_eq!(default_ownership(&Type::Null), Ownership::OwnedValue);
}

#[test]
fn object_is_shared_ref() {
    let obj = Type::Object(HashMap::new());
    assert_eq!(default_ownership(&obj), Ownership::SharedRef);
}

#[test]
fn function_is_shared_ref() {
    let f = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Any),
    };
    assert_eq!(default_ownership(&f), Ownership::SharedRef);
}

#[test]
fn promise_is_shared_ref() {
    let p = Type::Promise(Box::new(Type::Number));
    assert_eq!(default_ownership(&p), Ownership::SharedRef);
}

#[test]
fn tool_is_shared_ref() {
    let t = Type::Tool {
        server: "s".to_string(),
        tool_name: "t".to_string(),
    };
    assert_eq!(default_ownership(&t), Ownership::SharedRef);
}

#[test]
fn resource_is_shared_ref() {
    let r = Type::Resource {
        server: "s".to_string(),
        uri: "u".to_string(),
    };
    assert_eq!(default_ownership(&r), Ownership::SharedRef);
}

#[test]
fn server_is_shared_ref() {
    assert_eq!(default_ownership(&Type::Server), Ownership::SharedRef);
}

#[test]
fn union_is_value_semantic() {
    let u = Type::Union(vec![Type::Number, Type::String]);
    assert_eq!(default_ownership(&u), Ownership::OwnedValue);
}

#[test]
fn generic_is_value_semantic() {
    assert_eq!(
        default_ownership(&Type::Generic("T".to_string())),
        Ownership::OwnedValue
    );
}

#[test]
fn any_is_value_semantic() {
    assert_eq!(default_ownership(&Type::Any), Ownership::OwnedValue);
}

#[test]
fn class_is_shared_ref() {
    let c = Type::Class {
        name: "Foo".to_string(),
        parent: None,
    };
    assert_eq!(default_ownership(&c), Ownership::SharedRef);
}

#[test]
fn instance_is_shared_ref() {
    assert_eq!(
        default_ownership(&Type::Instance("Foo".to_string())),
        Ownership::SharedRef
    );
}

#[test]
fn value_type_is_not_reference_semantic() {
    assert!(!OwnedType::value(Type::Number).is_reference_semantic());
}

#[test]
fn shared_ref_is_not_mutable_and_not_value() {
    let sr = OwnedType::shared_ref(Type::Number);
    assert!(!sr.is_mutable());
    assert!(!sr.is_value_semantic());
}

#[test]
fn shared_mut_ref_is_not_value_semantic() {
    assert!(!OwnedType::shared_mut_ref(Type::Number).is_value_semantic());
}
