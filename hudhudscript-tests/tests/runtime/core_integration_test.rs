//! v0.2 Runtime Core Integration Tests
//!
//! Covers: provider mock tests, task execution, state persistence,
//! strategy execution, and CI-ready assertions.

use hudhudscript_runtime::{
    agent::{Agent, AgentState, StateValue, Task, TaskImplementation},
    persistence::FilePersistence,
    provider::{
        LLMRequest, LLMResponse, Provider, ProviderError, ProviderInfo, ProviderRegistry,
        ProviderType, TokenUsage, TokenUsageStats,
    },
};
use std::sync::Arc;
use tempfile::tempdir;

// ── Mock Provider ─────────────────────────────────────────────────────────────

struct MockProvider {
    name: String,
    model: String,
    should_fail: bool,
}

impl MockProvider {
    fn new(name: &str, model: &str) -> Self {
        Self {
            name: name.to_string(),
            model: model.to_string(),
            should_fail: false,
        }
    }

    fn failing(name: &str) -> Self {
        Self {
            name: name.to_string(),
            model: "none".to_string(),
            should_fail: true,
        }
    }
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::ApiError("mock failure".to_string()));
        }
        Ok(LLMResponse {
            content: format!("mock response to: {}", request.prompt),
            tokens_used: TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 10,
                total_tokens: 15,
            },
            model: self.model.clone(),
            finish_reason: "stop".to_string(),
            tool_calls: None,
        })
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

// ── Provider Mock Tests ───────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mock_provider_call() {
    let provider = MockProvider::new("test", "gpt-mock");
    let request = LLMRequest {
        prompt: "hello".to_string(),
        system_prompt: None,
        temperature: None,
        max_tokens: None,
        mnemonics: None,
        optimize: false,
        tools: None,
    };
    let response = provider.call(request).await.unwrap();
    assert!(response.content.contains("hello"));
    assert_eq!(response.tokens_used.total_tokens, 15);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mock_provider_failure() {
    let provider = MockProvider::failing("bad");
    let request = LLMRequest {
        prompt: "test".to_string(),
        system_prompt: None,
        temperature: None,
        max_tokens: None,
        mnemonics: None,
        optimize: false,
        tools: None,
    };
    let result = provider.call(request).await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_registry_register_and_get() {
    let registry = ProviderRegistry::new();
    let provider = Arc::new(MockProvider::new("openai", "gpt-4")) as Arc<dyn Provider>;
    registry.register("openai".to_string(), provider).await;

    assert!(registry.exists("openai").await);
    assert!(!registry.exists("anthropic").await);

    let retrieved = registry.get("openai").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().info().model, "gpt-4");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_registry_list() {
    let registry = ProviderRegistry::new();
    registry
        .register(
            "p1".to_string(),
            Arc::new(MockProvider::new("p1", "m1")) as Arc<dyn Provider>,
        )
        .await;
    registry
        .register(
            "p2".to_string(),
            Arc::new(MockProvider::new("p2", "m2")) as Arc<dyn Provider>,
        )
        .await;

    let list = registry.list().await;
    assert_eq!(list.len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_registry_unregister() {
    let registry = ProviderRegistry::new();
    registry
        .register(
            "tmp".to_string(),
            Arc::new(MockProvider::new("tmp", "m")) as Arc<dyn Provider>,
        )
        .await;
    assert!(registry.exists("tmp").await);

    registry.unregister("tmp").await;
    assert!(!registry.exists("tmp").await);
}

// ── Task Execution Tests ──────────────────────────────────────────────────────
//
// AgentRuntime construction requires an MCP server connection, so these tests
// exercise the agent/state/task APIs directly rather than going through
// the full runtime.

#[test]
fn test_task_native_execution_marker() {
    // Verify TaskImplementation::Native is the default
    let task = Task::new("t1".to_string(), "MyTask".to_string());
    assert!(matches!(task.implementation, TaskImplementation::Native));
}

#[test]
fn test_task_ast_implementation() {
    let mut task = Task::new("t2".to_string(), "AstTask".to_string());
    task.implementation = TaskImplementation::Ast("return 42;".to_string());
    assert!(matches!(task.implementation, TaskImplementation::Ast(_)));
}

// ── State Persistence Tests ───────────────────────────────────────────────────

#[test]
fn test_state_persistence_roundtrip() {
    let dir = tempdir().unwrap();
    let persistence = FilePersistence::new(dir.path()).unwrap();

    let mut state = AgentState::new("agent-persist".to_string());
    state.set("counter".to_string(), StateValue::Number(99.0));
    state.set("label".to_string(), StateValue::String("hello".to_string()));
    state.set("flag".to_string(), StateValue::Boolean(true));
    state.set("nothing".to_string(), StateValue::Null);
    state.set(
        "items".to_string(),
        StateValue::Array(vec![StateValue::Number(1.0), StateValue::Number(2.0)]),
    );

    persistence.save(&state).unwrap();
    let loaded = persistence.load("agent-persist").unwrap();

    assert_eq!(loaded.agent_id, "agent-persist");
    assert_eq!(loaded.version, state.version);

    match loaded.get("counter") {
        Some(StateValue::Number(n)) => assert_eq!(*n, 99.0),
        _ => panic!("counter mismatch"),
    }
    match loaded.get("label") {
        Some(StateValue::String(s)) => assert_eq!(s, "hello"),
        _ => panic!("label mismatch"),
    }
    match loaded.get("flag") {
        Some(StateValue::Boolean(b)) => assert!(*b),
        _ => panic!("flag mismatch"),
    }
    match loaded.get("nothing") {
        Some(StateValue::Null) => {}
        _ => panic!("null mismatch"),
    }
    match loaded.get("items") {
        Some(StateValue::Array(arr)) => assert_eq!(arr.len(), 2),
        _ => panic!("items mismatch"),
    }
}

#[test]
fn test_state_persistence_not_found() {
    let dir = tempdir().unwrap();
    let persistence = FilePersistence::new(dir.path()).unwrap();
    assert!(persistence.load("ghost-agent").is_err());
}

#[test]
fn test_state_persistence_overwrite() {
    let dir = tempdir().unwrap();
    let persistence = FilePersistence::new(dir.path()).unwrap();

    let mut state = AgentState::new("agent-ow".to_string());
    state.set("v".to_string(), StateValue::Number(1.0));
    persistence.save(&state).unwrap();

    state.set("v".to_string(), StateValue::Number(2.0));
    persistence.save(&state).unwrap();

    let loaded = persistence.load("agent-ow").unwrap();
    match loaded.get("v") {
        Some(StateValue::Number(n)) => assert_eq!(*n, 2.0),
        _ => panic!("overwrite failed"),
    }
}

#[test]
fn test_state_persistence_delete() {
    let dir = tempdir().unwrap();
    let persistence = FilePersistence::new(dir.path()).unwrap();

    let state = AgentState::new("agent-del".to_string());
    persistence.save(&state).unwrap();
    assert!(persistence.exists("agent-del"));

    persistence.delete("agent-del").unwrap();
    assert!(!persistence.exists("agent-del"));
}

// ── Strategy Execution Tests ──────────────────────────────────────────────────

#[test]
fn test_strategy_permissions_stored_in_state() {
    // Verify that strategy permissions can be stored as StateValue
    let mut state = AgentState::new("strategy-agent".to_string());
    let permissions = StateValue::Array(vec![
        StateValue::String("read".to_string()),
        StateValue::String("write".to_string()),
    ]);
    state.set("permissions".to_string(), permissions);

    match state.get("permissions") {
        Some(StateValue::Array(perms)) => assert_eq!(perms.len(), 2),
        _ => panic!("permissions not stored correctly"),
    }
}

#[test]
fn test_strategy_realm_stored_in_state() {
    let mut state = AgentState::new("realm-agent".to_string());
    state.set(
        "realm".to_string(),
        StateValue::String("production".to_string()),
    );

    match state.get("realm") {
        Some(StateValue::String(r)) => assert_eq!(r, "production"),
        _ => panic!("realm not stored correctly"),
    }
}

// ── Agent State Tests ─────────────────────────────────────────────────────────

#[test]
fn test_agent_state_versioning() {
    let mut state = AgentState::new("v-agent".to_string());
    assert_eq!(state.version, 0);

    state.set("a".to_string(), StateValue::Number(1.0));
    assert_eq!(state.version, 1);

    state.set("b".to_string(), StateValue::Number(2.0));
    assert_eq!(state.version, 2);

    state.remove("a");
    assert_eq!(state.version, 3);
}

#[test]
fn test_agent_state_clear() {
    let mut state = AgentState::new("clear-agent".to_string());
    state.set("x".to_string(), StateValue::Number(1.0));
    state.set("y".to_string(), StateValue::Number(2.0));
    assert_eq!(state.variables.len(), 2);

    state.clear();
    assert_eq!(state.variables.len(), 0);
}

#[test]
fn test_agent_add_and_get_task() {
    let mut agent = Agent::new("a1".to_string(), "TestAgent".to_string());
    let task = Task::new("t1".to_string(), "Greet".to_string());
    agent.add_task(task);

    assert!(agent.get_task("t1").is_some());
    assert!(agent.get_task("nonexistent").is_none());
}
