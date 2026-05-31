//! Orchestration tests — adapted to current hudhudscript-orchestration API

use hudhudscript_orchestration::*;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use hudhudscript_orchestration::orchestration::types::{StepConfig, StepType, WorkflowStep};

fn create_test_engine() -> (OrchestrationEngine, Arc<LayerExecutor>, Arc<NetworkExecutor>) {
    let event_bus = Arc::new(EventBus::new());
    let layer_executor = Arc::new(LayerExecutor::new());
    let network_executor = Arc::new(NetworkExecutor::new(layer_executor.clone()));
    let council_executor = Arc::new(CouncilExecutor::new(event_bus.clone()));
    let engine = OrchestrationEngine::new(
        event_bus,
        layer_executor.clone(),
        network_executor.clone(),
        council_executor,
    );
    (engine, layer_executor, network_executor)
}


#[tokio::test]
async fn test_orchestration_engine_creation() {
    let (engine, _, _) = create_test_engine();
    // New engine has no workflows
    assert!(engine.get_workflow(Uuid::new_v4()).await.is_none());
}

#[tokio::test]
async fn test_workflow_creation() {
    let workflow = Workflow { id: Uuid::new_v4(), name: "test_workflow".to_string(), steps: vec![], config: WorkflowConfig::default() };

    assert_eq!(workflow.name, "test_workflow");
    assert_eq!(workflow.steps.len(), 0);
}

#[tokio::test]
async fn test_workflow_add_network_step() {
    let mut workflow = Workflow { id: Uuid::new_v4(), name: "test".to_string(), steps: vec![], config: WorkflowConfig::default() };
    let network_id = Uuid::new_v4();

    workflow.steps.push(WorkflowStep { name: "step".to_string(), step_type: StepType::Network { network_id }, config: StepConfig::default() });
    assert_eq!(workflow.steps.len(), 1);
}

#[tokio::test]
async fn test_register_layer() {
    let (_, le, _) = create_test_engine();
    let layer = Layer { id: Uuid::new_v4(), name: "test_layer".to_string(), agents: vec![], config: LayerConfig::default() };
    let layer_id = layer.id;

    le.register_layer(layer).await.unwrap();
    assert!(le.get_layer(layer_id).await.is_some());
}

#[tokio::test]
async fn test_register_network() {
    let (_, le, _) = create_test_engine();

    // Create and register a layer first
    let layer = Layer { id: Uuid::new_v4(), name: "layer1".to_string(), agents: vec![], config: LayerConfig::default() };
    let layer_id = layer.id;
    le.register_layer(layer).await.unwrap();

    // Create network referencing the registered layer
    let mut network = Network::new("test_network".to_string());
    network.add_layer(layer_id);
    assert!(network.layers.len() > 0);
}

#[tokio::test]
async fn test_register_workflow() {
    let (engine, le, ne) = create_test_engine();

    // Register a layer first
    let layer = Layer { id: Uuid::new_v4(), name: "layer1".to_string(), agents: vec![], config: LayerConfig::default() };
    let layer_id = layer.id;
    le.register_layer(layer).await.unwrap();

    // Register network
    let mut network = Network::new("network1".to_string());
    network.add_layer(layer_id);
    let network_id = network.id;
    ne.register_network(network).await.unwrap();

    // Register workflow with the network step
    let mut workflow = Workflow { id: Uuid::new_v4(), name: "workflow1".to_string(), steps: vec![], config: WorkflowConfig::default() };
    workflow.steps.push(WorkflowStep { name: "step".to_string(), step_type: StepType::Network { network_id }, config: StepConfig::default() });

    let result = engine.register_workflow(workflow).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_duplicate_workflow() {
    let (engine, _, _) = create_test_engine();

    let workflow1 = Workflow { id: Uuid::new_v4(), name: "test".to_string(), steps: vec![], config: WorkflowConfig::default() };
    let workflow2 = Workflow { id: Uuid::new_v4(), name: "test".to_string(), steps: vec![], config: WorkflowConfig::default() };

    engine.register_workflow(workflow1).await.unwrap();
    let result = engine.register_workflow(workflow2).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        OrchestrationError::WorkflowAlreadyExists(_)
    ));
}

#[tokio::test]
async fn test_get_workflow_by_name() {
    let (engine, _, _) = create_test_engine();

    let workflow = Workflow { id: Uuid::new_v4(), name: "test".to_string(), steps: vec![], config: WorkflowConfig::default() };
    engine.register_workflow(workflow).await.unwrap();

    let retrieved = engine.get_workflow_by_name("test").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test");
}

#[tokio::test]
async fn test_execute_workflow() {
    let (engine, le, ne) = create_test_engine();

    // Register layer
    let layer = Layer { id: Uuid::new_v4(), name: "layer1".to_string(), agents: vec!["agent1".to_string()], config: LayerConfig::default() };
    let layer_id = layer.id;
    le.register_layer(layer).await.unwrap();

    // Register network
    let mut network = Network::new("network1".to_string());
    network.add_layer(layer_id);
    let network_id = network.id;
    ne.register_network(network).await.unwrap();

    // Register workflow
    let mut workflow = Workflow { id: Uuid::new_v4(), name: "workflow1".to_string(), steps: vec![], config: WorkflowConfig::default() };
    workflow.steps.push(WorkflowStep { name: "step".to_string(), step_type: StepType::Network { network_id }, config: StepConfig::default() });
    let workflow_id = engine.register_workflow(workflow).await.unwrap();

    // Execute workflow
    let input = WorkflowInput {
        data: serde_json::json!({"test": "data"}),
        metadata: HashMap::new(),
    };

    let result = engine.execute_workflow(workflow_id, input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.step_results.len(), 1);
}

#[tokio::test]
async fn test_list_workflows() {
    let (engine, _, _) = create_test_engine();

    let workflow1 = Workflow { id: Uuid::new_v4(), name: "workflow1".to_string(), steps: vec![], config: WorkflowConfig::default() };
    let workflow2 = Workflow { id: Uuid::new_v4(), name: "workflow2".to_string(), steps: vec![], config: WorkflowConfig::default() };

    let id1 = engine.register_workflow(workflow1).await.unwrap();
    let id2 = engine.register_workflow(workflow2).await.unwrap();

    // Verify via get_workflow
    assert!(engine.get_workflow(id1).await.is_some());
    assert!(engine.get_workflow(id2).await.is_some());
}

#[tokio::test]
async fn test_list_layers() {
    let (_, le, _) = create_test_engine();

    let layer1 = Layer { id: Uuid::new_v4(), name: "layer1".to_string(), agents: vec![], config: LayerConfig::default() };
    let layer2 = Layer { id: Uuid::new_v4(), name: "layer2".to_string(), agents: vec![], config: LayerConfig::default() };

    let id1 = layer1.id;
    let id2 = layer2.id;

    le.register_layer(layer1).await.unwrap();
    le.register_layer(layer2).await.unwrap();

    assert!(le.get_layer(id1).await.is_some());
    assert!(le.get_layer(id2).await.is_some());
}

#[tokio::test]
async fn test_engine_default() {
    let (engine, _, _) = create_test_engine();
    // New engine: no workflows exist
    assert!(engine.get_workflow(Uuid::new_v4()).await.is_none());
}

#[tokio::test]
async fn test_get_workflow_nonexistent() {
    let (engine, _, _) = create_test_engine();
    assert!(engine.get_workflow(Uuid::new_v4()).await.is_none());
}

#[tokio::test]
async fn test_get_workflow_by_name_nonexistent() {
    let (engine, _, _) = create_test_engine();
    assert!(engine.get_workflow_by_name("nope").await.is_none());
}

#[tokio::test]
async fn test_execute_nonexistent_workflow() {
    let (engine, _, _) = create_test_engine();
    let input = WorkflowInput {
        data: serde_json::json!({}),
        metadata: HashMap::new(),
    };
    let result = engine.execute_workflow(Uuid::new_v4(), input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        OrchestrationError::WorkflowNotFound(_)
    ));
}

#[tokio::test]
async fn test_workflow_config_default() {
    let config = WorkflowConfig::default();
    assert!(!config.monitoring);
    assert_eq!(config.default_timeout, 300);
    assert_eq!(config.max_concurrent, 4);
}

#[tokio::test]
async fn test_workflow_validation() {
    let (engine, _, _) = create_test_engine();

    // Create workflow referencing a non-existent network
    let mut workflow = Workflow { id: Uuid::new_v4(), name: "invalid".to_string(), steps: vec![], config: WorkflowConfig::default() };
    workflow.steps.push(WorkflowStep { name: "step".to_string(), step_type: StepType::Network { network_id: Uuid::new_v4() }, config: StepConfig::default() });

    // Registration succeeds (network existence checked at execution time)
    let wfid = engine.register_workflow(workflow).await.unwrap();

    // Execution should fail with NetworkNotFound
    let input = WorkflowInput {
        data: serde_json::json!({}),
        metadata: HashMap::new(),
    };
    // execute_workflow returns Ok even when steps fail (failure captured in step_results)
    let result = engine.execute_workflow(wfid, input).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.success);
    assert_eq!(output.step_results.len(), 1);
    assert!(!output.step_results[0].success);
}

#[tokio::test]
async fn test_orchestration_error_display_all_variants() {
    let e1 = OrchestrationError::LayerError(LayerError::ExecutionFailed("layer broke".to_string()));
    assert!(format!("{}", e1).contains("Layer error"));
    assert!(format!("{}", e1).contains("layer broke"));

    let e2 = OrchestrationError::WorkflowAlreadyExists("dup".to_string());
    assert!(format!("{}", e2).contains("Workflow already exists: dup"));

    let wid = Uuid::new_v4();
    let e3 = OrchestrationError::WorkflowNotFound(wid);
    assert!(format!("{}", e3).contains("Workflow not found"));

    let nid = Uuid::new_v4();
    let e4 = OrchestrationError::NetworkNotFound(nid);
    assert!(format!("{}", e4).contains("Network not found"));

    let e5 = OrchestrationError::NetworkExecutionFailed("timeout".to_string());
    let s5 = format!("{}", e5);
    assert!(s5.contains("Network execution failed"));
    assert!(s5.contains("timeout"));

    let e6 = OrchestrationError::InvalidWorkflow("bad".to_string());
    assert!(format!("{}", e6).contains("Invalid workflow: bad"));

    let e7 = OrchestrationError::WorkflowTimedOut(wid);
    assert!(format!("{}", e7).contains("timed out"));
}

#[tokio::test]
async fn test_get_workflow_by_id() {
    let (engine, _, _) = create_test_engine();
    let workflow = Workflow { id: Uuid::new_v4(), name: "findme".to_string(), steps: vec![], config: WorkflowConfig::default() };
    let wid = engine.register_workflow(workflow).await.unwrap();
    let found = engine.get_workflow(wid).await;
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "findme");
}
