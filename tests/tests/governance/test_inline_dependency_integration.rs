//! Integration tests for circular dependency detection

use chrono::Utc;
use hudhudscript_governance::dependency::{CircularDependencyError, DependencyGraph};
use hudhudscript_governance::types::Constitution;
use std::collections::HashMap;

#[test]
fn test_integration_validate_before_cache_storage() {
    // Simulate the workflow: validate before storing in cache
    let mut graph = DependencyGraph::new();

    // Add first constitution (no dependencies)
    let cons1 = Constitution {
        id: "cons.1".to_string(),
        name: "Base Constitution".to_string(),
        description: Some("Foundation constitution".to_string()),
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let result = graph.validate_no_cycle(&cons1.id, &[]);
    assert!(result.is_ok());
    graph.add_constitution(cons1.id.clone(), vec![]);

    // Add second constitution that references first
    let cons2 = Constitution {
        id: "cons.2".to_string(),
        name: "Extended Constitution".to_string(),
        description: Some("Extends base constitution".to_string()),
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let refs2 = vec!["cons.1".to_string()];
    let result = graph.validate_no_cycle(&cons2.id, &refs2);
    assert!(result.is_ok());
    graph.add_constitution(cons2.id.clone(), refs2);

    // Try to add third constitution that would create a cycle
    let cons3_refs = vec!["cons.2".to_string()];
    let result = graph.validate_no_cycle(&"cons.1".to_string(), &cons3_refs);
    assert!(result.is_err());

    if let Err(err) = result {
        assert!(err.dependency_chain.len() >= 2);
    }
}

#[test]
fn test_acyclic_hierarchy_arbitrary_depth() {
    let mut graph = DependencyGraph::new();

    // Create a deep hierarchy (depth 20)
    let depth = 20;
    graph.add_constitution("cons.0".to_string(), vec![]);

    for i in 1..=depth {
        let refs = vec![format!("cons.{}", i - 1)];
        let result = graph.validate_no_cycle(&format!("cons.{}", i), &refs);
        assert!(result.is_ok(), "Failed at depth {}", i);
        graph.add_constitution(format!("cons.{}", i), refs);
    }

    // Verify the depth
    assert_eq!(graph.get_depth(&format!("cons.{}", depth)), depth);

    // Verify no cycles
    assert!(graph.validate_all().is_ok());
}

#[test]
fn test_complex_acyclic_graph() {
    let mut graph = DependencyGraph::new();

    graph.add_constitution("cons.1".to_string(), vec![]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);
    graph.add_constitution("cons.3".to_string(), vec!["cons.1".to_string()]);
    graph.add_constitution(
        "cons.4".to_string(),
        vec!["cons.2".to_string(), "cons.3".to_string()],
    );
    graph.add_constitution("cons.5".to_string(), vec!["cons.4".to_string()]);

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&"cons.5".to_string()), 3);
}

#[test]
fn test_detect_cycle_in_complex_graph() {
    let mut graph = DependencyGraph::new();

    graph.add_constitution("cons.1".to_string(), vec!["cons.2".to_string()]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.3".to_string()]);
    graph.add_constitution("cons.3".to_string(), vec!["cons.1".to_string()]);

    let result = graph.validate_all();
    assert!(result.is_err());

    if let Err(err) = result {
        // The cycle should contain all three constitutions
        assert!(err.dependency_chain.len() >= 3);
        assert!(err.dependency_chain.contains(&"cons.1".to_string()));
        assert!(err.dependency_chain.contains(&"cons.2".to_string()));
        assert!(err.dependency_chain.contains(&"cons.3".to_string()));
    }
}

#[test]
fn test_multiple_independent_hierarchies() {
    let mut graph = DependencyGraph::new();

    // Hierarchy 1: cons.1 -> cons.2 -> cons.3
    graph.add_constitution("cons.1".to_string(), vec![]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);
    graph.add_constitution("cons.3".to_string(), vec!["cons.2".to_string()]);

    // Hierarchy 2: cons.10 -> cons.11 -> cons.12
    graph.add_constitution("cons.10".to_string(), vec![]);
    graph.add_constitution("cons.11".to_string(), vec!["cons.10".to_string()]);
    graph.add_constitution("cons.12".to_string(), vec!["cons.11".to_string()]);

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&"cons.3".to_string()), 2);
    assert_eq!(graph.get_depth(&"cons.12".to_string()), 2);
}

#[test]
fn test_prevent_cycle_creation_incrementally() {
    let mut graph = DependencyGraph::new();

    // Build a valid chain
    graph.add_constitution("cons.1".to_string(), vec![]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);
    graph.add_constitution("cons.3".to_string(), vec!["cons.2".to_string()]);

    // Now try to modify cons.1 to reference cons.3 (would create a cycle)
    let result = graph.validate_no_cycle(&"cons.1".to_string(), &["cons.3".to_string()]);
    assert!(result.is_err());

    // The original graph should still be valid
    assert!(graph.validate_all().is_ok());
}

#[test]
fn test_error_chain_shows_complete_cycle() {
    let mut graph = DependencyGraph::new();

    // Create: A -> B -> C -> D -> B (cycle from B)
    graph.add_constitution("cons.A".to_string(), vec!["cons.B".to_string()]);
    graph.add_constitution("cons.B".to_string(), vec!["cons.C".to_string()]);
    graph.add_constitution("cons.C".to_string(), vec!["cons.D".to_string()]);
    graph.add_constitution("cons.D".to_string(), vec!["cons.B".to_string()]);

    let result = graph.validate_all();
    assert!(result.is_err());

    if let Err(err) = result {
        // Verify the cycle is properly represented
        assert!(err.dependency_chain.len() >= 3);
    }
}

#[test]
fn test_wide_dependency_tree() {
    let mut graph = DependencyGraph::new();

    // Create a root with many children
    graph.add_constitution("cons.root".to_string(), vec![]);

    for i in 1..=10 {
        graph.add_constitution(format!("cons.child{}", i), vec!["cons.root".to_string()]);
    }

    // Add a constitution that depends on all children
    let all_children: Vec<String> = (1..=10).map(|i| format!("cons.child{}", i)).collect();
    graph.add_constitution("cons.grandchild".to_string(), all_children);

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&"cons.grandchild".to_string()), 2);
}

#[test]
fn test_circular_dependency_error_display() {
    let error = CircularDependencyError::new(vec![
        "cons.1".to_string(),
        "cons.2".to_string(),
        "cons.3".to_string(),
        "cons.1".to_string(),
    ]);

    let display_str = format!("{}", error);
    assert!(display_str.contains("Circular dependency detected"));
    assert!(display_str.contains("cons.1"));
    assert!(display_str.contains("cons.2"));
    assert!(display_str.contains("cons.3"));
}

#[test]
fn test_validate_empty_references() {
    let graph = DependencyGraph::new();

    // Validating a constitution with no references should always succeed
    let result = graph.validate_no_cycle(&"cons.1".to_string(), &[]);
    assert!(result.is_ok());
}

#[test]
fn test_validate_nonexistent_reference() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec![]);

    // Referencing a constitution that doesn't exist yet is OK
    // (it might be added later)
    let result = graph.validate_no_cycle(&"cons.2".to_string(), &["cons.999".to_string()]);
    assert!(result.is_ok());
}

#[test]
fn test_depth_calculation_with_cycles() {
    let mut graph = DependencyGraph::new();

    // Create a cycle
    graph.add_constitution("cons.1".to_string(), vec!["cons.2".to_string()]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);

    // Depth calculation should handle cycles gracefully (not infinite loop)
    let depth = graph.get_depth(&"cons.1".to_string());
    // Just verify it completes without hanging (usize is always >= 0)
    assert!(depth < 1000); // Should return some reasonable finite value
}
