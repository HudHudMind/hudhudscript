use hudhudscript_runtime::agent::{
    Agent, AgentMetadata, AgentState, ExecutionRecord, ExecutionStatus, StateValue, Task,
};
use std::collections::HashMap;

#[test]
fn test_agent_creation() {
    let agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());
    assert_eq!(agent.id, "agent-1");
    assert_eq!(agent.name, "TestAgent");
    assert_eq!(agent.tasks.len(), 0);
}

#[test]
fn test_agent_add_task() {
    let mut agent = Agent::new("agent-1".to_string(), "TestAgent".to_string());
    let task = Task::new("task-1".to_string(), "TestTask".to_string());

    agent.add_task(task);
    assert_eq!(agent.tasks.len(), 1);
    assert!(agent.get_task("task-1").is_some());
}

#[test]
fn test_agent_state() {
    let mut state = AgentState::new("agent-1".to_string());

    state.set("counter".to_string(), StateValue::Number(42.0));
    assert!(state.get("counter").is_some());

    state.set("name".to_string(), StateValue::String("test".to_string()));
    assert_eq!(state.variables.len(), 2);
    assert_eq!(state.version, 2);
}

#[test]
fn test_agent_metadata() {
    let mut metadata = AgentMetadata::new();

    metadata.record_execution(true);
    metadata.record_execution(true);
    metadata.record_execution(false);

    assert_eq!(metadata.execution_count, 3);
    assert_eq!(metadata.success_count, 2);
    assert_eq!(metadata.failure_count, 1);
    assert_eq!(metadata.success_rate(), 2.0 / 3.0);
}

#[test]
fn test_execution_record() {
    let mut record =
        ExecutionRecord::new("agent-1".to_string(), "task-1".to_string(), HashMap::new());

    assert_eq!(record.status, ExecutionStatus::Running);

    record.complete(StateValue::String("success".to_string()));
    assert_eq!(record.status, ExecutionStatus::Completed);
    assert!(record.output.is_some());
    assert!(record.duration_ms.is_some());
}
