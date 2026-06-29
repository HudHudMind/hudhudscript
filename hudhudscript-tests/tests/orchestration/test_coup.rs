//! Coup tests — adapted to current hudhudscript-orchestration API

use hudhudscript_orchestration::*;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_layer_creation() {
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "test_layer".to_string(),
        agents: vec!["agent1".to_string(), "agent2".to_string()],
        config: LayerConfig::default(),
    };

    assert_eq!(layer.name, "test_layer");
    assert_eq!(layer.agents.len(), 2);
    assert!(layer.config.dependencies.is_empty());
}

#[test]
fn test_layer_add_dependency() {
    let mut layer = Layer::new("test");
    let dep_id = Uuid::new_v4();

    layer.config.dependencies.push(dep_id);
    assert!(!layer.config.dependencies.is_empty());
    assert_eq!(layer.config.dependencies.len(), 1);

    // Push same again — Vec semantics, no dedup
    layer.config.dependencies.push(dep_id);
    assert_eq!(layer.config.dependencies.len(), 2);
}

#[test]
fn test_layer_config_default() {
    let config = LayerConfig::default();

    assert_eq!(config.mode, ExecutionMode::Parallel);
    assert_eq!(config.timeout, 300);
    assert_eq!(config.max_retries, 0);
    assert_eq!(config.failure_strategy, FailureStrategy::Stop);
}

#[test]
fn test_execution_mode() {
    assert_eq!(ExecutionMode::Parallel, ExecutionMode::Parallel);
    assert_ne!(ExecutionMode::Parallel, ExecutionMode::Sequential);
}

#[test]
fn test_failure_strategy() {
    assert_eq!(FailureStrategy::Stop, FailureStrategy::Stop);
    assert_ne!(FailureStrategy::Stop, FailureStrategy::Continue);
    assert_ne!(FailureStrategy::Continue, FailureStrategy::Retry);
}

#[tokio::test]
async fn test_layer_executor_creation() {
    let executor = LayerExecutor::new();
    assert!(executor.get_layer(Uuid::new_v4()).await.is_none());
}

#[tokio::test]
async fn test_register_layer() {
    let executor = LayerExecutor::new();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "test".to_string(),
        agents: vec!["agent1".to_string()],
        config: LayerConfig::default(),
    };
    let layer_id = layer.id;

    let result = executor.register_layer(layer).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), layer_id);

    let retrieved = executor.get_layer(layer_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test");
}

#[tokio::test]
async fn test_register_duplicate_layer() {
    let executor = LayerExecutor::new();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "dup".to_string(),
        agents: vec![],
        config: LayerConfig::default(),
    };

    executor.register_layer(layer.clone()).await.unwrap();
    let result = executor.register_layer(layer).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LayerError::LayerAlreadyExists(_)
    ));
}

#[tokio::test]
async fn test_get_layer_by_id() {
    let executor = LayerExecutor::new();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "findable".to_string(),
        agents: vec![],
        config: LayerConfig::default(),
    };
    let layer_id = layer.id;

    executor.register_layer(layer).await.unwrap();

    let retrieved = executor.get_layer(layer_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "findable");
}

#[tokio::test]
async fn test_execute_layer_parallel() {
    let executor = LayerExecutor::new();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "test".to_string(),
        agents: vec!["agent1".to_string(), "agent2".to_string()],
        config: LayerConfig::default(),
    };
    let layer_id = layer.id;

    executor.register_layer(layer).await.unwrap();

    let input = LayerInput {
        data: serde_json::json!({"test": "data"}),
        metadata: HashMap::new(),
    };

    let output = executor.execute_layer(layer_id, input).await.unwrap();
    assert!(output.agent_results.iter().all(|r| r.success));
    assert_eq!(output.agent_results.len(), 2);
}

#[tokio::test]
async fn test_validate_dependencies() {
    let executor = LayerExecutor::new();

    let layer1 = Layer {
        id: Uuid::new_v4(),
        name: "layer1".to_string(),
        agents: vec![],
        config: LayerConfig::default(),
    };
    let layer1_id = layer1.id;
    executor.register_layer(layer1).await.unwrap();

    let mut layer2 = Layer {
        id: Uuid::new_v4(),
        name: "layer2".to_string(),
        agents: vec![],
        config: LayerConfig::default(),
    };
    layer2.config.dependencies.push(layer1_id);

    let result = executor.register_layer(layer2).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_missing_dependency() {
    let executor = LayerExecutor::new();

    let mut layer = Layer {
        id: Uuid::new_v4(),
        name: "test".to_string(),
        agents: vec![],
        config: LayerConfig::default(),
    };
    layer.config.dependencies.push(Uuid::new_v4());

    let result = executor.register_layer(layer).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LayerError::DependencyNotFound(_)
    ));
}

#[tokio::test]
async fn test_execute_layer_not_found() {
    let executor = LayerExecutor::new();
    let input = LayerInput {
        data: serde_json::json!({}),
        metadata: HashMap::new(),
    };
    let result = executor.execute_layer(Uuid::new_v4(), input).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LayerError::LayerNotFound(_)));
}

#[tokio::test]
async fn test_execute_sequential_layer() {
    let executor = LayerExecutor::new();
    let mut layer = Layer {
        id: Uuid::new_v4(),
        name: "seq_test".to_string(),
        agents: vec!["agent1".to_string(), "agent2".to_string()],
        config: LayerConfig::default(),
    };
    layer.config.mode = ExecutionMode::Sequential;
    let layer_id = layer.id;

    executor.register_layer(layer).await.unwrap();

    let input = LayerInput {
        data: serde_json::json!({"test": "data"}),
        metadata: HashMap::new(),
    };

    let output = executor.execute_layer(layer_id, input).await.unwrap();
    assert!(output.agent_results.iter().all(|r| r.success));
    assert_eq!(output.agent_results.len(), 2);
}

#[tokio::test]
async fn test_get_layer_nonexistent() {
    let executor = LayerExecutor::new();
    let result = executor.get_layer(Uuid::new_v4()).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_layer_executor_default() {
    let executor = LayerExecutor::default();
    assert!(executor.get_layer(Uuid::new_v4()).await.is_none());
}

#[tokio::test]
async fn test_execute_layer_continue_strategy() {
    let executor = LayerExecutor::new();
    let mut layer = Layer {
        id: Uuid::new_v4(),
        name: "seq_continue".to_string(),
        agents: vec!["a1".to_string(), "a2".to_string()],
        config: LayerConfig::default(),
    };
    layer.config.mode = ExecutionMode::Sequential;
    layer.config.failure_strategy = FailureStrategy::Continue;
    let layer_id = layer.id;

    executor.register_layer(layer).await.unwrap();

    let input = LayerInput {
        data: serde_json::json!({}),
        metadata: HashMap::new(),
    };

    let output = executor.execute_layer(layer_id, input).await.unwrap();
    assert!(output.agent_results.iter().all(|r| r.success));
    assert_eq!(output.agent_results.len(), 2);
}

#[tokio::test]
async fn test_layer_error_display_all_variants() {
    let lid = Uuid::new_v4();
    let e1 = LayerError::LayerAlreadyExists(lid);
    assert!(format!("{}", e1).contains("Layer already exists"));

    let e2 = LayerError::LayerNotFound(lid);
    assert!(format!("{}", e2).contains("Layer not found"));

    let e3 = LayerError::DependencyNotFound(lid);
    assert!(format!("{}", e3).contains("Dependency not found"));

    let e4 = LayerError::ExecutionFailed("oops".to_string());
    assert!(format!("{}", e4).contains("Execution failed: oops"));

    let e5 = LayerError::TimeoutExceeded(lid);
    assert!(format!("{}", e5).contains("timed out"));
}
