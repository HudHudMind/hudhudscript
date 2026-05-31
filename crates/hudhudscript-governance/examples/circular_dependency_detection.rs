//! Example demonstrating circular dependency detection for constitutions
//!
//! This example shows how to use the dependency graph to validate that
//! constitutions don't create circular references.

use hudhudscript_governance::dependency::DependencyGraph;

fn main() {
    println!("=== Circular Dependency Detection Example ===\n");

    // Example 1: Valid linear hierarchy
    println!("Example 1: Valid Linear Hierarchy");
    println!("-----------------------------------");
    let mut graph = DependencyGraph::new();

    graph.add_constitution("cons.1".to_string(), vec![]);
    println!("✓ Added cons.1 (no dependencies)");

    graph.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);
    println!("✓ Added cons.2 (depends on cons.1)");

    graph.add_constitution("cons.3".to_string(), vec!["cons.2".to_string()]);
    println!("✓ Added cons.3 (depends on cons.2)");

    match graph.validate_all() {
        Ok(()) => println!("✓ No circular dependencies detected\n"),
        Err(e) => println!("✗ Error: {}\n", e),
    }

    // Example 2: Detecting a simple cycle
    println!("Example 2: Simple Circular Dependency");
    println!("--------------------------------------");
    let mut graph2 = DependencyGraph::new();

    graph2.add_constitution("cons.A".to_string(), vec!["cons.B".to_string()]);
    println!("Added cons.A -> cons.B");

    graph2.add_constitution("cons.B".to_string(), vec!["cons.A".to_string()]);
    println!("Added cons.B -> cons.A");

    match graph2.validate_all() {
        Ok(()) => println!("✓ No circular dependencies detected\n"),
        Err(e) => println!("✗ Circular dependency detected: {}\n", e),
    }

    // Example 3: Validating before adding
    println!("Example 3: Pre-validation Before Adding");
    println!("----------------------------------------");
    let mut graph3 = DependencyGraph::new();

    graph3.add_constitution("cons.1".to_string(), vec![]);
    graph3.add_constitution("cons.2".to_string(), vec!["cons.1".to_string()]);
    graph3.add_constitution("cons.3".to_string(), vec!["cons.2".to_string()]);

    println!("Current chain: cons.1 <- cons.2 <- cons.3");

    // Try to make cons.1 reference cons.3 (would create a cycle)
    println!("\nAttempting to add: cons.1 -> cons.3");
    match graph3.validate_no_cycle(&"cons.1".to_string(), &["cons.3".to_string()]) {
        Ok(()) => println!("✓ Would be valid (no cycle)"),
        Err(e) => println!("✗ Would create cycle: {}", e),
    }

    // Try to add a new constitution that doesn't create a cycle
    println!("\nAttempting to add: cons.4 -> cons.3");
    match graph3.validate_no_cycle(&"cons.4".to_string(), &["cons.3".to_string()]) {
        Ok(()) => {
            println!("✓ Would be valid (no cycle)");
            graph3.add_constitution("cons.4".to_string(), vec!["cons.3".to_string()]);
            println!("✓ Successfully added cons.4\n");
        }
        Err(e) => println!("✗ Would create cycle: {}\n", e),
    }

    // Example 4: Complex cycle detection
    println!("Example 4: Complex Circular Dependency");
    println!("---------------------------------------");
    let mut graph4 = DependencyGraph::new();

    graph4.add_constitution("cons.1".to_string(), vec!["cons.2".to_string()]);
    graph4.add_constitution("cons.2".to_string(), vec!["cons.3".to_string()]);
    graph4.add_constitution("cons.3".to_string(), vec!["cons.4".to_string()]);
    graph4.add_constitution("cons.4".to_string(), vec!["cons.2".to_string()]);

    println!("Chain: cons.1 -> cons.2 -> cons.3 -> cons.4 -> cons.2");

    match graph4.validate_all() {
        Ok(()) => println!("✓ No circular dependencies detected\n"),
        Err(e) => {
            println!("✗ Circular dependency detected!");
            println!("   Cycle: {}\n", e.message());
        }
    }

    // Example 5: Diamond dependency (valid)
    println!("Example 5: Diamond Dependency (Valid)");
    println!("--------------------------------------");
    let mut graph5 = DependencyGraph::new();

    graph5.add_constitution("cons.base".to_string(), vec![]);
    graph5.add_constitution("cons.left".to_string(), vec!["cons.base".to_string()]);
    graph5.add_constitution("cons.right".to_string(), vec!["cons.base".to_string()]);
    graph5.add_constitution(
        "cons.top".to_string(),
        vec!["cons.left".to_string(), "cons.right".to_string()],
    );

    println!("Structure:");
    println!("       cons.top");
    println!("       /      \\");
    println!("  cons.left  cons.right");
    println!("       \\      /");
    println!("      cons.base");

    match graph5.validate_all() {
        Ok(()) => {
            println!("\n✓ No circular dependencies detected");
            println!(
                "✓ Depth of cons.top: {}\n",
                graph5.get_depth(&"cons.top".to_string())
            );
        }
        Err(e) => println!("\n✗ Error: {}\n", e),
    }

    // Example 6: Arbitrary depth hierarchy
    println!("Example 6: Deep Hierarchy (Depth 10)");
    println!("-------------------------------------");
    let mut graph6 = DependencyGraph::new();

    graph6.add_constitution("cons.0".to_string(), vec![]);
    for i in 1..=10 {
        graph6.add_constitution(format!("cons.{}", i), vec![format!("cons.{}", i - 1)]);
    }

    println!("Chain: cons.0 <- cons.1 <- ... <- cons.10");

    match graph6.validate_all() {
        Ok(()) => {
            println!("✓ No circular dependencies detected");
            println!(
                "✓ Depth of cons.10: {}\n",
                graph6.get_depth(&"cons.10".to_string())
            );
        }
        Err(e) => println!("✗ Error: {}\n", e),
    }

    println!("=== Example Complete ===");
}
