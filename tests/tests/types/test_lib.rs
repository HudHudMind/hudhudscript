//! Public API tests for hudhudscript-types — Type, TypeError, from_ast,
//! is_compatible_with, and Display.

use hudhudscript_ast::{Position, Span, TypeAnnotation as AstType};
use hudhudscript_types::Type;
use std::collections::HashMap;

// ── helpers ──────────────────────────────────────────────────────────────────

fn dummy_span() -> Span {
    let pos = Position::new(1, 1, 0);
    Span::new(pos, pos)
}

// ── Type::from_ast ────────────────────────────────────────────────────────────

#[test]
fn from_ast_string() {
    assert_eq!(Type::from_ast(&AstType::String), Type::String);
}

#[test]
fn from_ast_number() {
    assert_eq!(Type::from_ast(&AstType::Number), Type::Number);
}

#[test]
fn from_ast_boolean() {
    assert_eq!(Type::from_ast(&AstType::Boolean), Type::Boolean);
}

#[test]
fn from_ast_null() {
    assert_eq!(Type::from_ast(&AstType::Null), Type::Null);
}

#[test]
fn from_ast_any() {
    assert_eq!(Type::from_ast(&AstType::Any), Type::Any);
}

#[test]
fn from_ast_server() {
    assert_eq!(Type::from_ast(&AstType::Server), Type::Server);
}

#[test]
fn from_ast_tool_produces_empty_server_and_tool_name() {
    let t = Type::from_ast(&AstType::Tool);
    assert!(
        matches!(t, Type::Tool { ref server, ref tool_name } if server.is_empty() && tool_name.is_empty())
    );
}

#[test]
fn from_ast_resource_produces_empty_server_and_uri() {
    let t = Type::from_ast(&AstType::Resource);
    assert!(
        matches!(t, Type::Resource { ref server, ref uri } if server.is_empty() && uri.is_empty())
    );
}

#[test]
fn from_ast_generic() {
    let t = Type::from_ast(&AstType::Generic("T".to_string()));
    assert_eq!(t, Type::Generic("T".to_string()));
}

#[test]
fn from_ast_array_of_number() {
    let t = Type::from_ast(&AstType::Array(Box::new(AstType::Number)));
    assert_eq!(t, Type::Array(Box::new(Type::Number)));
}

#[test]
fn from_ast_array_of_string() {
    let t = Type::from_ast(&AstType::Array(Box::new(AstType::String)));
    assert_eq!(t, Type::Array(Box::new(Type::String)));
}

#[test]
fn from_ast_union_two_variants() {
    let t = Type::from_ast(&AstType::Union(vec![AstType::String, AstType::Number]));
    assert_eq!(t, Type::Union(vec![Type::String, Type::Number]));
}

#[test]
fn from_ast_union_three_variants() {
    let t = Type::from_ast(&AstType::Union(vec![
        AstType::String,
        AstType::Number,
        AstType::Boolean,
    ]));
    assert_eq!(
        t,
        Type::Union(vec![Type::String, Type::Number, Type::Boolean])
    );
}

#[test]
fn from_ast_parameterized_falls_back_to_base() {
    // Parameterized types fall back to the base type (generic instantiation not yet supported)
    let t = Type::from_ast(&AstType::Parameterized {
        base: Box::new(AstType::String),
        args: vec![AstType::Number],
    });
    assert_eq!(t, Type::String);
}

// ── is_compatible_with — primitive types ─────────────────────────────────────

#[test]
fn string_compatible_with_string() {
    assert!(Type::String.is_compatible_with(&Type::String));
}

#[test]
fn number_compatible_with_number() {
    assert!(Type::Number.is_compatible_with(&Type::Number));
}

#[test]
fn boolean_compatible_with_boolean() {
    assert!(Type::Boolean.is_compatible_with(&Type::Boolean));
}

#[test]
fn null_compatible_with_null() {
    assert!(Type::Null.is_compatible_with(&Type::Null));
}

#[test]
fn server_compatible_with_server() {
    assert!(Type::Server.is_compatible_with(&Type::Server));
}

#[test]
fn string_not_compatible_with_number() {
    assert!(!Type::String.is_compatible_with(&Type::Number));
}

#[test]
fn number_not_compatible_with_string() {
    assert!(!Type::Number.is_compatible_with(&Type::String));
}

#[test]
fn boolean_not_compatible_with_null() {
    assert!(!Type::Boolean.is_compatible_with(&Type::Null));
}

#[test]
fn null_not_compatible_with_string() {
    assert!(!Type::Null.is_compatible_with(&Type::String));
}

#[test]
fn server_not_compatible_with_string() {
    assert!(!Type::Server.is_compatible_with(&Type::String));
}

// ── is_compatible_with — Any (top type) ──────────────────────────────────────

#[test]
fn any_compatible_with_string() {
    assert!(Type::Any.is_compatible_with(&Type::String));
}

#[test]
fn any_compatible_with_number() {
    assert!(Type::Any.is_compatible_with(&Type::Number));
}

#[test]
fn any_compatible_with_null() {
    assert!(Type::Any.is_compatible_with(&Type::Null));
}

#[test]
fn any_compatible_with_boolean() {
    assert!(Type::Any.is_compatible_with(&Type::Boolean));
}

#[test]
fn any_compatible_with_server() {
    assert!(Type::Any.is_compatible_with(&Type::Server));
}

#[test]
fn string_compatible_with_any() {
    assert!(Type::String.is_compatible_with(&Type::Any));
}

#[test]
fn number_compatible_with_any() {
    assert!(Type::Number.is_compatible_with(&Type::Any));
}

#[test]
fn null_compatible_with_any() {
    assert!(Type::Null.is_compatible_with(&Type::Any));
}

#[test]
fn any_compatible_with_any() {
    assert!(Type::Any.is_compatible_with(&Type::Any));
}

// ── is_compatible_with — Array ────────────────────────────────────────────────

#[test]
fn array_number_compatible_with_array_number() {
    let a = Type::Array(Box::new(Type::Number));
    let b = Type::Array(Box::new(Type::Number));
    assert!(a.is_compatible_with(&b));
}

#[test]
fn array_string_compatible_with_array_string() {
    let a = Type::Array(Box::new(Type::String));
    let b = Type::Array(Box::new(Type::String));
    assert!(a.is_compatible_with(&b));
}

#[test]
fn array_string_not_compatible_with_array_number() {
    let a = Type::Array(Box::new(Type::String));
    let b = Type::Array(Box::new(Type::Number));
    assert!(!a.is_compatible_with(&b));
}

#[test]
fn array_any_compatible_with_array_string() {
    let a = Type::Array(Box::new(Type::Any));
    let b = Type::Array(Box::new(Type::String));
    assert!(a.is_compatible_with(&b));
}

#[test]
fn array_not_compatible_with_primitive_string() {
    let a = Type::Array(Box::new(Type::Number));
    assert!(!a.is_compatible_with(&Type::String));
}

// ── is_compatible_with — Function ────────────────────────────────────────────

#[test]
fn function_same_sig_compatible() {
    let f1 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    assert!(f1.is_compatible_with(&f2));
}

#[test]
fn function_different_arity_not_compatible() {
    let f1 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![Type::Number, Type::String],
        return_type: Box::new(Type::String),
    };
    assert!(!f1.is_compatible_with(&f2));
}

#[test]
fn function_no_params_compatible() {
    let f1 = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Null),
    };
    let f2 = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Null),
    };
    assert!(f1.is_compatible_with(&f2));
}

#[test]
fn function_not_compatible_with_string() {
    let f = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Null),
    };
    assert!(!f.is_compatible_with(&Type::String));
}

// ── is_compatible_with — Promise ─────────────────────────────────────────────

#[test]
fn promise_number_compatible_with_promise_number() {
    let a = Type::Promise(Box::new(Type::Number));
    let b = Type::Promise(Box::new(Type::Number));
    assert!(a.is_compatible_with(&b));
}

#[test]
fn promise_string_not_compatible_with_promise_number() {
    let a = Type::Promise(Box::new(Type::String));
    let b = Type::Promise(Box::new(Type::Number));
    assert!(!a.is_compatible_with(&b));
}

#[test]
fn promise_any_compatible_with_promise_string() {
    let a = Type::Promise(Box::new(Type::Any));
    let b = Type::Promise(Box::new(Type::String));
    assert!(a.is_compatible_with(&b));
}

// ── is_compatible_with — Union ────────────────────────────────────────────────

#[test]
fn union_string_number_compatible_with_string() {
    let u = Type::Union(vec![Type::String, Type::Number]);
    assert!(u.is_compatible_with(&Type::String));
}

#[test]
fn union_string_number_compatible_with_number() {
    let u = Type::Union(vec![Type::String, Type::Number]);
    assert!(u.is_compatible_with(&Type::Number));
}

#[test]
fn union_string_number_not_compatible_with_boolean() {
    let u = Type::Union(vec![Type::String, Type::Number]);
    assert!(!u.is_compatible_with(&Type::Boolean));
}

#[test]
fn string_compatible_with_union_containing_string() {
    let u = Type::Union(vec![Type::String, Type::Number]);
    assert!(Type::String.is_compatible_with(&u));
}

#[test]
fn boolean_not_compatible_with_union_string_number() {
    let u = Type::Union(vec![Type::String, Type::Number]);
    assert!(!Type::Boolean.is_compatible_with(&u));
}

// ── is_compatible_with — Generic ─────────────────────────────────────────────

#[test]
fn generic_same_name_compatible() {
    let a = Type::Generic("T".to_string());
    let b = Type::Generic("T".to_string());
    assert!(a.is_compatible_with(&b));
}

#[test]
fn generic_different_names_not_compatible() {
    let a = Type::Generic("T".to_string());
    let b = Type::Generic("U".to_string());
    assert!(!a.is_compatible_with(&b));
}

// ── is_compatible_with — Class and Instance ───────────────────────────────────

#[test]
fn class_same_name_compatible() {
    let a = Type::Class {
        name: "Foo".to_string(),
        parent: None,
    };
    let b = Type::Class {
        name: "Foo".to_string(),
        parent: None,
    };
    assert!(a.is_compatible_with(&b));
}

#[test]
fn class_different_name_not_compatible() {
    let a = Type::Class {
        name: "Foo".to_string(),
        parent: None,
    };
    let b = Type::Class {
        name: "Bar".to_string(),
        parent: None,
    };
    assert!(!a.is_compatible_with(&b));
}

#[test]
fn instance_same_name_compatible() {
    let a = Type::Instance("Foo".to_string());
    let b = Type::Instance("Foo".to_string());
    assert!(a.is_compatible_with(&b));
}

#[test]
fn instance_different_name_not_compatible() {
    let a = Type::Instance("Foo".to_string());
    let b = Type::Instance("Bar".to_string());
    assert!(!a.is_compatible_with(&b));
}

// ── Display ───────────────────────────────────────────────────────────────────

#[test]
fn display_string() {
    assert_eq!(format!("{}", Type::String), "String");
}

#[test]
fn display_number() {
    assert_eq!(format!("{}", Type::Number), "Number");
}

#[test]
fn display_boolean() {
    assert_eq!(format!("{}", Type::Boolean), "Boolean");
}

#[test]
fn display_null() {
    assert_eq!(format!("{}", Type::Null), "Null");
}

#[test]
fn display_any() {
    assert_eq!(format!("{}", Type::Any), "Any");
}

#[test]
fn display_server() {
    assert_eq!(format!("{}", Type::Server), "Server");
}

#[test]
fn display_array_number() {
    assert_eq!(
        format!("{}", Type::Array(Box::new(Type::Number))),
        "Array<Number>"
    );
}

#[test]
fn display_array_string() {
    assert_eq!(
        format!("{}", Type::Array(Box::new(Type::String))),
        "Array<String>"
    );
}

#[test]
fn display_union_string_number() {
    assert_eq!(
        format!("{}", Type::Union(vec![Type::String, Type::Number])),
        "String | Number"
    );
}

#[test]
fn display_function_no_params() {
    let f = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Null),
    };
    assert_eq!(format!("{}", f), "() => Null");
}

#[test]
fn display_function_with_params() {
    let f = Type::Function {
        params: vec![Type::Number, Type::String],
        return_type: Box::new(Type::Boolean),
    };
    assert_eq!(format!("{}", f), "(Number, String) => Boolean");
}

#[test]
fn display_promise_number() {
    let p = Type::Promise(Box::new(Type::Number));
    assert_eq!(format!("{}", p), "Promise<Number>");
}

#[test]
fn display_tool() {
    let t = Type::Tool {
        server: "s".to_string(),
        tool_name: "t".to_string(),
    };
    assert_eq!(format!("{}", t), "Tool(s.t)");
}

#[test]
fn display_resource() {
    let r = Type::Resource {
        server: "s".to_string(),
        uri: "myuri".to_string(),
    };
    assert_eq!(format!("{}", r), "Resource(s.myuri)");
}

#[test]
fn display_generic() {
    assert_eq!(format!("{}", Type::Generic("T".to_string())), "T");
}

#[test]
fn display_class() {
    let c = Type::Class {
        name: "MyClass".to_string(),
        parent: None,
    };
    assert_eq!(format!("{}", c), "class MyClass");
}

#[test]
fn display_instance() {
    let i = Type::Instance("MyClass".to_string());
    assert_eq!(format!("{}", i), "MyClass");
}

// ── TypeError variants ────────────────────────────────────────────────────────

#[test]
fn type_error_mismatch_display() {
    let err = hudhudscript_types::type_codes::mismatch(
        "Number".to_string(),
        "String".to_string(),
        dummy_span(),
    );
    let msg = format!("{}", err);
    assert!(msg.contains("Number"));
    assert!(msg.contains("String"));
}

#[test]
fn type_error_undefined_variable_display() {
    let err = hudhudscript_types::type_codes::undefined_variable("foo".to_string(), dummy_span());
    let msg = format!("{}", err);
    assert!(msg.contains("foo"));
}

#[test]
fn type_error_undefined_function_display() {
    let err = hudhudscript_types::type_codes::undefined_function("bar".to_string(), dummy_span());
    let msg = format!("{}", err);
    assert!(msg.contains("bar"));
}

#[test]
fn type_error_wrong_argument_count_display() {
    let err = hudhudscript_types::type_codes::wrong_argument_count(2, 3, dummy_span());
    let msg = format!("{}", err);
    assert!(msg.contains("2"));
    assert!(msg.contains("3"));
}

#[test]
fn type_error_invalid_operator_display() {
    let err = hudhudscript_types::type_codes::invalid_operator(
        "+".to_string(),
        "Boolean".to_string(),
        dummy_span(),
    );
    let msg = format!("{}", err);
    assert!(msg.contains("+"));
    assert!(msg.contains("Boolean"));
}

#[test]
fn type_error_invalid_index_display() {
    let err = hudhudscript_types::type_codes::invalid_index("String".to_string(), dummy_span());
    let msg = format!("{}", err);
    assert!(msg.contains("String"));
}

#[test]
fn type_error_invalid_member_display() {
    let err = hudhudscript_types::type_codes::invalid_member(
        "Number".to_string(),
        "length".to_string(),
        dummy_span(),
    );
    let msg = format!("{}", err);
    assert!(msg.contains("Number"));
    assert!(msg.contains("length"));
}

#[test]
fn type_error_duplicate_variable_display() {
    let err = hudhudscript_types::type_codes::duplicate_variable("x".to_string(), dummy_span());
    let msg = format!("{}", err);
    assert!(msg.contains("x"));
}

#[test]
fn type_error_invalid_await_display() {
    let err = hudhudscript_types::type_codes::invalid_await("String".to_string(), dummy_span());
    let msg = format!("{}", err);
    assert!(msg.contains("String"));
}

#[test]
fn type_error_type_mismatch_in_decl_display() {
    let err = hudhudscript_types::type_codes::type_mismatch_in_decl(
        "Number".to_string(),
        "String".to_string(),
        "myVar".to_string(),
        dummy_span(),
    );
    let msg = format!("{}", err);
    assert!(msg.contains("Number"));
    assert!(msg.contains("String"));
    assert!(msg.contains("myVar"));
}

// ── TypeError equality ────────────────────────────────────────────────────────

#[test]
fn type_error_equality_same_variant() {
    let e1 = hudhudscript_types::type_codes::undefined_variable("x".to_string(), dummy_span());
    let e2 = hudhudscript_types::type_codes::undefined_variable("x".to_string(), dummy_span());
    assert_eq!(e1, e2);
}

#[test]
fn type_error_inequality_different_names() {
    let e1 = hudhudscript_types::type_codes::undefined_variable("x".to_string(), dummy_span());
    let e2 = hudhudscript_types::type_codes::undefined_variable("y".to_string(), dummy_span());
    assert_ne!(e1, e2);
}

// ── Type PartialEq ────────────────────────────────────────────────────────────

#[test]
fn type_equality_primitives() {
    assert_eq!(Type::String, Type::String);
    assert_eq!(Type::Number, Type::Number);
    assert_eq!(Type::Boolean, Type::Boolean);
    assert_eq!(Type::Null, Type::Null);
    assert_eq!(Type::Any, Type::Any);
    assert_eq!(Type::Server, Type::Server);
}

#[test]
fn type_inequality_different_primitives() {
    assert_ne!(Type::String, Type::Number);
    assert_ne!(Type::Boolean, Type::Null);
    assert_ne!(Type::Any, Type::Server);
}

#[test]
fn type_clone_preserves_equality() {
    let t = Type::Array(Box::new(Type::Number));
    assert_eq!(t.clone(), t);
}

#[test]
fn type_object_variant_is_object() {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Type::String);
    fields.insert("age".to_string(), Type::Number);
    let obj = Type::Object(fields);
    assert!(matches!(obj, Type::Object(_)));
}

#[test]
fn type_class_with_parent() {
    let c = Type::Class {
        name: "Child".to_string(),
        parent: Some("Parent".to_string()),
    };
    if let Type::Class { name, parent } = c {
        assert_eq!(name, "Child");
        assert_eq!(parent, Some("Parent".to_string()));
    } else {
        panic!("Expected Class variant");
    }
}

#[test]
fn type_class_without_parent() {
    let c = Type::Class {
        name: "Root".to_string(),
        parent: None,
    };
    if let Type::Class { name, parent } = c {
        assert_eq!(name, "Root");
        assert!(parent.is_none());
    } else {
        panic!("Expected Class variant");
    }
}

#[test]
fn type_debug_format_contains_variant_name() {
    let s = format!("{:?}", Type::String);
    assert!(s.contains("String"));
    let n = format!("{:?}", Type::Number);
    assert!(n.contains("Number"));
}

#[test]
fn type_array_nested_display() {
    let t = Type::Array(Box::new(Type::Array(Box::new(Type::Number))));
    assert_eq!(format!("{}", t), "Array<Array<Number>>");
}

// ── Additional coverage moved from inline #[cfg(test)] blocks ─────────────────

#[test]
fn test_type_compatibility() {
    assert!(Type::String.is_compatible_with(&Type::String));
    assert!(Type::Number.is_compatible_with(&Type::Number));
    assert!(Type::Any.is_compatible_with(&Type::String));
    assert!(Type::String.is_compatible_with(&Type::Any));
    assert!(!Type::String.is_compatible_with(&Type::Number));
}

#[test]
fn test_array_compatibility() {
    let arr_num = Type::Array(Box::new(Type::Number));
    let arr_num2 = Type::Array(Box::new(Type::Number));
    let arr_str = Type::Array(Box::new(Type::String));

    assert!(arr_num.is_compatible_with(&arr_num2));
    assert!(!arr_num.is_compatible_with(&arr_str));
}

#[test]
fn test_union_compatibility() {
    let union = Type::Union(vec![Type::String, Type::Number]);
    assert!(union.is_compatible_with(&Type::String));
    assert!(union.is_compatible_with(&Type::Number));
    assert!(!union.is_compatible_with(&Type::Boolean));
}

#[test]
fn test_from_ast() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    let ast_type = AstType::String;
    let ty = Type::from_ast(&ast_type);
    assert_eq!(ty, Type::String);

    let ast_type = AstType::Array(Box::new(AstType::Number));
    let ty = Type::from_ast(&ast_type);
    assert_eq!(ty, Type::Array(Box::new(Type::Number)));
}

#[test]
fn test_type_display() {
    assert_eq!(format!("{}", Type::String), "String");
    assert_eq!(format!("{}", Type::Number), "Number");
    assert_eq!(format!("{}", Type::Boolean), "Boolean");
    assert_eq!(format!("{}", Type::Any), "Any");
    assert_eq!(
        format!("{}", Type::Array(Box::new(Type::Number))),
        "Array<Number>"
    );
    assert_eq!(
        format!("{}", Type::Union(vec![Type::String, Type::Number])),
        "String | Number"
    );
}

#[test]
fn test_type_display_null() {
    assert_eq!(format!("{}", Type::Null), "Null");
}

#[test]
fn test_type_display_server() {
    assert_eq!(format!("{}", Type::Server), "Server");
}

#[test]
fn test_type_display_function() {
    let f = Type::Function {
        params: vec![Type::Number, Type::String],
        return_type: Box::new(Type::Boolean),
    };
    assert_eq!(format!("{}", f), "(Number, String) => Boolean");
}

#[test]
fn test_type_display_function_no_params() {
    let f = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Null),
    };
    assert_eq!(format!("{}", f), "() => Null");
}

#[test]
fn test_type_display_promise() {
    let p = Type::Promise(Box::new(Type::String));
    assert_eq!(format!("{}", p), "Promise<String>");
}

#[test]
fn test_type_display_object() {
    use std::collections::HashMap;
    assert_eq!(format!("{}", Type::Object(HashMap::new())), "Object");
}

#[test]
fn test_type_display_tool() {
    let t = Type::Tool {
        server: "srv".to_string(),
        tool_name: "tl".to_string(),
    };
    assert_eq!(format!("{}", t), "Tool(srv.tl)");
}

#[test]
fn test_type_display_resource() {
    let r = Type::Resource {
        server: "s".to_string(),
        uri: "u".to_string(),
    };
    assert_eq!(format!("{}", r), "Resource(s.u)");
}

#[test]
fn test_type_display_generic() {
    assert_eq!(format!("{}", Type::Generic("T".to_string())), "T");
}

#[test]
fn test_type_display_class() {
    let c = Type::Class {
        name: "Foo".to_string(),
        parent: Some("Bar".to_string()),
    };
    assert_eq!(format!("{}", c), "class Foo");
}

#[test]
fn test_type_display_instance() {
    assert_eq!(
        format!("{}", Type::Instance("MyClass".to_string())),
        "MyClass"
    );
}

#[test]
fn test_from_ast_boolean() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    assert_eq!(Type::from_ast(&AstType::Boolean), Type::Boolean);
}

#[test]
fn test_from_ast_null() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    assert_eq!(Type::from_ast(&AstType::Null), Type::Null);
}

#[test]
fn test_from_ast_any() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    assert_eq!(Type::from_ast(&AstType::Any), Type::Any);
}

#[test]
fn test_from_ast_tool() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    assert_eq!(
        Type::from_ast(&AstType::Tool),
        Type::Tool {
            server: String::new(),
            tool_name: String::new()
        }
    );
}

#[test]
fn test_from_ast_resource() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    assert_eq!(
        Type::from_ast(&AstType::Resource),
        Type::Resource {
            server: String::new(),
            uri: String::new()
        }
    );
}

#[test]
fn test_from_ast_server() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    assert_eq!(Type::from_ast(&AstType::Server), Type::Server);
}

#[test]
fn test_from_ast_generic() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    assert_eq!(
        Type::from_ast(&AstType::Generic("T".to_string())),
        Type::Generic("T".to_string())
    );
}

#[test]
fn test_from_ast_union() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    let ast = AstType::Union(vec![AstType::String, AstType::Number]);
    assert_eq!(
        Type::from_ast(&ast),
        Type::Union(vec![Type::String, Type::Number])
    );
}

#[test]
fn test_from_ast_parameterized() {
    use hudhudscript_ast::TypeAnnotation as AstType;
    let ast = AstType::Parameterized {
        base: Box::new(AstType::Array(Box::new(AstType::Number))),
        args: vec![AstType::String],
    };
    // Parameterized falls back to base type
    assert_eq!(Type::from_ast(&ast), Type::Array(Box::new(Type::Number)));
}

#[test]
fn test_compat_boolean_boolean() {
    assert!(Type::Boolean.is_compatible_with(&Type::Boolean));
}

#[test]
fn test_compat_null_null() {
    assert!(Type::Null.is_compatible_with(&Type::Null));
}

#[test]
fn test_compat_server_server() {
    assert!(Type::Server.is_compatible_with(&Type::Server));
}

#[test]
fn test_compat_function_same_signature() {
    let f1 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    assert!(f1.is_compatible_with(&f2));
}

#[test]
fn test_compat_function_different_arity() {
    let f1 = Type::Function {
        params: vec![Type::Number],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![Type::Number, Type::Boolean],
        return_type: Box::new(Type::String),
    };
    assert!(!f1.is_compatible_with(&f2));
}

#[test]
fn test_compat_function_incompatible_return() {
    let f1 = Type::Function {
        params: vec![],
        return_type: Box::new(Type::String),
    };
    let f2 = Type::Function {
        params: vec![],
        return_type: Box::new(Type::Number),
    };
    assert!(!f1.is_compatible_with(&f2));
}

#[test]
fn test_compat_promise() {
    let p1 = Type::Promise(Box::new(Type::String));
    let p2 = Type::Promise(Box::new(Type::String));
    let p3 = Type::Promise(Box::new(Type::Number));
    assert!(p1.is_compatible_with(&p2));
    assert!(!p1.is_compatible_with(&p3));
}

#[test]
fn test_compat_other_union() {
    // Type::String compatible with Union(String, Number)
    let union = Type::Union(vec![Type::String, Type::Number]);
    assert!(Type::String.is_compatible_with(&union));
    assert!(!Type::Boolean.is_compatible_with(&union));
}

#[test]
fn test_compat_generic_same() {
    let g1 = Type::Generic("T".to_string());
    let g2 = Type::Generic("T".to_string());
    assert!(g1.is_compatible_with(&g2));
}

#[test]
fn test_compat_generic_different() {
    let g1 = Type::Generic("T".to_string());
    let g2 = Type::Generic("U".to_string());
    assert!(!g1.is_compatible_with(&g2));
}

#[test]
fn test_compat_class_same() {
    let c1 = Type::Class {
        name: "A".to_string(),
        parent: None,
    };
    let c2 = Type::Class {
        name: "A".to_string(),
        parent: Some("B".to_string()),
    };
    assert!(c1.is_compatible_with(&c2));
}

#[test]
fn test_compat_class_different() {
    let c1 = Type::Class {
        name: "A".to_string(),
        parent: None,
    };
    let c2 = Type::Class {
        name: "B".to_string(),
        parent: None,
    };
    assert!(!c1.is_compatible_with(&c2));
}

#[test]
fn test_compat_instance_same() {
    assert!(Type::Instance("A".to_string()).is_compatible_with(&Type::Instance("A".to_string())));
}

#[test]
fn test_compat_instance_different() {
    assert!(!Type::Instance("A".to_string()).is_compatible_with(&Type::Instance("B".to_string())));
}

#[test]
fn test_compat_cross_type_fail() {
    // String vs Boolean
    assert!(!Type::String.is_compatible_with(&Type::Boolean));
    // Number vs Null
    assert!(!Type::Number.is_compatible_with(&Type::Null));
    // Server vs Number
    assert!(!Type::Server.is_compatible_with(&Type::Number));
}

#[test]
fn test_type_error_mismatch_display() {
    let e = hudhudscript_types::type_codes::mismatch(
        "Number".to_string(),
        "String".to_string(),
        dummy_span(),
    );
    assert!(format!("{}", e).contains("Type mismatch: expected Number, found String"));
}

#[test]
fn test_type_error_undefined_variable_display() {
    let e = hudhudscript_types::type_codes::undefined_variable("x".to_string(), dummy_span());
    assert!(format!("{}", e).contains("Undefined variable: x"));
}

#[test]
fn test_type_error_undefined_function_display() {
    let e = hudhudscript_types::type_codes::undefined_function("foo".to_string(), dummy_span());
    assert!(format!("{}", e).contains("Undefined function: foo"));
}

#[test]
fn test_type_error_wrong_arg_count_display() {
    let e = hudhudscript_types::type_codes::wrong_argument_count(2, 3, dummy_span());
    assert!(format!("{}", e).contains("Wrong number of arguments: expected 2, found 3"));
}

#[test]
fn test_type_error_invalid_operator_display() {
    let e = hudhudscript_types::type_codes::invalid_operator(
        "+".to_string(),
        "Boolean".to_string(),
        dummy_span(),
    );
    assert!(format!("{}", e).contains("Cannot apply operator + to type Boolean"));
}

#[test]
fn test_type_error_invalid_index_display() {
    let e = hudhudscript_types::type_codes::invalid_index("String".to_string(), dummy_span());
    assert!(format!("{}", e).contains("Cannot index type String"));
}

#[test]
fn test_type_error_invalid_member_display() {
    let e = hudhudscript_types::type_codes::invalid_member(
        "Number".to_string(),
        "foo".to_string(),
        dummy_span(),
    );
    assert!(format!("{}", e).contains("Cannot access member foo on type Number"));
}

#[test]
fn test_type_error_duplicate_variable_display() {
    let e = hudhudscript_types::type_codes::duplicate_variable("x".to_string(), dummy_span());
    assert!(format!("{}", e).contains("Duplicate variable declaration: x"));
}

#[test]
fn test_type_error_invalid_await_display() {
    let e = hudhudscript_types::type_codes::invalid_await("Number".to_string(), dummy_span());
    assert!(format!("{}", e).contains("Cannot await non-promise type: Number"));
}

#[test]
fn test_type_error_type_mismatch_in_decl_display() {
    let e = hudhudscript_types::type_codes::type_mismatch_in_decl(
        "Number".to_string(),
        "String".to_string(),
        "x".to_string(),
        dummy_span(),
    );
    assert!(
        format!("{}", e).contains("Type error: expected Number, got String in declaration of 'x'")
    );
}
