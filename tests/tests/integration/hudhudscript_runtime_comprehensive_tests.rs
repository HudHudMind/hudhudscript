//! Comprehensive tests for Agent Runtime
//! Target: 100% code coverage

use hudhudscript_mcp::client::McpClient;
use hudhudscript_mcp::transport::TransportConfig;
use hudhudscript_resources::ResourceManager;
use hudhudscript_runtime::{
    Agent, AgentRuntime, AgentState, ExecutionRecord, ExecutionStatus, StateValue, Task,
    TaskImplementation, TaskParameter,
};
use hudhudscript_tools::ToolRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

async fn create_test_runtime() -> AgentRuntime {
    let mcp_client = Arc::new(
        McpClient::new(TransportConfig::stdio("echo", vec![]))
            .await
            .unwrap(),
    );

    let tool_registry = Arc::new(ToolRegistry::new());
    let resource_manager = Arc::new(ResourceManager::new(
        mcp_client.clone(),
        Duration::from_secs(300),
    ));

    AgentRuntime::new(tool_registry, resource_manager, mcp_client)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_creation() {
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    assert_eq!(agent.id, "agent-1");
    assert_eq!(agent.name, "TestAgent");
    assert_eq!(agent.tasks.len(), 0);
    assert_eq!(agent.tools.len(), 0);
    assert_eq!(agent.resources.len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_add_task() {
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());
    let task = Task::new("task-1".to_string(), "TestTask".to_string());

    agent.add_task(task);

    assert_eq!(agent.tasks.len(), 1);
    assert!(agent.get_task("task-1").is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_add_tool() {
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    agent.add_tool("tool1".to_string());
    agent.add_tool("tool2".to_string());
    agent.add_tool("tool1".to_string()); // Duplicate

    assert_eq!(agent.tools.len(), 2);
    assert!(agent.tools.contains(&"tool1".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_add_resource() {
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    agent.add_resource("resource1".to_string());
    agent.add_resource("resource2".to_string());
    agent.add_resource("resource1".to_string()); // Duplicate

    assert_eq!(agent.resources.len(), 2);
    assert!(agent.resources.contains(&"resource1".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_state_creation() {
    let state = AgentState::new("agent-1".to_string());

    assert_eq!(state.agent_id, "agent-1");
    assert_eq!(state.variables.len(), 0);
    assert_eq!(state.version, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_state_set_get() {
    let mut state = AgentState::new("agent-1".to_string());

    state.set("counter".to_string(), StateValue::Number(42.0));

    assert_eq!(state.version, 1);
    assert!(state.get("counter").is_some());

    if let Some(StateValue::Number(n)) = state.get("counter") {
        assert_eq!(*n, 42.0);
    } else {
        panic!("Expected Number value");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_state_remove() {
    let mut state = AgentState::new("agent-1".to_string());

    state.set("key".to_string(), StateValue::String("value".to_string()));
    assert_eq!(state.variables.len(), 1);

    let removed = state.remove("key");
    assert!(removed.is_some());
    assert_eq!(state.variables.len(), 0);
    assert_eq!(state.version, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_state_clear() {
    let mut state = AgentState::new("agent-1".to_string());

    state.set("key1".to_string(), StateValue::Number(1.0));
    state.set("key2".to_string(), StateValue::Number(2.0));

    state.clear();

    assert_eq!(state.variables.len(), 0);
    assert_eq!(state.version, 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_state_value_variants() {
    let string_val = StateValue::String("test".to_string());
    let number_val = StateValue::Number(42.0);
    let bool_val = StateValue::Boolean(true);
    let null_val = StateValue::Null;
    let array_val = StateValue::Array(vec![StateValue::Number(1.0)]);
    let mut map = HashMap::new();
    map.insert("key".to_string(), StateValue::String("value".to_string()));
    let object_val = StateValue::Object(map);

    assert!(matches!(string_val, StateValue::String(_)));
    assert!(matches!(number_val, StateValue::Number(_)));
    assert!(matches!(bool_val, StateValue::Boolean(_)));
    assert!(matches!(null_val, StateValue::Null));
    assert!(matches!(array_val, StateValue::Array(_)));
    assert!(matches!(object_val, StateValue::Object(_)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_execution_record_creation() {
    let record = ExecutionRecord::new("agent-1".to_string(), "task-1".to_string(), HashMap::new());

    assert_eq!(record.agent_id, "agent-1");
    assert_eq!(record.task_id, "task-1");
    assert_eq!(record.status, ExecutionStatus::Running);
    assert!(record.output.is_none());
    assert!(record.error.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_execution_record_complete() {
    let mut record =
        ExecutionRecord::new("agent-1".to_string(), "task-1".to_string(), HashMap::new());

    record.complete(StateValue::String("success".to_string()));

    assert_eq!(record.status, ExecutionStatus::Completed);
    assert!(record.output.is_some());
    assert!(record.ended_at.is_some());
    assert!(record.duration_ms.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_execution_record_fail() {
    let mut record =
        ExecutionRecord::new("agent-1".to_string(), "task-1".to_string(), HashMap::new());

    record.fail("Error message".to_string());

    assert_eq!(record.status, ExecutionStatus::Failed);
    assert_eq!(record.error, Some("Error message".to_string()));
    assert!(record.ended_at.is_some());
    assert!(record.duration_ms.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_metadata_record_execution() {
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    agent.metadata.record_execution(true);
    agent.metadata.record_execution(true);
    agent.metadata.record_execution(false);

    assert_eq!(agent.metadata.execution_count, 3);
    assert_eq!(agent.metadata.success_count, 2);
    assert_eq!(agent.metadata.failure_count, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_metadata_success_rate() {
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    assert_eq!(agent.metadata.success_rate(), 0.0);

    agent.metadata.record_execution(true);
    agent.metadata.record_execution(true);
    agent.metadata.record_execution(false);

    assert_eq!(agent.metadata.success_rate(), 2.0 / 3.0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_register_agent() {
    let runtime = create_test_runtime().await;
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    let result = runtime.register_agent(agent).await;
    assert!(result.is_ok());

    let retrieved = runtime.get_agent("agent-1").await;
    assert!(retrieved.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_register_duplicate_agent() {
    let runtime = create_test_runtime().await;
    let agent1 = Agent::new("agent-1".to_string(), "TestAgent1".to_string());
    let agent2 = Agent::new("agent-1".to_string(), "TestAgent2".to_string());

    runtime.register_agent(agent1).await.unwrap();
    let result = runtime.register_agent(agent2).await;

    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_unregister_agent() {
    let runtime = create_test_runtime().await;
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    runtime.register_agent(agent).await.unwrap();
    let result = runtime.unregister_agent("agent-1").await;

    assert!(result.is_ok());
    assert!(runtime.get_agent("agent-1").await.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_list_agents() {
    let runtime = create_test_runtime().await;

    for i in 0..3 {
        let agent = Agent::new(format!("agent-{}", i), format!("Agent{}", i));
        runtime.register_agent(agent).await.unwrap();
    }

    let agents = runtime.list_agents().await;
    assert_eq!(agents.len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_get_state() {
    let runtime = create_test_runtime().await;
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    runtime.register_agent(agent).await.unwrap();

    let state = runtime.get_state("agent-1").await;
    assert!(state.is_some());
    assert_eq!(state.unwrap().agent_id, "agent-1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_update_state() {
    let runtime = create_test_runtime().await;
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    runtime.register_agent(agent).await.unwrap();

    let result = runtime
        .update_state("agent-1", |state| {
            state.set("counter".to_string(), StateValue::Number(42.0));
        })
        .await;

    assert!(result.is_ok());

    let state = runtime.get_state("agent-1").await.unwrap();
    assert!(state.get("counter").is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_execute_task() {
    let runtime = create_test_runtime().await;
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());
    let task = Task::new("task-1".to_string(), "TestTask".to_string());

    agent.add_task(task);
    runtime.register_agent(agent).await.unwrap();

    let result = runtime
        .execute_task("agent-1", "task-1", HashMap::new())
        .await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_execution_history() {
    let runtime = create_test_runtime().await;
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());
    let task = Task::new("task-1".to_string(), "TestTask".to_string());

    agent.add_task(task);
    runtime.register_agent(agent).await.unwrap();

    runtime
        .execute_task("agent-1", "task-1", HashMap::new())
        .await
        .unwrap();

    let history = runtime.get_execution_history("agent-1").await;
    assert_eq!(history.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_statistics() {
    let runtime = create_test_runtime().await;
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());
    let task = Task::new("task-1".to_string(), "TestTask".to_string());

    agent.add_task(task);
    runtime.register_agent(agent).await.unwrap();

    runtime
        .execute_task("agent-1", "task-1", HashMap::new())
        .await
        .unwrap();

    let stats = runtime.get_statistics().await;
    assert_eq!(stats.total_agents, 1);
    assert_eq!(stats.total_executions, 1);
    assert_eq!(stats.successful_executions, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_runtime_clear_history() {
    let runtime = create_test_runtime().await;
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());
    let task = Task::new("task-1".to_string(), "TestTask".to_string());

    agent.add_task(task);
    runtime.register_agent(agent).await.unwrap();

    runtime
        .execute_task("agent-1", "task-1", HashMap::new())
        .await
        .unwrap();
    runtime.clear_execution_history().await;

    let history = runtime.get_all_executions().await;
    assert_eq!(history.len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_task_creation() {
    let task = Task::new("task-1".to_string(), "TestTask".to_string());

    assert_eq!(task.id, "task-1");
    assert_eq!(task.name, "TestTask");
    assert_eq!(task.parameters.len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_task_add_parameter() {
    let mut task = Task::new("task-1".to_string(), "TestTask".to_string());

    let param = TaskParameter {
        name: "input".to_string(),
        param_type: Some("string".to_string()),
        optional: false,
        default: None,
    };

    task.add_parameter(param);
    assert_eq!(task.parameters.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_task_implementation_variants() {
    let native = TaskImplementation::Native;
    let bytecode = TaskImplementation::Bytecode(vec![1, 2, 3]);
    let ast = TaskImplementation::Ast("ast_ref".to_string());

    assert!(matches!(native, TaskImplementation::Native));
    assert!(matches!(bytecode, TaskImplementation::Bytecode(_)));
    assert!(matches!(ast, TaskImplementation::Ast(_)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_execution_status_variants() {
    assert_eq!(ExecutionStatus::Running, ExecutionStatus::Running);
    assert_ne!(ExecutionStatus::Running, ExecutionStatus::Completed);
    assert_ne!(ExecutionStatus::Completed, ExecutionStatus::Failed);
    assert_ne!(ExecutionStatus::Failed, ExecutionStatus::Cancelled);
}
