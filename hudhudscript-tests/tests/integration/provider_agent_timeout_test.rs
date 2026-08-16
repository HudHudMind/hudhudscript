use async_trait::async_trait;
use hudhudscript_compiler::Compiler;
use hudhudscript_runtime::provider::{
    LLMRequest, LLMResponse, Provider, ProviderError, ProviderInfo, ProviderRegistry, ProviderType,
    TokenUsage,
};
use hudhudscript_vm::{OutputLocale, SandboxConfig, VM};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct MockProvider {
    recorded_timeout: Arc<Mutex<Option<u64>>>,
}

impl MockProvider {
    fn new() -> Self {
        Self {
            recorded_timeout: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        *self.recorded_timeout.lock().unwrap() = request.timeout_secs;
        Ok(LLMResponse {
            content: format!("Timeout was: {:?}", request.timeout_secs),
            tokens_used: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            model: "mock-model".to_string(),
            finish_reason: "stop".to_string(),
            tool_calls: None,
        })
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "MockProvider".to_string(),
            model: "mock".to_string(),
            provider_type: ProviderType::OpenAI,
        }
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["mock".to_string()])
    }

    fn check_budget(&self, _tokens: usize) -> Result<(), ProviderError> {
        Ok(())
    }

    async fn get_usage_stats(&self) -> hudhudscript_runtime::provider::TokenUsageStats {
        hudhudscript_runtime::provider::TokenUsageStats {
            daily_usage: 0,
            monthly_usage: 0,
            estimated_cost: 0.0,
            last_reset: std::time::SystemTime::now(),
        }
    }
}

fn execute_script_with_vm(
    script: &str,
    provider: Arc<MockProvider>,
    setup_vm: impl FnOnce(&mut VM),
) -> Result<(), hudhudscript_errors::Error> {
    let mut compiler = Compiler::new();
    let stmts = hudhudscript_parser::parse(script).unwrap();
    let bytecode = compiler.compile(&stmts).unwrap();

    let registry = ProviderRegistry::new();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        registry
            .register("mock_provider".to_string(), provider.clone())
            .await;
    });

    let mut vm = VM::new();
    let sandbox = SandboxConfig {
        allowed_paths: vec![],
        allowed_hosts: vec![],
        allow_file_read: true,
        allow_file_write: false,
        allow_network: true,
        allow_process: false,
        allowed_commands: vec![],
        denied_commands: vec![],
    };
    vm.with_sandbox(sandbox);
    vm.set_provider_registry(Arc::new(registry));

    setup_vm(&mut vm);

    vm.execute(&bytecode)
}

fn execute_script(script: &str, provider: Arc<MockProvider>) {
    execute_script_with_vm(script, provider, |_| {}).unwrap();
}

#[test]
fn test_agent_timeout_overrides_call_site_timeout() {
    let mock = Arc::new(MockProvider::new());
    let script = r#"
        provider MyProv {
            type: "mock_provider",
            timeout_secs: 50
        }

        agent MyAgent {
            provider: MyProv,
            timeout_secs: 75
        }

        return MyAgent.call({ prompt: "hello", timeout_secs: 99 })
    "#;
    execute_script(script, mock.clone());
    assert_eq!(*mock.recorded_timeout.lock().unwrap(), Some(75));
}

#[test]
fn test_provider_timeout_used_when_agent_timeout_absent() {
    let mock = Arc::new(MockProvider::new());
    let script = r#"
        provider MyProv {
            type: "mock_provider",
            timeout_secs: 42
        }

        agent MyAgent {
            provider: MyProv
        }

        return MyAgent.call({ prompt: "hello" })
    "#;
    execute_script(script, mock.clone());
    assert_eq!(*mock.recorded_timeout.lock().unwrap(), Some(42));
}

#[test]
fn test_runtime_provider_timeout_secs_used_when_no_agent_or_provider_timeout() {
    let mock = Arc::new(MockProvider::new());
    let script = r#"
        provider MyProv {
            type: "mock_provider"
        }

        agent MyAgent {
            provider: MyProv
        }

        return MyAgent.call({ prompt: "hello" })
    "#;

    // Test with VM default
    execute_script(script, mock.clone());
    assert_eq!(*mock.recorded_timeout.lock().unwrap(), Some(120));

    // Test with custom runtime default
    execute_script_with_vm(script, mock.clone(), |vm| {
        vm.with_provider_timeout_secs(180);
    })
    .unwrap();
    assert_eq!(*mock.recorded_timeout.lock().unwrap(), Some(180));
}

#[test]
fn test_toml_provider_timeout_and_script_override() {
    let mock = Arc::new(MockProvider::new());
    let script = r#"
        provider MyProv {
            type: "mock_provider"
        }

        agent MyAgent {
            provider: MyProv
        }

        return MyAgent.call({ prompt: "hello" })
    "#;

    // TOML overrides empty script
    execute_script_with_vm(script, mock.clone(), |vm| {
        let mut toml_providers = std::collections::HashMap::new();
        let mut prov_config = std::collections::HashMap::new();
        prov_config.insert("timeout".to_string(), "123".to_string());
        toml_providers.insert("MyProv".to_string(), prov_config);
        vm.set_toml_providers(toml_providers);
    })
    .unwrap();
    assert_eq!(*mock.recorded_timeout.lock().unwrap(), Some(123));

    // Script overrides TOML
    let script2 = r#"
        provider MyProv {
            type: "mock_provider",
            timeout_secs: 45
        }

        agent MyAgent {
            provider: MyProv
        }

        return MyAgent.call({ prompt: "hello" })
    "#;

    execute_script_with_vm(script2, mock.clone(), |vm| {
        let mut toml_providers = std::collections::HashMap::new();
        let mut prov_config = std::collections::HashMap::new();
        prov_config.insert("timeout".to_string(), "123".to_string());
        toml_providers.insert("MyProv".to_string(), prov_config);
        vm.set_toml_providers(toml_providers);
    })
    .unwrap();
    assert_eq!(*mock.recorded_timeout.lock().unwrap(), Some(45));
}

#[test]
fn test_invalid_timeout_values_fail_clearly() {
    let mock = Arc::new(MockProvider::new());

    // 1. Negative in script
    let script1 = r#"
        provider MyProv {
            type: "mock_provider",
            timeout: -1
        }
        agent MyAgent { provider: MyProv }
        return MyAgent.call({ prompt: "hello" })
    "#;
    let err1 = execute_script_with_vm(script1, mock.clone(), |_| {}).unwrap_err();
    assert!(format!("{:?}", err1).contains("Invalid provider config timeout"));

    // 2. String "abc" in TOML
    let script2 = r#"
        provider MyProv { type: "mock_provider" }
        agent MyAgent { provider: MyProv }
        return MyAgent.call({ prompt: "hello" })
    "#;
    let err2 = execute_script_with_vm(script2, mock.clone(), |vm| {
        let mut toml_providers = std::collections::HashMap::new();
        let mut prov_config = std::collections::HashMap::new();
        prov_config.insert("timeout".to_string(), "abc".to_string());
        toml_providers.insert("MyProv".to_string(), prov_config);
        vm.set_toml_providers(toml_providers);
    })
    .unwrap_err();
    assert!(format!("{:?}", err2).contains("Invalid provider config timeout"));

    // 3. String "0" in TOML
    let err3 = execute_script_with_vm(script2, mock.clone(), |vm| {
        let mut toml_providers = std::collections::HashMap::new();
        let mut prov_config = std::collections::HashMap::new();
        prov_config.insert("timeout".to_string(), "0".to_string());
        toml_providers.insert("MyProv".to_string(), prov_config);
        vm.set_toml_providers(toml_providers);
    })
    .unwrap_err();
    assert!(format!("{:?}", err3).contains("Invalid provider config timeout"));
    assert!(format!("{:?}", err3).contains("expected positive seconds"));
}
