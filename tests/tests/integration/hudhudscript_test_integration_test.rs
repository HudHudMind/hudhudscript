//! Integration tests for HudHudScript (Issue #26) — restored.
//!
//! End-to-end tests that parse source, run through the VM, and verify
//! variable values in the VM scope.  The legacy `interp.environment.get(x)`
//! pattern is replaced with `interp.get_variable(x).unwrap()`, which is
//! the VM-equivalent accessor the shim exposes.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;
use hudhudscript_runtime::ProviderRegistry;
use std::sync::Arc;

/// Parse + run through the VM; return the `Interpreter` so callers can
/// inspect globals via `get_variable(name)`.
fn run(source: &str) -> Interpreter {
    let stmts = parse(source).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.execute(&stmts).expect("execution failed");
    interp
}

// ---------------------------------------------------------------------------
// 1. Parse and interpret simple programs
// ---------------------------------------------------------------------------

#[test]
fn test_interpret_let_number() {
    let interp = run("let x = 42;");
    let val = interp.get_variable("x").expect("x not found");
    assert_eq!(val, Value16::number(42.0));
}

#[test]
fn test_interpret_let_string() {
    let interp = run(r#"let msg = "hello";"#);
    let val = interp.get_variable("msg").expect("msg not found");
    assert_eq!(val, Value16::string("hello".to_string()));
}

#[test]
fn test_interpret_let_boolean() {
    let interp = run("let flag = true;");
    let val = interp.get_variable("flag").expect("flag not found");
    assert_eq!(val, Value16::boolean(true));
}

#[test]
fn test_interpret_arithmetic() {
    let interp = run("let result = 2 + 3 * 4;");
    let val = interp.get_variable("result").expect("result not found");
    assert_eq!(val, Value16::number(14.0));
}

#[test]
fn test_interpret_multiple_statements() {
    let interp = run(r#"
        let a = 10;
        let b = 20;
        let c = a + b;
        "#);
    assert_eq!(
        interp.get_variable("c").expect("c not found"),
        Value16::number(30.0),
    );
}

#[test]
fn test_interpret_boolean_expression() {
    let interp = run("let ok = 5 > 3;");
    assert_eq!(
        interp.get_variable("ok").expect("ok not found"),
        Value16::boolean(true),
    );
}

#[test]
fn test_interpret_null() {
    let interp = run("let n = null;");
    assert_eq!(
        interp.get_variable("n").expect("n not found"),
        Value16::null(),
    );
}

#[test]
fn test_interpret_array() {
    let interp = run("let arr = [1, 2, 3];");
    let val = interp.get_variable("arr").expect("arr not found");
    assert_eq!(
        val,
        Value16::array(vec![
            Value16::number(1.0),
            Value16::number(2.0),
            Value16::number(3.0),
        ]),
    );
}

#[test]
fn test_interpret_function_call() {
    let interp = run(r#"
        fn add(a, b) {
            return a + b;
        }
        let result = add(3, 7);
        "#);
    assert_eq!(
        interp.get_variable("result").expect("result not found"),
        Value16::number(10.0),
    );
}

#[test]
fn test_interpret_if_true_branch() {
    let interp = run(r#"
        let x = 10;
        let y = 0;
        if (x > 5) {
            y = 1;
        }
        "#);
    assert_eq!(
        interp.get_variable("y").expect("y not found"),
        Value16::number(1.0),
    );
}

#[test]
fn test_interpret_if_false_branch() {
    let interp = run(r#"
        let x = 2;
        let y = 0;
        if (x > 5) {
            y = 1;
        } else {
            y = 99;
        }
        "#);
    assert_eq!(
        interp.get_variable("y").expect("y not found"),
        Value16::number(99.0),
    );
}

#[test]
fn test_interpret_while_loop() {
    let interp = run(r#"
        let i = 0;
        let sum = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        "#);
    assert_eq!(
        interp.get_variable("sum").expect("sum not found"),
        Value16::number(10.0),
    );
}

#[test]
fn test_interpret_recursive_function() {
    let interp = run(r#"
        fn factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        let result = factorial(5);
        "#);
    assert_eq!(
        interp.get_variable("result").expect("result not found"),
        Value16::number(120.0),
    );
}

#[test]
fn test_interpret_nested_function_calls() {
    let interp = run(r#"
        fn double(x) {
            return x * 2;
        }
        fn inc(x) {
            return x + 1;
        }
        let result = double(inc(4));
        "#);
    assert_eq!(
        interp.get_variable("result").expect("result not found"),
        Value16::number(10.0),
    );
}

// ---------------------------------------------------------------------------
// 2. Test provider registration
// ---------------------------------------------------------------------------

#[test]
fn test_provider_registry_creation() {
    let registry = ProviderRegistry::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let providers = rt.block_on(registry.list());
    assert!(providers.is_empty());
}

#[test]
fn test_interpreter_with_provider_registry() {
    let registry = Arc::new(ProviderRegistry::new());
    let mut interp = Interpreter::new();
    interp.set_provider_registry(registry.clone());

    let stmts = parse("let x = 1;").unwrap();
    interp.execute(&stmts).unwrap();
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(1.0),);
}

// ---------------------------------------------------------------------------
// 3. Test basic expressions (parsing only)
// ---------------------------------------------------------------------------

#[test]
fn test_parse_empty_program() {
    let result = parse("");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_parse_simple_let() {
    let stmts = parse("let x = 1 + 2;").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_function_decl() {
    let stmts = parse(
        r#"
        fn greet(name) {
            return name;
        }
        "#,
    )
    .unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_agent_decl() {
    let stmts = parse(
        r#"
        agent Helper {
            model: "gpt-4",
            task: "Assist users"
        }
        "#,
    )
    .unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_provider_decl() {
    let stmts = parse(
        r#"
        provider my_llm {
            type: "openai",
            model: "gpt-4",
            api_key: "test"
        }
        "#,
    )
    .unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_complex_program() {
    let source = r#"
        use postgres as db;

        provider main_llm {
            type: "openai",
            model: "gpt-4o",
            api_key: "sk-test",
            max_tokens: 4000
        }

        fn process(input) {
            let result = input + 1;
            return result;
        }

        agent DataAgent {
            model: "gpt-4o",
            task: "Analyze data"
        }

        let x = process(41);
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 5);
}

#[test]
fn test_parse_invalid_syntax_fails() {
    let result = parse("let = ;");
    assert!(result.is_err());
}

#[test]
fn test_parse_and_run_string_concat() {
    let interp = run(r#"let s = "hello" + " world";"#);
    assert_eq!(
        interp.get_variable("s").unwrap(),
        Value16::string("hello world".to_string()),
    );
}

#[test]
fn test_interpret_object_literal() {
    let interp = run(r#"let obj = { name: "test", value: 42 };"#);
    let val = interp.get_variable("obj").expect("obj not found");
    if let Some(map) = val.as_object() {
        assert_eq!(map.get("name"), Some(&Value16::string("test".to_string())));
        assert_eq!(map.get("value"), Some(&Value16::number(42.0)));
    } else {
        panic!("Expected Object, got {:?}", val);
    }
}
