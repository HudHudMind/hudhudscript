//! End-to-end orchestration tests — workflow registration and lookup.

use hudhudscript_orchestration::orchestration::{StepConfig, StepType, WorkflowStep};
use hudhudscript_orchestration::*;
use std::sync::Arc;
use uuid::Uuid;

fn create_test_engine() -> OrchestrationEngine {
    let event_bus = std::sync::Arc::new(EventBus::new());
    let layer_executor = std::sync::Arc::new(LayerExecutor::new());
    let network_executor = std::sync::Arc::new(NetworkExecutor::new(layer_executor.clone()));
    let council_executor = std::sync::Arc::new(CouncilExecutor::new(event_bus.clone()));
    OrchestrationEngine::new(
        event_bus,
        layer_executor,
        network_executor,
        council_executor,
    )
}

#[tokio::test]
async fn test_orchestration_engine_creation() {
    let _engine = create_test_engine();
    // Engine creates successfully with all executors wired.
}

#[tokio::test]
async fn test_register_multiple_workflows() {
    let engine = create_test_engine();
    for i in 0..4 {
        let workflow = Workflow {
            id: Uuid::new_v4(),
            name: format!("workflow_{}", i),
            steps: vec![],
            config: WorkflowConfig::default(),
        };
        engine.register_workflow(workflow).await.unwrap();
    }
    let wf = engine.get_workflow_by_name("workflow_3").await;
    assert!(wf.is_some());
}

#[tokio::test]
async fn test_get_workflow_by_name() {
    let engine = create_test_engine();
    let workflow = Workflow {
        id: Uuid::new_v4(),
        name: "my_workflow".to_string(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    engine.register_workflow(workflow).await.unwrap();
    let retrieved = engine.get_workflow_by_name("my_workflow").await;
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_get_nonexistent_workflow() {
    let engine = create_test_engine();
    let retrieved = engine.get_workflow_by_name("nonexistent").await;
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_concurrent_workflow_registration() {
    let engine = Arc::new(create_test_engine());
    let mut handles = vec![];
    for i in 0..10 {
        let eng = Arc::clone(&engine);
        handles.push(tokio::spawn(async move {
            let workflow = Workflow {
                id: Uuid::new_v4(),
                name: format!("workflow_{}", i),
                steps: vec![],
                config: WorkflowConfig::default(),
            };
            eng.register_workflow(workflow).await
        }));
    }
    for h in handles {
        h.await.unwrap().unwrap();
    }
    let wf = engine.get_workflow_by_name("workflow_9").await;
    assert!(wf.is_some());
}

#[tokio::test]
async fn test_empty_workflow_registration() {
    let engine = create_test_engine();
    let workflow = Workflow {
        id: Uuid::new_v4(),
        name: "empty_workflow".to_string(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    let result = engine.register_workflow(workflow).await;
    assert!(result.is_ok());
}
