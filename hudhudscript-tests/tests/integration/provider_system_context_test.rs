use async_trait::async_trait;
use hudhudscript_compiler::Compiler;
use hudhudscript_runtime::provider::{
    LLMRequest, LLMResponse, Provider, ProviderError, ProviderInfo, ProviderRegistry, ProviderType,
    TokenUsage,
};
use hudhudscript_vm::{OutputLocale, SandboxConfig, VM};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct MockSystemProvider {
    recorded_request: Arc<Mutex<Option<LLMRequest>>>,
}

impl MockSystemProvider {
    fn new() -> Self {
        Self {
            recorded_request: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Provider for MockSystemProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        *self.recorded_request.lock().unwrap() = Some(request);
        Ok(LLMResponse {
            content: "mock response".to_string(),
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
            name: "MockSystemProvider".to_string(),
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

fn execute_script(script: &str, provider: Arc<MockSystemProvider>) {
    let mut compiler = Compiler::new();
    let stmts = hudhudscript_parser::parse(script).unwrap();
    let bytecode = compiler.compile(&stmts).unwrap();

    let mut registry = ProviderRegistry::new();

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

    vm.execute(&bytecode).unwrap();
}

#[test]
fn test_agent_role_system_context() {
    let mock = Arc::new(MockSystemProvider::new());

    let script = r#"
        provider MockAI {
            type: "mock_provider"
        }

        agent MetinYazari {
            provider: MockAI
            role: "Sen yaratıcı bir içerik yazarısın."
            action run() {
                return this.call({ prompt: "Robot için slogan yaz" });
            }
        }

        let result = MetinYazari.run();
    "#;

    execute_script(script, mock.clone());

    let request = mock.recorded_request.lock().unwrap().clone().unwrap();
    assert_eq!(request.prompt, "Robot için slogan yaz");
    let system_prompt = request.system_prompt.unwrap();
    assert!(system_prompt.contains("Sen yaratıcı bir içerik yazarısın."));
    assert!(system_prompt.contains("[Agent Role]"));
}

#[test]
fn test_constitution_and_role() {
    let mock = Arc::new(MockSystemProvider::new());

    let script = r#"
        constitution SafeAI {
            description: "Dürüst ve güvenli cevap ver.",
            laws: [{
                name: "NoLie",
                description: "Yalan söyleme.",
                enforcement: advisory,
                rules: ["Bilmediğin şeyi uydurma"]
            }]
        }

        provider MockAI {
            type: "mock_provider"
        }

        agent A {
            provider: MockAI
            role: "Kısa cevap ver."
            action run() {
                return this.call({ prompt: "Bir şey açıkla" });
            }
        }

        let result = A.run();
    "#;

    execute_script(script, mock.clone());

    let request = mock.recorded_request.lock().unwrap().clone().unwrap();
    assert_eq!(request.prompt, "Bir şey açıkla");
    let system_prompt = request.system_prompt.unwrap();
    assert!(system_prompt.contains("SafeAI"));
    assert!(system_prompt.contains("NoLie"));
    assert!(system_prompt.contains("Kısa cevap ver."));
    assert!(system_prompt.contains("Bilmediğin şeyi uydurma"));
}

#[test]
fn test_explicit_system_prompt_order() {
    let mock = Arc::new(MockSystemProvider::new());

    let script = r#"
        provider MockAI {
            type: "mock_provider"
        }

        agent A {
            provider: MockAI
            role: "Role text"
            action run() {
                return this.call({
                    system_prompt: "Call system text",
                    prompt: "User text"
                });
            }
        }

        let result = A.run();
    "#;

    execute_script(script, mock.clone());

    let request = mock.recorded_request.lock().unwrap().clone().unwrap();
    assert_eq!(request.prompt, "User text");
    let system_prompt = request.system_prompt.unwrap();
    assert!(system_prompt.contains("Role text"));
    assert!(system_prompt.contains("Call system text"));

    // Ensure order
    let role_idx = system_prompt.find("Role text").unwrap();
    let call_idx = system_prompt.find("Call system text").unwrap();
    assert!(
        role_idx < call_idx,
        "Role should appear before explicit call system prompt"
    );
}

#[test]
fn test_provider_system_context_compose_order() {
    let mock = Arc::new(MockSystemProvider::new());

    let script = r#"
        provider MockAI {
            type: "mock_provider"
            system: "Provider system text"
        }

        agent A {
            provider: MockAI
            role: "Agent role text"
            action run() {
                return this.call({
                    system_prompt: "Call system text",
                    prompt: "User text"
                });
            }
        }

        let result = A.run();
    "#;

    execute_script(script, mock.clone());

    let request = mock.recorded_request.lock().unwrap().clone().unwrap();
    assert_eq!(request.prompt, "User text");
    let system_prompt = request.system_prompt.unwrap();
    assert!(system_prompt.contains("Provider system text"));
    assert!(system_prompt.contains("Agent role text"));
    assert!(system_prompt.contains("Call system text"));

    let prov_idx = system_prompt.find("Provider system text").unwrap();
    let role_idx = system_prompt.find("Agent role text").unwrap();
    let call_idx = system_prompt.find("Call system text").unwrap();
    assert!(
        prov_idx < role_idx,
        "Provider system should appear before agent role"
    );
    assert!(
        role_idx < call_idx,
        "Agent role should appear before explicit call system prompt"
    );
}

#[test]
fn test_agent_system_context_multiple_fields() {
    let mock = Arc::new(MockSystemProvider::new());

    let script = r#"
        provider MockAI {
            type: "mock_provider"
        }

        agent A {
            provider: MockAI
            role: "My Role"
            system: "My System"
            action run() {
                return this.call({
                    prompt: "User text"
                });
            }
        }

        let result = A.run();
    "#;

    execute_script(script, mock.clone());

    let request = mock.recorded_request.lock().unwrap().clone().unwrap();
    let system_prompt = request.system_prompt.unwrap();
    assert!(system_prompt.contains("My Role"));
    assert!(system_prompt.contains("My System"));
}
#[test]
fn test_nested_imported_agent_system_context() {
    let mock = Arc::new(MockSystemProvider::new());

    // Create a temporary directory structure
    let temp_dir = std::env::temp_dir().join("hudhud_nested_test");
    std::fs::create_dir_all(&temp_dir).unwrap();

    // Write providers.hudhud
    let providers_script = r#"
        provider MockAI {
            type: "mock_provider"
        }
        export MockAI;
    "#;
    std::fs::write(temp_dir.join("providers.hudhud"), providers_script).unwrap();

    // Write agents.hudhud
    let agents_script = r#"
        use "providers.hudhud" as providers

        agent MetinYazari {
            provider: providers.MockAI
            role: "Sen yaratıcı bir içerik yazarısın."
            system: "Harika içerikler üretirsin."
            action run() {
                return this.call({ prompt: "Merhaba" });
            }
        }
        export MetinYazari;
    "#;
    std::fs::write(temp_dir.join("agents.hudhud"), agents_script).unwrap();

    // Write author.hudhud
    let author_script = r#"
        use "agents.hudhud" as agents

        let result = agents.MetinYazari.run();
    "#;
    std::fs::write(temp_dir.join("author.hudhud"), author_script).unwrap();

    // Compile and run author.hudhud
    let mut compiler = Compiler::new();
    compiler.set_module_base_dir(temp_dir.clone());
    let stmts = hudhudscript_parser::parse(
        &std::fs::read_to_string(temp_dir.join("author.hudhud")).unwrap(),
    )
    .unwrap();
    let bytecode = compiler.compile(&stmts).unwrap();

    let mut registry = ProviderRegistry::new();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        registry
            .register("mock_provider".to_string(), mock.clone())
            .await;
    });

    let mut vm = VM::new();
    let sandbox = SandboxConfig {
        allowed_paths: vec![temp_dir.to_string_lossy().to_string()],
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

    vm.execute(&bytecode).unwrap();

    let request = mock.recorded_request.lock().unwrap().clone().unwrap();
    assert_eq!(request.prompt, "Merhaba");
    let system_prompt = request.system_prompt.unwrap();
    assert!(system_prompt.contains("Sen yaratıcı bir içerik yazarısın."));
    assert!(system_prompt.contains("Harika içerikler üretirsin."));

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).unwrap();
}
