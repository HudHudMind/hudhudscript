use hudhudscript_governance::swarm::SwarmError;
use hudhudscript_governance::*;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn test_swarm_new() {
    let swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string(), "agent2".to_string()],
        CoordinationStrategy::Parallel,
    );

    assert_eq!(swarm.id, "swarm1");
    assert_eq!(swarm.name, "Test Swarm");
    assert_eq!(swarm.agents.len(), 2);
    assert_eq!(swarm.coordination_strategy, CoordinationStrategy::Parallel);
    assert!(swarm.shared_state.is_empty());
}

#[test]
fn test_swarm_initialization_empty_shared_state() {
    let swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Sequential,
    );

    assert!(swarm.shared_state.is_empty());
    assert_eq!(swarm.shared_state.len(), 0);
}

#[test]
fn test_swarm_all_coordination_strategies() {
    let strategies = vec![
        CoordinationStrategy::Parallel,
        CoordinationStrategy::Sequential,
        CoordinationStrategy::Competitive,
        CoordinationStrategy::Collaborative,
    ];

    for strategy in strategies {
        let swarm = Swarm::new(
            "swarm1".to_string(),
            "Test Swarm".to_string(),
            vec!["agent1".to_string()],
            strategy,
        );

        assert_eq!(swarm.coordination_strategy, strategy);
    }
}

#[test]
fn test_validate_agents_all_exist() {
    let swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string(), "agent2".to_string()],
        CoordinationStrategy::Parallel,
    );

    let mut valid_agents = HashSet::new();
    valid_agents.insert("agent1".to_string());
    valid_agents.insert("agent2".to_string());
    valid_agents.insert("agent3".to_string());

    assert!(swarm.validate_agents(&valid_agents).is_ok());
}

#[test]
fn test_validate_agents_missing_agent() {
    let swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec![
            "agent1".to_string(),
            "agent2".to_string(),
            "agent3".to_string(),
        ],
        CoordinationStrategy::Parallel,
    );

    let mut valid_agents = HashSet::new();
    valid_agents.insert("agent1".to_string());
    valid_agents.insert("agent2".to_string());
    // agent3 is missing

    let result = swarm.validate_agents(&valid_agents);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        SwarmError::AgentNotFound("agent3".to_string())
    );
}

#[test]
fn test_add_agent() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    assert_eq!(swarm.agents.len(), 1);

    assert!(swarm.add_agent("agent2".to_string()).is_ok());
    assert_eq!(swarm.agents.len(), 2);
    assert!(swarm.has_agent("agent2"));
}

#[test]
fn test_add_duplicate_agent() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    let result = swarm.add_agent("agent1".to_string());
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        SwarmError::DuplicateAgent("agent1".to_string())
    );
    assert_eq!(swarm.agents.len(), 1);
}

#[test]
fn test_remove_agent() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string(), "agent2".to_string()],
        CoordinationStrategy::Parallel,
    );

    assert!(swarm.remove_agent("agent1").is_ok());
    assert_eq!(swarm.agents.len(), 1);
    assert!(!swarm.has_agent("agent1"));
    assert!(swarm.has_agent("agent2"));
}

#[test]
fn test_remove_nonexistent_agent() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    let result = swarm.remove_agent("agent2");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        SwarmError::AgentNotFound("agent2".to_string())
    );
}

#[test]
fn test_get_set_strategy() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    assert_eq!(swarm.get_strategy(), CoordinationStrategy::Parallel);

    swarm.set_strategy(CoordinationStrategy::Sequential);
    assert_eq!(swarm.get_strategy(), CoordinationStrategy::Sequential);

    swarm.set_strategy(CoordinationStrategy::Competitive);
    assert_eq!(swarm.get_strategy(), CoordinationStrategy::Competitive);

    swarm.set_strategy(CoordinationStrategy::Collaborative);
    assert_eq!(swarm.get_strategy(), CoordinationStrategy::Collaborative);
}

#[test]
fn test_shared_state_operations() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    // Set and get
    swarm.set_shared_state("counter".to_string(), json!(42));
    assert_eq!(swarm.get_shared_state("counter"), Some(&json!(42)));

    // Update existing key
    swarm.set_shared_state("counter".to_string(), json!(100));
    assert_eq!(swarm.get_shared_state("counter"), Some(&json!(100)));

    // Get non-existent key
    assert_eq!(swarm.get_shared_state("missing"), None);
}

#[test]
fn test_remove_shared_state() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    swarm.set_shared_state("temp".to_string(), json!("value"));
    assert!(swarm.get_shared_state("temp").is_some());

    let removed = swarm.remove_shared_state("temp");
    assert!(removed.is_ok());
    assert_eq!(removed.unwrap(), json!("value"));
    assert!(swarm.get_shared_state("temp").is_none());
}

#[test]
fn test_remove_nonexistent_shared_state() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    let result = swarm.remove_shared_state("missing");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        SwarmError::StateKeyNotFound("missing".to_string())
    );
}

#[test]
fn test_clear_shared_state() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    swarm.set_shared_state("key1".to_string(), json!(1));
    swarm.set_shared_state("key2".to_string(), json!(2));
    swarm.set_shared_state("key3".to_string(), json!(3));
    assert_eq!(swarm.shared_state.len(), 3);

    swarm.clear_shared_state();
    assert!(swarm.shared_state.is_empty());
    assert_eq!(swarm.shared_state.len(), 0);
}

#[test]
fn test_agent_count() {
    let swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec![
            "agent1".to_string(),
            "agent2".to_string(),
            "agent3".to_string(),
        ],
        CoordinationStrategy::Parallel,
    );

    assert_eq!(swarm.agent_count(), 3);
}

#[test]
fn test_has_agent() {
    let swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string(), "agent2".to_string()],
        CoordinationStrategy::Parallel,
    );

    assert!(swarm.has_agent("agent1"));
    assert!(swarm.has_agent("agent2"));
    assert!(!swarm.has_agent("agent3"));
    assert!(!swarm.has_agent("nonexistent"));
}

#[test]
fn test_shared_state_with_complex_values() {
    let mut swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string()],
        CoordinationStrategy::Parallel,
    );

    // Test with different JSON types
    swarm.set_shared_state("number".to_string(), json!(42));
    swarm.set_shared_state("string".to_string(), json!("hello"));
    swarm.set_shared_state("boolean".to_string(), json!(true));
    swarm.set_shared_state("array".to_string(), json!([1, 2, 3]));
    swarm.set_shared_state("object".to_string(), json!({"key": "value"}));

    assert_eq!(swarm.get_shared_state("number"), Some(&json!(42)));
    assert_eq!(swarm.get_shared_state("string"), Some(&json!("hello")));
    assert_eq!(swarm.get_shared_state("boolean"), Some(&json!(true)));
    assert_eq!(swarm.get_shared_state("array"), Some(&json!([1, 2, 3])));
    assert_eq!(
        swarm.get_shared_state("object"),
        Some(&json!({"key": "value"}))
    );
}

#[test]
fn test_swarm_error_display() {
    let e1 = SwarmError::AgentNotFound("agent1".to_string());
    assert!(format!("{}", e1).contains("Agent not found"));
    assert!(format!("{}", e1).contains("agent1"));

    let e2 = SwarmError::DuplicateAgent("agent2".to_string());
    assert!(format!("{}", e2).contains("Duplicate agent"));
    assert!(format!("{}", e2).contains("agent2"));

    let e3 = SwarmError::StateKeyNotFound("mykey".to_string());
    assert!(format!("{}", e3).contains("State key not found"));
    assert!(format!("{}", e3).contains("mykey"));
}

#[test]
fn test_swarm_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(SwarmError::AgentNotFound("a".to_string()));
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_swarm_serialization() {
    let swarm = Swarm::new(
        "swarm1".to_string(),
        "Test Swarm".to_string(),
        vec!["agent1".to_string(), "agent2".to_string()],
        CoordinationStrategy::Collaborative,
    );

    let json = serde_json::to_string(&swarm).unwrap();
    let deserialized: Swarm = serde_json::from_str(&json).unwrap();

    assert_eq!(swarm.id, deserialized.id);
    assert_eq!(swarm.name, deserialized.name);
    assert_eq!(swarm.agents, deserialized.agents);
    assert_eq!(
        swarm.coordination_strategy,
        deserialized.coordination_strategy
    );
    assert_eq!(swarm.shared_state, deserialized.shared_state);
}
