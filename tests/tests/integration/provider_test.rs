//! Provider declaration interpreter tests

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

#[test]
fn test_provider_declaration() {
    let source = r#"
        provider openai_gpt4 {
            type: "openai",
            model: "gpt-4",
            api_key: "sk-test-key",
            max_tokens: 4000
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());

    // Check that provider was registered as a variable
    let provider = interpreter.get_variable("openai_gpt4");
    assert!(provider.is_ok(), "Provider not found in environment");

    // Verify it's an object with the correct fields
    let provider = provider.unwrap();
    if let Some(obj) = provider.as_object() {
        // 4 declared fields + 1 injected "name" field = 5
        assert_eq!(obj.len(), 5);

        // Check type field
        match obj.get("type") {
            Some(v) => {
                if let Some(s) = v.as_str() {
                    assert_eq!(s, "openai");
                } else {
                    panic!("Expected string for type field");
                }
            }
            None => panic!("Expected string for type field"),
        }

        // Check model field
        match obj.get("model") {
            Some(v) => {
                if let Some(s) = v.as_str() {
                    assert_eq!(s, "gpt-4");
                } else {
                    panic!("Expected string for model field");
                }
            }
            None => panic!("Expected string for model field"),
        }

        // Check max_tokens field
        match obj.get("max_tokens") {
            Some(v) => {
                if let Some(n) = v.as_number() {
                    assert_eq!(n, 4000.0);
                } else {
                    panic!("Expected number for max_tokens field");
                }
            }
            None => panic!("Expected number for max_tokens field"),
        }
    } else {
        panic!("Expected object for provider");
    }
}

#[test]
fn test_multiple_providers() {
    let source = r#"
        provider openai_gpt4 {
            type: "openai",
            model: "gpt-4"
        }
        
        provider anthropic_claude {
            type: "anthropic",
            model: "claude-3-opus"
        }
        
        provider ollama_llama {
            type: "ollama",
            model: "llama2"
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());

    // Check all three providers exist
    assert!(interpreter.get_variable("openai_gpt4").is_ok());
    assert!(interpreter.get_variable("anthropic_claude").is_ok());
    assert!(interpreter.get_variable("ollama_llama").is_ok());
}

#[test]
fn test_provider_with_nested_config() {
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

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());

    let provider = interpreter.get_variable("openai_with_budget").unwrap();
    if let Some(obj) = provider.as_object() {
        // Check token_budget is an object
        match obj.get("token_budget") {
            Some(v) => {
                if let Some(budget) = v.as_object() {
                    assert_eq!(budget.len(), 2);

                    match budget.get("daily_limit") {
                        Some(v) => {
                            if let Some(n) = v.as_number() {
                                assert_eq!(n, 100000.0);
                            } else {
                                panic!("Expected number for daily_limit");
                            }
                        }
                        None => panic!("Expected number for daily_limit"),
                    }

                    match budget.get("monthly_limit") {
                        Some(v) => {
                            if let Some(n) = v.as_number() {
                                assert_eq!(n, 1000000.0);
                            } else {
                                panic!("Expected number for monthly_limit");
                            }
                        }
                        None => panic!("Expected number for monthly_limit"),
                    }
                } else {
                    panic!("Expected object for token_budget");
                }
            }
            None => panic!("Expected object for token_budget"),
        }
    } else {
        panic!("Expected object for provider")
    }
}
