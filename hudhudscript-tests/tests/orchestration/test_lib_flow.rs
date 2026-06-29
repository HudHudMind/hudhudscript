//! Full orchestration flow test — adapted to current API

use hudhudscript_orchestration::*;
use uuid::Uuid;

use hudhudscript_orchestration::orchestration::types::{StepConfig, StepType, WorkflowStep};
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_engine() -> (
    OrchestrationEngine,
    Arc<LayerExecutor>,
    Arc<NetworkExecutor>,
) {
    let event_bus = std::sync::Arc::new(EventBus::new());
    let layer_executor = std::sync::Arc::new(LayerExecutor::new());
    let network_executor = std::sync::Arc::new(NetworkExecutor::new(layer_executor.clone()));
    let council_executor = std::sync::Arc::new(CouncilExecutor::new(event_bus.clone()));
    let engine = OrchestrationEngine::new(
        event_bus,
        layer_executor.clone(),
        network_executor.clone(),
        council_executor,
    );
    (engine, layer_executor, network_executor)
}

#[tokio::test]
async fn test_full_orchestration_flow() {
    let (engine, le, ne) = create_test_engine();

    // Create and register layers
    let layer1 = Layer {
        id: Uuid::new_v4(),
        name: "input_layer".to_string(),
        agents: vec!["agent1".to_string()],
        config: LayerConfig::default(),
    };
    let layer2 = Layer {
        id: Uuid::new_v4(),
        name: "processing_layer".to_string(),
        agents: vec!["agent2".to_string()],
        config: LayerConfig::default(),
    };
    let layer3 = Layer {
        id: Uuid::new_v4(),
        name: "output_layer".to_string(),
        agents: vec!["agent3".to_string()],
        config: LayerConfig::default(),
    };

    let layer1_id = layer1.id;
    let layer2_id = layer2.id;
    let layer3_id = layer3.id;

    le.register_layer(layer1).await.unwrap();
    le.register_layer(layer2).await.unwrap();
    le.register_layer(layer3).await.unwrap();

    // Create network with sequential layers
    let mut network = Network::new("pipeline".to_string());
    network.add_layer(layer1_id);
    network.add_layer(layer2_id);
    network.add_layer(layer3_id);
    network.add_edge(layer1_id, layer2_id);
    network.add_edge(layer2_id, layer3_id);
    let network_id = network.id;
    ne.register_network(network).await.unwrap();

    // Create and register workflow
    let mut workflow = Workflow {
        id: Uuid::new_v4(),
        name: "main_workflow".to_string(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    workflow.steps.push(WorkflowStep {
        name: "step".to_string(),
        step_type: StepType::Network { network_id },
        config: StepConfig::default(),
    });
    let workflow_id = engine.register_workflow(workflow).await.unwrap();

    // Execute workflow
    let input = WorkflowInput {
        data: serde_json::json!({"message": "test"}),
        metadata: HashMap::new(),
    };

    let result = engine.execute_workflow(workflow_id, input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.step_results.len(), 1);
}
