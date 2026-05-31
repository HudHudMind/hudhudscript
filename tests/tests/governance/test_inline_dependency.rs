use chrono::Utc;
use hudhudscript_governance::dependency::{
    extract_constitution_references, CircularDependencyError, DependencyGraph,
};
use hudhudscript_governance::*;

#[test]
fn test_empty_graph() {
    let graph = DependencyGraph::new();
    assert!(graph.validate_all().is_ok());
}

#[test]
fn test_single_constitution_no_dependencies() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec![]);

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&"cons.1".to_string()), 0);
}

#[test]
fn test_linear_dependency_chain() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec![]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);
    graph.add_constitution("cons.3".to_string(), vec!["cons.2".to_string()]);

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&"cons.1".to_string()), 0);
    assert_eq!(graph.get_depth(&"cons.2".to_string()), 1);
    assert_eq!(graph.get_depth(&"cons.3".to_string()), 2);
}

#[test]
fn test_simple_circular_dependency() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec!["cons.2".to_string()]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);

    let result = graph.validate_all();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.dependency_chain.contains(&"cons.1".to_string()));
    assert!(err.dependency_chain.contains(&"cons.2".to_string()));
}

#[test]
fn test_self_reference() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec!["cons.1".to_string()]);

    let result = graph.validate_all();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.dependency_chain.len(), 2);
    assert_eq!(err.dependency_chain[0], "cons.1");
    assert_eq!(err.dependency_chain[1], "cons.1");
}

#[test]
fn test_complex_circular_dependency() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec!["cons.2".to_string()]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.3".to_string()]);
    graph.add_constitution("cons.3".to_string(), vec!["cons.4".to_string()]);
    graph.add_constitution("cons.4".to_string(), vec!["cons.2".to_string()]);

    let result = graph.validate_all();
    assert!(result.is_err());

    let err = result.unwrap_err();
    // The cycle should include cons.2, cons.3, cons.4
    assert!(err.dependency_chain.len() >= 3);
}

#[test]
fn test_validate_no_cycle_before_adding() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec![]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);

    // Try to add cons.3 that references cons.2 (should be OK)
    let result = graph.validate_no_cycle(&"cons.3".to_string(), &["cons.2".to_string()]);
    assert!(result.is_ok());

    // Try to add cons.1 that references cons.2 (would create a cycle)
    let result = graph.validate_no_cycle(&"cons.1".to_string(), &["cons.2".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_multiple_dependencies() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec![]);
    graph.add_constitution("cons.2".to_string(), vec![]);
    graph.add_constitution(
        "cons.3".to_string(),
        vec!["cons.1".to_string(), "cons.2".to_string()],
    );

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&"cons.3".to_string()), 1);
}

#[test]
fn test_diamond_dependency() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec![]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);
    graph.add_constitution("cons.3".to_string(), vec!["cons.1".to_string()]);
    graph.add_constitution(
        "cons.4".to_string(),
        vec!["cons.2".to_string(), "cons.3".to_string()],
    );

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&"cons.4".to_string()), 2);
}

#[test]
fn test_arbitrary_depth() {
    let mut graph = DependencyGraph::new();
    let depth = 10;

    // Create a chain of depth 10
    graph.add_constitution("cons.0".to_string(), vec![]);
    for i in 1..=depth {
        graph.add_constitution(format!("cons.{}", i), vec![format!("cons.{}", i - 1)]);
    }

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&format!("cons.{}", depth)), depth);
}

#[test]
fn test_error_message_format() {
    let error = CircularDependencyError::new(vec![
        "cons.1".to_string(),
        "cons.2".to_string(),
        "cons.3".to_string(),
        "cons.1".to_string(),
    ]);

    let message = error.message();
    assert!(message.contains("cons.1"));
    assert!(message.contains("cons.2"));
    assert!(message.contains("cons.3"));
    assert!(message.contains("->"));
}

#[test]
fn test_validate_no_cycle_with_existing_cycle() {
    let mut graph = DependencyGraph::new();
    // Create an existing cycle
    graph.add_constitution("cons.1".to_string(), vec!["cons.2".to_string()]);
    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);

    // Try to add a new constitution that doesn't participate in the cycle
    let result = graph.validate_no_cycle(&"cons.3".to_string(), &["cons.1".to_string()]);
    // This should still detect the existing cycle when traversing from cons.3
    assert!(result.is_err());
}

#[test]
fn test_depth_with_no_dependencies() {
    let graph = DependencyGraph::new();
    assert_eq!(graph.get_depth(&"cons.1".to_string()), 0);
}

#[test]
fn test_depth_with_nonexistent_constitution() {
    let mut graph = DependencyGraph::new();
    graph.add_constitution("cons.1".to_string(), vec![]);

    // Asking for depth of a constitution that doesn't exist
    assert_eq!(graph.get_depth(&"cons.999".to_string()), 0);
}

// ---- extract_constitution_references tests ----

#[test]
fn test_extract_refs_from_description_extends() {
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: Some("extends cons.2".to_string()),
        laws: std::collections::HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let refs = extract_constitution_references(&constitution);
    assert_eq!(refs, vec!["cons.2".to_string()]);
}

#[test]
fn test_extract_refs_from_description_includes() {
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: Some("includes cons.3".to_string()),
        laws: std::collections::HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let refs = extract_constitution_references(&constitution);
    assert_eq!(refs, vec!["cons.3".to_string()]);
}

#[test]
fn test_extract_refs_no_self_reference() {
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: Some("extends cons.1".to_string()),
        laws: std::collections::HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let refs = extract_constitution_references(&constitution);
    assert!(refs.is_empty(), "Should not include self-reference");
}

#[test]
fn test_extract_refs_from_law_conditions() {
    let mut laws = std::collections::HashMap::new();
    laws.insert(
        "law.1".to_string(),
        Law {
            id: "law.1".to_string(),
            constitution_id: "cons.1".to_string(),
            name: "Test Law".to_string(),
            description: "A regular law".to_string(),
            enforcement_level: EnforcementLevel::Mandatory,
            conditions: vec![Condition::Equals {
                field: "target_constitution".to_string(),
                value: serde_json::Value::String("cons.5".to_string()),
            }],
        },
    );

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws,
        created_at: Utc::now(),
        version: 1,
    };

    let refs = extract_constitution_references(&constitution);
    assert_eq!(refs, vec!["cons.5".to_string()]);
}

#[test]
fn test_extract_refs_deduplication() {
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: Some("extends cons.2, includes cons.2".to_string()),
        laws: std::collections::HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let refs = extract_constitution_references(&constitution);
    assert_eq!(refs.len(), 1, "Should deduplicate references");
    assert_eq!(refs[0], "cons.2");
}

#[test]
fn test_extract_refs_empty_constitution() {
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws: std::collections::HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let refs = extract_constitution_references(&constitution);
    assert!(refs.is_empty());
}

#[test]
fn test_extract_refs_integrates_with_dependency_graph() {
    let mut graph = DependencyGraph::new();

    // cons.1 extends cons.2
    let cons1 = Constitution {
        id: "cons.1".to_string(),
        name: "Child".to_string(),
        description: Some("extends cons.2".to_string()),
        laws: std::collections::HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let refs = extract_constitution_references(&cons1);
    graph.add_constitution(cons1.id.clone(), refs);
    graph.add_constitution("cons.2".to_string(), vec![]);

    assert!(graph.validate_all().is_ok());
    assert_eq!(graph.get_depth(&cons1.id), 1);
}
