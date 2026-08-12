// Tests for generics (#658) and traits/interfaces (#659) — compiler/VM

use hudhudscript_ast::*;
use hudhudscript_compiler::Compiler;
use hudhudscript_vm::VM;

fn dummy_span() -> Span {
    Span::new(Position::new(1, 1, 0), Position::new(1, 1, 0))
}

#[test]
fn test_generic_function_compiles() {
    let stmts = vec![
        // function identity<T>(x) { return x; }
        Stmt::Function {
            name: "identity".to_string(),
            params: vec!["x".to_string()],
            body: vec![Stmt::Return {
                value: Some(Expr::Identifier("x".to_string(), dummy_span())),
                span: dummy_span(),
            }],
            is_async: false,
            is_generator: false,
            type_params: vec![GenericParam {
                name: "T".to_string(),
                constraint: None,
                span: dummy_span(),
            }],
            span: dummy_span(),
        },
        // let result = identity(42);
        Stmt::Let {
            name: "result".to_string(),
            value: Expr::Call {
                callee: Box::new(Expr::Identifier("identity".to_string(), dummy_span())),
                args: vec![Expr::Literal(Literal::Number(42.0, false), dummy_span())],
                span: dummy_span(),
            },
            span: dummy_span(),
        },
    ];

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts);
    assert!(bytecode.is_ok(), "Compilation failed: {:?}", bytecode.err());

    let mut vm = VM::new();
    let result = vm.execute(&bytecode.unwrap());
    assert!(result.is_ok(), "VM execution failed: {:?}", result.err());
}

#[test]
fn test_trait_declaration_compiles() {
    let stmts = vec![
        // trait Serializable { function serialize(): String; }
        Stmt::Trait {
            name: "Serializable".to_string(),
            type_params: vec![],
            methods: vec![TraitMethodSig {
                name: "serialize".to_string(),
                params: vec![],
                return_type: Some(TypeAnnotation::String),
                span: dummy_span(),
            }],
            span: dummy_span(),
        },
    ];

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts);
    assert!(
        bytecode.is_ok(),
        "Trait compilation failed: {:?}",
        bytecode.err()
    );

    let mut vm = VM::new();
    let result = vm.execute(&bytecode.unwrap());
    assert!(
        result.is_ok(),
        "VM execution with trait failed: {:?}",
        result.err()
    );
}

#[test]
fn test_generic_class_compiles() {
    let stmts = vec![
        // class Box<T> { constructor(value) { this.value = value; } }
        Stmt::Class(ClassDecl {
            is_abstract: false,
            name: "Box".to_string(),
            parent: None,
            type_params: vec![GenericParam {
                name: "T".to_string(),
                constraint: None,
                span: dummy_span(),
            }],
            implements: vec![],
            members: vec![ClassMember::Constructor {
                params: vec![Param {
                    name: "value".to_string(),
                    type_annotation: None,
                    span: dummy_span(),
                }],
                body: vec![Stmt::Assignment {
                    target: Expr::Member {
                        object: Box::new(Expr::This(dummy_span())),
                        property: "value".to_string(),
                        span: dummy_span(),
                    },
                    value: Expr::Identifier("value".to_string(), dummy_span()),
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }],
            span: dummy_span(),
        }),
        // let b = new Box(42);
        Stmt::Let {
            name: "b".to_string(),
            value: Expr::New {
                class_name: "Box".to_string(),
                args: vec![Expr::Literal(Literal::Number(42.0, false), dummy_span())],
                span: dummy_span(),
            },
            span: dummy_span(),
        },
    ];

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts);
    assert!(
        bytecode.is_ok(),
        "Generic class compilation failed: {:?}",
        bytecode.err()
    );

    let mut vm = VM::new();
    let result = vm.execute(&bytecode.unwrap());
    assert!(
        result.is_ok(),
        "VM execution with generic class failed: {:?}",
        result.err()
    );
}

#[test]
fn test_class_implements_trait_compiles() {
    let stmts = vec![
        // trait Greeter {}
        Stmt::Trait {
            name: "Greeter".to_string(),
            type_params: vec![],
            methods: vec![],
            span: dummy_span(),
        },
        // class Hello implements Greeter {}
        Stmt::Class(ClassDecl {
            is_abstract: false,
            name: "Hello".to_string(),
            parent: None,
            type_params: vec![],
            implements: vec!["Greeter".to_string()],
            members: vec![],
            span: dummy_span(),
        }),
    ];

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts);
    assert!(
        bytecode.is_ok(),
        "Class implements compilation failed: {:?}",
        bytecode.err()
    );

    let mut vm = VM::new();
    let result = vm.execute(&bytecode.unwrap());
    assert!(
        result.is_ok(),
        "VM execution with implements failed: {:?}",
        result.err()
    );
}
