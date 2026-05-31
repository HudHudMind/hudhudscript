use hudhudscript_mcp::transport::TransportConfig;
use hudhudscript_mcp::McpClient;
use hudhudscript_runtime::agent::{Agent, StateValue, Task};
use hudhudscript_runtime::runtime::AgentRuntime;
use hudhudscript_runtime::runtime::RuntimeError;
use std::collections::HashMap;
use std::sync::Arc;

// Note: The tool registry and resource manager types — import from their respective modules.
// We reference AgentRuntime::new() which requires McpClient, ToolRegistry, ResourceManager.

async fn create_test_runtime() -> AgentRuntime {
    let mcp_client = Arc::new(
        McpClient::new(TransportConfig::stdio("echo", vec![]))
            .await
            .unwrap(),
    );

    let tool_registry = Arc::new(hudhudscript_tools::ToolRegistry::new());
    let resource_manager = Arc::new(hudhudscript_resources::ResourceManager::new(
        mcp_client.clone(),
        std::time::Duration::from_secs(300),
    ));

    AgentRuntime::new(tool_registry, resource_manager, mcp_client)
}

#[tokio::test]
async fn test_register_agent() {
    let runtime = create_test_runtime().await;
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    let result = runtime.register_agent(agent).await;
    assert!(result.is_ok());

    let retrieved = runtime.get_agent("agent-1").await;
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_duplicate_agent() {
    let runtime = create_test_runtime().await;
    let agent1 = Agent::new("agent-1".to_string(), "TestAgent1".to_string());
    let agent2 = Agent::new("agent-1".to_string(), "TestAgent2".to_string());

    runtime.register_agent(agent1).await.unwrap();
    let result = runtime.register_agent(agent2).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_unregister_agent() {
    let runtime = create_test_runtime().await;
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    runtime.register_agent(agent).await.unwrap();
    let result = runtime.unregister_agent("agent-1").await;

    assert!(result.is_ok());
    assert!(runtime.get_agent("agent-1").await.is_none());
}

#[tokio::test]
async fn test_agent_state() {
    let runtime = create_test_runtime().await;
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());

    runtime.register_agent(agent).await.unwrap();

    runtime
        .update_state("agent-1", |state| {
            state.set("counter".to_string(), StateValue::Number(42.0));
        })
        .await
        .unwrap();

    let state = runtime.get_state("agent-1").await.unwrap();
    assert!(state.get("counter").is_some());
}

#[tokio::test]
async fn test_execute_task() {
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

#[tokio::test]
async fn test_execution_history() {
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

#[tokio::test]
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
