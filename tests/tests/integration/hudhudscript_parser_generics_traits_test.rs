// Tests for generics (#658) and traits/interfaces (#659)

use hudhudscript_ast::Stmt;
use hudhudscript_parser::parse;

// ============================================================================
// Issue #658: Generics
// ============================================================================

#[test]
fn test_generic_function_single_param() {
    let source = r#"
        function identity<T>(x) {
            return x;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse generic function: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Function {
            name, type_params, ..
        } => {
            assert_eq!(name, "identity");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
            assert!(type_params[0].constraint.is_none());
        }
        other => panic!("Expected Function, got: {:?}", other),
    }
}

#[test]
fn test_generic_function_multiple_params() {
    let source = r#"
        function map<T, U>(arr, fn) {
            return arr;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse multi-generic function: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Function { type_params, .. } => {
            assert_eq!(type_params.len(), 2);
            assert_eq!(type_params[0].name, "T");
            assert_eq!(type_params[1].name, "U");
        }
        other => panic!("Expected Function, got: {:?}", other),
    }
}

#[test]
fn test_generic_function_with_constraint() {
    let source = r#"
        function sort<T: Comparable>(arr) {
            return arr;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse constrained generic: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Function { type_params, .. } => {
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
            assert_eq!(type_params[0].constraint, Some("Comparable".to_string()));
        }
        other => panic!("Expected Function, got: {:?}", other),
    }
}

#[test]
fn test_async_generic_function() {
    let source = r#"
        async function fetch<T>(url) {
            return null;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse async generic function: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Function {
            name,
            type_params,
            is_async,
            ..
        } => {
            assert_eq!(name, "fetch");
            assert!(*is_async);
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
        }
        other => panic!("Expected Function, got: {:?}", other),
    }
}

#[test]
fn test_generic_class() {
    let source = r#"
        class Stack<T> {
            constructor() {
                this.items = [];
            }
            function push(item) {
                this.items = this.items;
            }
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse generic class: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "Stack");
            assert_eq!(class_decl.type_params.len(), 1);
            assert_eq!(class_decl.type_params[0].name, "T");
        }
        other => panic!("Expected Class, got: {:?}", other),
    }
}

#[test]
fn test_generic_class_with_constraint() {
    let source = r#"
        class SortedList<T: Comparable> {
            constructor() {
                this.items = [];
            }
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse constrained generic class: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "SortedList");
            assert_eq!(class_decl.type_params.len(), 1);
            assert_eq!(class_decl.type_params[0].name, "T");
            assert_eq!(
                class_decl.type_params[0].constraint,
                Some("Comparable".to_string())
            );
        }
        other => panic!("Expected Class, got: {:?}", other),
    }
}

#[test]
fn test_generic_class_multiple_params() {
    let source = r#"
        class HashMap<K, V> {
            constructor() {
                this.entries = [];
            }
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse multi-generic class: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "HashMap");
            assert_eq!(class_decl.type_params.len(), 2);
            assert_eq!(class_decl.type_params[0].name, "K");
            assert_eq!(class_decl.type_params[1].name, "V");
        }
        other => panic!("Expected Class, got: {:?}", other),
    }
}

#[test]
fn test_function_without_generics_still_works() {
    let source = r#"
        function add(a, b) {
            return a + b;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse non-generic function: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Function { type_params, .. } => {
            assert!(type_params.is_empty());
        }
        other => panic!("Expected Function, got: {:?}", other),
    }
}

// ============================================================================
// Issue #659: Traits/Interfaces
// ============================================================================

#[test]
fn test_trait_declaration() {
    let source = r#"
        trait Serializable {
            function serialize(): String;
            function deserialize(data): Any;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse trait declaration: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Trait {
            name,
            methods,
            type_params,
            ..
        } => {
            assert_eq!(name, "Serializable");
            assert!(type_params.is_empty());
            assert_eq!(methods.len(), 2);
            assert_eq!(methods[0].name, "serialize");
            assert_eq!(methods[1].name, "deserialize");
        }
        other => panic!("Expected Trait, got: {:?}", other),
    }
}

#[test]
fn test_interface_keyword() {
    let source = r#"
        interface Printable {
            function toString(): String;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse interface: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Trait { name, methods, .. } => {
            assert_eq!(name, "Printable");
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "toString");
        }
        other => panic!("Expected Trait, got: {:?}", other),
    }
}

#[test]
fn test_generic_trait() {
    let source = r#"
        trait Comparable<T> {
            function compareTo(other): Number;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse generic trait: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Trait {
            name,
            type_params,
            methods,
            ..
        } => {
            assert_eq!(name, "Comparable");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
            assert_eq!(methods.len(), 1);
        }
        other => panic!("Expected Trait, got: {:?}", other),
    }
}

#[test]
fn test_class_implements_trait() {
    let source = r#"
        class JsonSerializer implements Serializable {
            function serialize() {
                return "{}";
            }
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse class implements: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "JsonSerializer");
            assert_eq!(class_decl.implements, vec!["Serializable".to_string()]);
        }
        other => panic!("Expected Class, got: {:?}", other),
    }
}

#[test]
fn test_class_implements_multiple_traits() {
    let source = r#"
        class User implements Serializable, Printable {
            constructor(name) {
                this.name = name;
            }
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse class implements multiple: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "User");
            assert_eq!(class_decl.implements.len(), 2);
            assert_eq!(class_decl.implements[0], "Serializable");
            assert_eq!(class_decl.implements[1], "Printable");
        }
        other => panic!("Expected Class, got: {:?}", other),
    }
}

#[test]
fn test_class_extends_and_implements() {
    let source = r#"
        class Dog extends Animal implements Pet, Serializable {
            constructor(name) {
                this.name = name;
            }
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse extends+implements: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "Dog");
            assert_eq!(class_decl.parent, Some("Animal".to_string()));
            assert_eq!(class_decl.implements.len(), 2);
            assert_eq!(class_decl.implements[0], "Pet");
            assert_eq!(class_decl.implements[1], "Serializable");
        }
        other => panic!("Expected Class, got: {:?}", other),
    }
}

#[test]
fn test_generic_class_with_implements() {
    let source = r#"
        class Stack<T> implements Iterable {
            constructor() {
                this.items = [];
            }
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse generic class with implements: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "Stack");
            assert_eq!(class_decl.type_params.len(), 1);
            assert_eq!(class_decl.type_params[0].name, "T");
            assert_eq!(class_decl.implements, vec!["Iterable".to_string()]);
        }
        other => panic!("Expected Class, got: {:?}", other),
    }
}

#[test]
fn test_trait_empty() {
    let source = r#"
        trait Marker {
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse empty trait: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Trait { name, methods, .. } => {
            assert_eq!(name, "Marker");
            assert!(methods.is_empty());
        }
        other => panic!("Expected Trait, got: {:?}", other),
    }
}

#[test]
fn test_class_without_implements_still_works() {
    let source = r#"
        class Car extends Vehicle {
            constructor() {
                this.speed = 0;
            }
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse class without implements: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "Car");
            assert_eq!(class_decl.parent, Some("Vehicle".to_string()));
            assert!(class_decl.implements.is_empty());
        }
        other => panic!("Expected Class, got: {:?}", other),
    }
}
