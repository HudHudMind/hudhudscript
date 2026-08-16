//! M3 — persona/rol `swarm.run` ve `council.vote` yollarında da akmalı.
//!
//! Kök neden (v0.8.217): `dispatch_swarm_run` receiver'a PROVIDER objesini
//! koyuyordu; ortak sistem-prompt şeridi (`provider_build_system_context`)
//! [Agent Role]'ü yalnız receiver AGENT objesiyken kurar → persona
//! council/swarm'da düşüyordu. Fix: receiver = agent objesi — `Agent.call`
//! ile AYNI şerit (Kural 7, tek yol). council.vote ve community.run zaten
//! `dispatch_swarm_run`'a aktığı için tek fix üç yolu düzeltir.
use async_trait::async_trait;
use hudhudscript_compiler::Compiler;
use hudhudscript_runtime::provider::{
    LLMRequest, LLMResponse, Provider, ProviderError, ProviderInfo, ProviderRegistry, ProviderType,
    TokenUsage,
};
use hudhudscript_vm::{SandboxConfig, VM};
use std::sync::{Arc, Mutex};

/// Her isteği sırayla kaydeden mock — swarm'da AJAN BAŞINA istek doğrulanır.
#[derive(Debug, Clone)]
struct RecordingProvider {
    requests: Arc<Mutex<Vec<LLMRequest>>>,
}

impl RecordingProvider {
    fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Provider for RecordingProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        self.requests.lock().unwrap().push(request);
        Ok(LLMResponse {
            content: "ok".to_string(),
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
            name: "RecordingProvider".to_string(),
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

fn execute_script(script: &str, provider: Arc<RecordingProvider>) {
    let stmts = hudhudscript_parser::parse(script).unwrap();
    let bytecode = Compiler::new().compile(&stmts).unwrap();

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
    vm.with_sandbox(SandboxConfig {
        allowed_paths: vec![],
        allowed_hosts: vec![],
        allow_file_read: true,
        allow_file_write: false,
        allow_network: true,
        allow_process: false,
        allowed_commands: vec![],
        denied_commands: vec![],
    });
    vm.set_provider_registry(Arc::new(registry));
    vm.execute(&bytecode).unwrap();
}

const SWARM_SCRIPT: &str = r#"
    provider MockAI {
        type: "mock_provider"
    }

    agent Arastirmaci {
        provider: MockAI
        role: "Sen titiz bir arastirmacisin."
    }

    agent Yazar {
        provider: MockAI
        role: "Sen yaratici bir yazarsin."
    }

    swarm Takim {
        agents: ["Arastirmaci", "Yazar"]
        strategy: "sequential"
    }

    let sonuc = Takim.run("Kuantum ozetle")
"#;

#[test]
fn m3_swarm_run_carries_persona_per_agent() {
    let mock = Arc::new(RecordingProvider::new());
    execute_script(SWARM_SCRIPT, mock.clone());

    let requests = mock.requests.lock().unwrap().clone();
    assert_eq!(requests.len(), 2, "swarm iki ajan için iki istek atmalı");

    let sys0 = requests[0].system_prompt.clone().unwrap_or_default();
    let sys1 = requests[1].system_prompt.clone().unwrap_or_default();
    assert!(
        sys0.contains("[Agent Role]") && sys0.contains("Sen titiz bir arastirmacisin."),
        "1. ajanın isteği kendi personasını taşımalı, got: {sys0:?}"
    );
    assert!(
        sys1.contains("[Agent Role]") && sys1.contains("Sen yaratici bir yazarsin."),
        "2. ajanın isteği kendi personasını taşımalı, got: {sys1:?}"
    );
}

#[test]
fn m3_council_vote_carries_persona() {
    let mock = Arc::new(RecordingProvider::new());
    let script = r#"
        provider MockAI {
            type: "mock_provider"
        }

        agent Yargic {
            provider: MockAI
            role: "Sen adil bir yargicsin."
        }

        council Kurul {
            members: [
                { agent: "Yargic", role: "judge" }
            ]
            rules: ["majority"]
        }

        let karar = Kurul.vote("Onaylansin mi?")
    "#;
    execute_script(script, mock.clone());

    let requests = mock.requests.lock().unwrap().clone();
    assert_eq!(requests.len(), 1, "tek üyeli konsey bir istek atmalı");
    let sys = requests[0].system_prompt.clone().unwrap_or_default();
    assert!(
        sys.contains("[Agent Role]") && sys.contains("Sen adil bir yargicsin."),
        "council.vote isteği persona taşımalı, got: {sys:?}"
    );
}

#[test]
fn m3_swarm_persona_format_matches_agent_call() {
    // Kural 7 kanıtı: aynı ajan için Agent.call ile swarm.run'ın kurduğu
    // sistem mesajı BİREBİR aynı olmalı (tek şerit — format sapması yok).
    let mock_direct = Arc::new(RecordingProvider::new());
    execute_script(
        r#"
        provider MockAI {
            type: "mock_provider"
        }

        agent Solist {
            provider: MockAI
            role: "Sen kidemli bir muhendissin."
            action calis() {
                return this.call({ prompt: "Task: Merhaba" });
            }
        }

        let r = Solist.calis()
        "#,
        mock_direct.clone(),
    );

    let mock_swarm = Arc::new(RecordingProvider::new());
    execute_script(
        r#"
        provider MockAI {
            type: "mock_provider"
        }

        agent Solist {
            provider: MockAI
            role: "Sen kidemli bir muhendissin."
        }

        swarm Tek {
            agents: ["Solist"]
            strategy: "sequential"
        }

        let r = Tek.run("Merhaba")
        "#,
        mock_swarm.clone(),
    );

    let direct_sys = mock_direct.requests.lock().unwrap()[0]
        .system_prompt
        .clone()
        .unwrap_or_default();
    let swarm_sys = mock_swarm.requests.lock().unwrap()[0]
        .system_prompt
        .clone()
        .unwrap_or_default();
    assert_eq!(
        direct_sys, swarm_sys,
        "Agent.call ile swarm.run aynı ajan için aynı sistem mesajını kurmalı"
    );
}
