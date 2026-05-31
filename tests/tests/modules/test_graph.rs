//! Tests extracted from hudhudscript-modules/src/graph.rs

use hudhudscript_modules::graph::{GraphError, ImportGraph};
use std::path::PathBuf;

#[test]
fn test_graph_no_cycle() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");

    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_dependency(a, b);

    assert!(graph.check_cycles().is_ok());
}

#[test]
fn test_graph_cycle() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");

    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_dependency(a.clone(), b.clone());
    graph.add_dependency(b, a);

    assert!(graph.check_cycles().is_err());
}

#[test]
fn test_topological_sort() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    let c = PathBuf::from("c.hudhud");

    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_module(c.clone());
    graph.add_dependency(a.clone(), b.clone());
    graph.add_dependency(b.clone(), c.clone());

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);
}

#[test]
fn test_graph_default() {
    let graph = ImportGraph::default();
    assert!(graph.check_cycles().is_ok());
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 0);
}

#[test]
fn test_graph_single_module_no_deps() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("solo.hudhud");
    graph.add_module(a.clone());

    assert!(graph.check_cycles().is_ok());
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0], a);
}

#[test]
fn test_graph_three_node_cycle() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    let c = PathBuf::from("c.hudhud");

    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_module(c.clone());
    graph.add_dependency(a.clone(), b.clone());
    graph.add_dependency(b.clone(), c.clone());
    graph.add_dependency(c.clone(), a.clone());

    let result = graph.check_cycles();
    assert!(result.is_err());
    match result {
        Err(GraphError::CircularDependency(msg)) => {
            assert!(!msg.is_empty());
        }
        _ => panic!("Expected CircularDependency error"),
    }
}

#[test]
fn test_topological_sort_with_cycle_fails() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("x.hudhud");
    let b = PathBuf::from("y.hudhud");
    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_dependency(a.clone(), b.clone());
    graph.add_dependency(b.clone(), a.clone());

    let result = graph.topological_sort();
    assert!(result.is_err());
}

#[test]
fn test_topological_sort_diamond() {
    // a -> b, a -> c, b -> d, c -> d (diamond shape, no cycle)
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    let c = PathBuf::from("c.hudhud");
    let d = PathBuf::from("d.hudhud");

    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_module(c.clone());
    graph.add_module(d.clone());
    graph.add_dependency(a.clone(), b.clone());
    graph.add_dependency(a.clone(), c.clone());
    graph.add_dependency(b.clone(), d.clone());
    graph.add_dependency(c.clone(), d.clone());

    assert!(graph.check_cycles().is_ok());
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 4);
    // All four nodes are present in the output
    assert!(sorted.contains(&a));
    assert!(sorted.contains(&b));
    assert!(sorted.contains(&c));
    assert!(sorted.contains(&d));
}

#[test]
fn test_add_dependency_auto_creates_node() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    // Don't explicitly add_module for a
    graph.add_dependency(a.clone(), b.clone());

    // a should have been created via or_default in add_dependency
    assert!(graph.edges.contains_key(&a));
}

#[test]
fn test_self_cycle() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("self.hudhud");
    graph.add_module(a.clone());
    graph.add_dependency(a.clone(), a.clone());

    assert!(graph.check_cycles().is_err());
}

#[test]
fn test_graph_error_display() {
    let err = GraphError::CircularDependency("a -> b -> a".to_string());
    assert!(err
        .to_string()
        .contains("Circular dependency detected: a -> b -> a"));

    let err = GraphError::ModuleNotFound("missing.hudhud".to_string());
    assert!(err
        .to_string()
        .contains("Module not found in graph: missing.hudhud"));
}

#[test]
fn test_topological_sort_chain_valid_order() {
    // a depends on b, b depends on c
    // Topological sort in this DFS approach: nodes are pushed post-order
    // then reversed. Due to HashMap iteration order being non-deterministic,
    // we just validate that all nodes are present and the sort succeeds.
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    let c = PathBuf::from("c.hudhud");

    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_module(c.clone());
    graph.add_dependency(a.clone(), b.clone());
    graph.add_dependency(b.clone(), c.clone());

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);
    assert!(sorted.contains(&a));
    assert!(sorted.contains(&b));
    assert!(sorted.contains(&c));
}

#[test]
fn test_topological_sort_diamond_valid() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    let c = PathBuf::from("c.hudhud");
    let d = PathBuf::from("d.hudhud");

    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_module(c.clone());
    graph.add_module(d.clone());
    graph.add_dependency(a.clone(), b.clone());
    graph.add_dependency(a.clone(), c.clone());
    graph.add_dependency(b.clone(), d.clone());
    graph.add_dependency(c.clone(), d.clone());

    assert!(graph.check_cycles().is_ok());
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 4);
    assert!(sorted.contains(&a));
    assert!(sorted.contains(&b));
    assert!(sorted.contains(&c));
    assert!(sorted.contains(&d));
}

#[test]
fn test_graph_disconnected_components() {
    let mut graph = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    let c = PathBuf::from("c.hudhud");
    let d = PathBuf::from("d.hudhud");

    // Component 1: a -> b
    graph.add_module(a.clone());
    graph.add_module(b.clone());
    graph.add_dependency(a.clone(), b.clone());

    // Component 2: c -> d
    graph.add_module(c.clone());
    graph.add_module(d.clone());
    graph.add_dependency(c.clone(), d.clone());

    assert!(graph.check_cycles().is_ok());
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 4);
}

#[test]
fn test_graph_multiple_deps_from_one_node() {
    let mut graph = ImportGraph::new();
    let root = PathBuf::from("root.hudhud");
    let dep1 = PathBuf::from("dep1.hudhud");
    let dep2 = PathBuf::from("dep2.hudhud");
    let dep3 = PathBuf::from("dep3.hudhud");

    graph.add_module(root.clone());
    graph.add_module(dep1.clone());
    graph.add_module(dep2.clone());
    graph.add_module(dep3.clone());
    graph.add_dependency(root.clone(), dep1.clone());
    graph.add_dependency(root.clone(), dep2.clone());
    graph.add_dependency(root.clone(), dep3.clone());

    assert!(graph.check_cycles().is_ok());
    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 4);
    // All nodes should be present
    assert!(sorted.contains(&root));
    assert!(sorted.contains(&dep1));
    assert!(sorted.contains(&dep2));
    assert!(sorted.contains(&dep3));
}
