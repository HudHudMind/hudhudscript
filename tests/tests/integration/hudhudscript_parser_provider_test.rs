//! Provider declaration parsing tests

use hudhudscript_ast::{Decl, Expr, Literal, Stmt};
use hudhudscript_parser::parse;

#[test]
fn test_parse_provider_english() {
    let source = r#"
        provider openai_gpt4 {
            type: "openai",
            model: "gpt-4",
            api_key: "sk-test-key",
            max_tokens: 4000
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Provider { name, fields, .. }) => {
            assert_eq!(name, "openai_gpt4");
            assert_eq!(fields.len(), 4);

            // Check type field
            assert_eq!(fields[0].0, "type");
            match &fields[0].1 {
                Expr::Literal(Literal::String(s), _) => assert_eq!(s, "openai"),
                _ => panic!("Expected string literal for type"),
            }

            // Check model field
            assert_eq!(fields[1].0, "model");
            match &fields[1].1 {
                Expr::Literal(Literal::String(s), _) => assert_eq!(s, "gpt-4"),
                _ => panic!("Expected string literal for model"),
            }

            // Check max_tokens field
            assert_eq!(fields[3].0, "max_tokens");
            match &fields[3].1 {
                Expr::Literal(Literal::Number(n, _), _) => assert_eq!(*n, 4000.0),
                _ => panic!("Expected number literal for max_tokens"),
            }
        }
        _ => panic!("Expected provider declaration"),
    }
}

#[test]
fn test_parse_provider_turkish() {
    let source = r#"
        sağlayıcı anthropic_claude {
            type: "anthropic",
            model: "claude-3-opus"
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Provider { name, fields, .. }) => {
            assert_eq!(name, "anthropic_claude");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected provider declaration"),
    }
}

#[test]
fn test_parse_provider_japanese() {
    // Native Japanese: プロバイダー (provider)
    let source = r#"
        プロバイダー ollama_llama {
            type: "ollama",
            model: "llama2"
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Provider { name, fields, .. }) => {
            assert_eq!(name, "ollama_llama");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected provider declaration"),
    }
}

#[test]
fn test_parse_provider_with_nested_config() {
    let source = r#"
        provider openai_with_budget {
            type: "openai",
            model: "gpt-4",
            token_budget: {
                daily_limit: 100000,
                monthly_limit: 1000000
            }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Provider { name, fields, .. }) => {
            assert_eq!(name, "openai_with_budget");
            assert_eq!(fields.len(), 3);

            // Check token_budget is an object
            assert_eq!(fields[2].0, "token_budget");
            match &fields[2].1 {
                Expr::Object { properties, .. } => {
                    assert_eq!(properties.len(), 2);
                }
                _ => panic!("Expected object for token_budget"),
            }
        }
        _ => panic!("Expected provider declaration"),
    }
}
