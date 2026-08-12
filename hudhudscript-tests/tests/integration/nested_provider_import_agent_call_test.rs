use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_runtime::provider::{
    LLMRequest, LLMResponse, Provider, ProviderError, ProviderInfo, ProviderRegistry, ProviderType,
    TokenUsage, TokenUsageStats,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

fn setup_test_files(dir: &PathBuf) {
    let providers_src = r#"
provider DeepSeek {
    type: "deepseek",
    api_key: env("DEEPSEEK_API_KEY"),
    temperature: 0.7
}

provider OllamaLocal {
    type: "ollama",
    temperature: 0.7
}
"#;
    fs::write(dir.join("providers.hudhud"), providers_src).unwrap();

    let agents_src = r#"
use "providers.hudhud" as providers;

agent DeepSeekv4Pro {
    provider: providers.DeepSeek
    model: "deepseek-v4-pro"
    action translate(text, target) {
          return this.call({ prompt: "Translate to " + target + ": " + text });
    }
}

agent DeepSeekv4Flash {
    provider: providers.DeepSeek
    model: "deepseek-v4-flash"
    action translate(text, target) {
          return this.call({ prompt: "Translate to " + target + ": " + text });
    }
}

agent Gemma {
    provider: providers.OllamaLocal
    model: "gemma3:4b"
}

agent Qwen {
    provider: providers.OllamaLocal
    model: "qwen2.5-coder:7b"
    reasoning: true
}

agent Falcon {
    provider: providers.OllamaLocal
    model: "falcon3:10b"
}

agent MetinYazari {
    provider: providers.OllamaLocal
    model: "gemma3:4b"
    role: "Sen yaratıcı bir içerik yazarısın. Kısa ve öz cümleler kurarsın."
    temperature: 0.8

    action slogan_yaz(urun_adi) {
        return this.call({ prompt: urun_adi + " için slogan yaz" }).content;
    }
}

agent MantikAnalisti {
    provider: providers.DeepSeek
    model: "deepseek-v4-flash"
    role: "Sen sert mizaçlı bir mantık analistisin. Sadece gerçekleri söylersin."
    temperature: 0.1

    action veriyi_onayla(veri_metni) {
        return this.call({ prompt: veri_metni + " için slogan yaz dedik diğer ajana ve o da bu sloganı yazdı. Bu sloganı onaylar mısın? Onaylıyorum ya da onaylamaıyorum de." }).content;
    }
}
"#;
    fs::write(dir.join("agents.hudhud"), agents_src).unwrap();
}

struct MockProvider {
    name: String,
    model: String,
    response: String,
}

impl MockProvider {
    fn new(name: String, model: String, response: String) -> Self {
        Self { name, model, response }
    }
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: self.name.clone(),
            provider_type: ProviderType::OpenAI,
            model: self.model.clone(),
        }
    }

    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        let content = if request.prompt.contains("slogan yaz dedik") {
            "onaylandi"
        } else if request.prompt.contains("slogan yaz") {
            "Robot yapımı icin slogan yazıldı"
        } else {
            &self.response
        };

        Ok(LLMResponse {
            content: content.to_string(),
            model: self.model.clone(),
            finish_reason: "stop".to_string(),
            tokens_used: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            tool_calls: None,
        })
    }
    
    fn check_budget(&self, _tokens: usize) -> Result<(), ProviderError> {
        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(vec![self.model.clone()])
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_nested_provider_import_agent_call() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path().to_path_buf();
    setup_test_files(&dir);

    let main_src = r#"
use "agents.hudhud" as agents;

let slogan = agents.MetinYazari.slogan_yaz("Robot yapimi");
let kontrol = agents.MantikAnalisti.veriyi_onayla(slogan);
"#;
    let main_path = dir.join("author.hudhud");
    fs::write(&main_path, main_src).unwrap();

    let mut interpreter = Interpreter::new();
    let registry = Arc::new(ProviderRegistry::new());
    
    // Register "ollama" and "deepseek" types
    // The model must match what the agent being called declares. A registered
    // provider is only reused when it already serves the requested model —
    // LLMRequest carries no model field, so a mock advertising "gemma3" cannot
    // stand in for an agent asking for "gemma3:4b", and the call would fall
    // through to a real network provider.
    let mock_ollama = Arc::new(MockProvider::new("Ollama".to_string(), "gemma3:4b".to_string(), "mock response".to_string()));
    let mock_deepseek = Arc::new(MockProvider::new("DeepSeek".to_string(), "deepseek-v4-flash".to_string(), "mock response".to_string()));
    
    registry.register("ollama".to_string(), mock_ollama).await;
    registry.register("deepseek".to_string(), mock_deepseek).await;
    
    interpreter.set_provider_registry(registry);

    let ast = hudhudscript_parser::parse(main_src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    compiler.set_module_base_dir(dir.clone());
    let bc = compiler.compile(&ast).unwrap();
    interpreter.vm.execute(&bc).unwrap();

    let slogan_val = interpreter.vm.get_variable("slogan").unwrap();
    let kontrol_val = interpreter.vm.get_variable("kontrol").unwrap();

    let slogan_content = slogan_val.as_string().unwrap();
    assert!(slogan_content.contains("slogan yazıldı"));
    
    let kontrol_content = kontrol_val.as_string().unwrap();
    assert!(kontrol_content.contains("onaylandi"));
    
    // Test 2: Namespace Object Provider Not Null
    let agents_val = interpreter.vm.get_variable("agents").unwrap();
    let agents_obj = agents_val.as_object().unwrap();
    
    let metin_yazari = agents_obj.get("MetinYazari").unwrap().as_object().unwrap();
    let my_provider = metin_yazari.get("provider").unwrap();
    assert!(my_provider.is_object());
    assert_eq!(my_provider.as_object().unwrap().get("type").unwrap().as_str().unwrap(), "ollama");

    let mantik_analisti = agents_obj.get("MantikAnalisti").unwrap().as_object().unwrap();
    let ma_provider = mantik_analisti.get("provider").unwrap();
    assert!(ma_provider.is_object());
    assert_eq!(ma_provider.as_object().unwrap().get("type").unwrap().as_str().unwrap(), "deepseek");
}

#[tokio::test]
async fn test_provider_namespace_export() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path().to_path_buf();
    setup_test_files(&dir);
    
    let main_src = r#"
use "providers.hudhud" as providers;
let ds_type = providers.DeepSeek.type;
let ollama_type = providers.OllamaLocal.type;
"#;
    let main_path = dir.join("author.hudhud");
    fs::write(&main_path, main_src).unwrap();
    
    let mut interpreter = Interpreter::new();
    let ast = hudhudscript_parser::parse(main_src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    compiler.set_module_base_dir(dir.clone());
    let bc = compiler.compile(&ast).unwrap();
    interpreter.vm.execute(&bc).unwrap();
    
    let providers_val = interpreter.vm.get_variable("providers").unwrap();
    let providers_obj = providers_val.as_object().unwrap();
    
    assert!(providers_obj.contains_key("DeepSeek"));
    assert!(providers_obj.contains_key("OllamaLocal"));
    
    let ds_type = interpreter.vm.get_variable("ds_type").unwrap();
    assert_eq!(ds_type.as_str().unwrap(), "deepseek");
    
    let ollama_type = interpreter.vm.get_variable("ollama_type").unwrap();
    assert_eq!(ollama_type.as_str().unwrap(), "ollama");
    
    // Assert not exported:
    assert!(!providers_obj.contains_key("env"));
    assert!(!providers_obj.contains_key("this"));
    assert!(!providers_obj.contains_key("tcp"));
    assert!(!providers_obj.contains_key("fs"));
    assert!(!providers_obj.contains_key("__hudhud_env"));
}
