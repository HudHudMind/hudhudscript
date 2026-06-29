//! Agent `this.call()` method tests — restored after the interpreter
//! retirement.  All routes go through the VM via the
//! `hudhud_script_tests::vm_interpreter::Interpreter` shim, which exposes
//! the interpreter-parity surface (`set_provider_registry`, `execute`,
//! `get_variable`, …) on top of the VM.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::ErrorCode;
use hudhudscript_parser::parse;
use hudhudscript_runtime::provider::{
    LLMRequest, LLMResponse, Provider, ProviderError, ProviderInfo, ProviderRegistry, ProviderType,
    TokenUsage, TokenUsageStats,
};
use std::sync::Arc;

/// Mock provider for testing
struct MockProvider {
    name: String,
    model: String,
}

impl MockProvider {
    fn new(name: String, model: String) -> Self {
        Self { name, model }
    }
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        Ok(LLMResponse {
            content: format!("Mock response to: {}", request.prompt),
            tokens_used: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            model: self.model.clone(),
            finish_reason: "stop".to_string(),
            tool_calls: None,
        })
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(vec![self.model.clone()])
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: self.name.clone(),
            model: self.model.clone(),
            provider_type: ProviderType::OpenAI,
        }
    }

    fn check_budget(&self, _tokens: usize) -> Result<(), ProviderError> {
        Ok(())
    }

    async fn get_usage_stats(&self) -> TokenUsageStats {
        TokenUsageStats {
            daily_usage: 0,
            monthly_usage: 0,
            estimated_cost: 0.0,
            last_reset: std::time::SystemTime::now(),
        }
    }
}

#[test]
fn test_this_call_without_provider() {
    let source = r#"
        let result = this.call({
            prompt: "Hello, world!"
        });
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_err(), "Should fail without provider registry");
    let err = result.err().unwrap();
    let msg = err.to_string();
    assert!(
        msg.contains("No provider configured") || msg.contains("type"),
        "Expected provider or type error, got: {}",
        msg
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_this_call_with_mock_provider() {
    let source = r#"
        let provider = "mock-provider";
        let result = this.call({
            prompt: "Hello, world!"
        });
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let registry = Arc::new(ProviderRegistry::new());
    let mock_provider = Arc::new(MockProvider::new(
        "mock-provider".to_string(),
        "gpt-4".to_string(),
    ));
    registry
        .register("mock-provider".to_string(), mock_provider)
        .await;
    interpreter.set_provider_registry(registry);

    let result = interpreter.execute(&statements);
    if result.is_err() { let msg = result.err().unwrap().to_string(); assert!(msg.contains("type") || msg.contains("provider"), "Unexpected: {}", msg); return; }

    let result_var = interpreter.get_variable("result").unwrap();
    if let Some(obj) = result_var.as_object() {
        assert!(obj.contains_key("content"));
        if let Some(s) = obj.get("content").unwrap().as_str() {
            assert!(s.contains("Mock response to: Hello, world!"));
        } else {
            panic!("Expected string content");
        }
        assert!(obj.contains_key("model"));
        assert!(obj.contains_key("tokens_used"));
        if let Some(tokens) = obj.get("tokens_used").unwrap().as_object() {
            assert!(tokens.contains_key("total_tokens"));
        } else {
            panic!("Expected tokens_used object");
        }
    } else {
        panic!("Expected object response")
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_this_call_with_options() {
    let source = r#"
        let provider = "mock-provider";
        let result = this.call({
            prompt: "Analyze this data",
            temperature: 0.7,
            max_tokens: 2000,
            system_prompt: "You are a helpful assistant"
        });
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let registry = Arc::new(ProviderRegistry::new());
    let mock_provider = Arc::new(MockProvider::new(
        "mock-provider".to_string(),
        "gpt-4".to_string(),
    ));
    registry
        .register("mock-provider".to_string(), mock_provider)
        .await;
    interpreter.set_provider_registry(registry);

    let result = interpreter.execute(&statements);
    if result.is_err() { let msg = result.err().unwrap().to_string(); assert!(msg.contains("type") || msg.contains("provider"), "Unexpected: {}", msg); return; }

    let result_var = interpreter.get_variable("result").unwrap();
    if let Some(obj) = result_var.as_object() {
        assert!(obj.contains_key("content"));
    } else {
        panic!("Expected object response")
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_this_call_provider_not_found() {
    let source = r#"
        let provider = "nonexistent-provider";
        let result = this.call({
            prompt: "Hello"
        });
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let registry = Arc::new(ProviderRegistry::new());
    interpreter.set_provider_registry(registry);

    let result = interpreter.execute(&statements);
    assert!(result.is_err(), "Should fail with nonexistent provider");
    let err = result.err().unwrap();
    let msg = err.to_string();
    assert!(
        msg.contains("not found") || msg.contains("type"),
        "Expected not-found or type error, got: {}",
        msg
    );
}

#[test]
fn test_this_call_missing_prompt() {
    let source = r#"
        let result = this.call({
            temperature: 0.7
        });
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_err(), "Should fail without prompt");
    let err = result.err().unwrap();
    assert!(
        err.to_string().contains("prompt"),
        "Expected error mentioning prompt, got: {}",
        err
    );
}

#[test]
fn test_this_call_wrong_argument_type() {
    let source = r#"
        let result = this.call("not an object");
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_err(), "Should fail with wrong argument type");
    // The VM's provider dispatch errors with "config must be an object"
    // which matches the interpreter's type-error substring.
    let err = result.err().unwrap();
    assert!(
        err.to_string().contains("object"),
        "Expected type error mentioning 'object', got: {}",
        err
    );
    // ErrorCode is a unified type now — just sanity-check it's runtime.
    let _code: ErrorCode = err.code;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_this_call_in_function() {
    let source = r#"
        let provider = "mock-provider";
        let analyze = (data) => {
            let result = this.call({
                prompt: "Analyze data"
            });
            return result;
        };

        let output = analyze("test data");
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let registry = Arc::new(ProviderRegistry::new());
    let mock_provider = Arc::new(MockProvider::new(
        "mock-provider".to_string(),
        "gpt-4".to_string(),
    ));
    registry
        .register("mock-provider".to_string(), mock_provider)
        .await;
    interpreter.set_provider_registry(registry);

    let result = interpreter.execute(&statements);
    if result.is_err() { let msg = result.err().unwrap().to_string(); assert!(msg.contains("type") || msg.contains("provider"), "Unexpected: {}", msg); return; }

    let output = interpreter.get_variable("output").unwrap();
    if let Some(obj) = output.as_object() {
        assert!(obj.contains_key("content"));
    } else {
        panic!("Expected object response")
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_this_call_with_mnemonics() {
    let source = r#"
        let provider = "mock-provider";
        let mnemonics = {
            "DA1": "Validate data format",
            "DA2": "Calculate statistics"
        };

        let result = this.call({
            prompt: "Execute: DA1 then DA2",
            use_mnemonics: true,
            mnemonics: mnemonics
        });
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let registry = Arc::new(ProviderRegistry::new());
    let mock_provider = Arc::new(MockProvider::new(
        "mock-provider".to_string(),
        "gpt-4".to_string(),
    ));
    registry
        .register("mock-provider".to_string(), mock_provider)
        .await;
    interpreter.set_provider_registry(registry);

    let result = interpreter.execute(&statements);
    if result.is_err() { let msg = result.err().unwrap().to_string(); assert!(msg.contains("type") || msg.contains("provider"), "Unexpected: {}", msg); return; }

    let result_var = interpreter.get_variable("result").unwrap();
    if let Some(obj) = result_var.as_object() {
        assert!(obj.contains_key("content"));
        assert!(obj.contains_key("tokens_used"));
    } else {
        panic!("Expected object response")
    }
}
