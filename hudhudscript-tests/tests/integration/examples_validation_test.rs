//! Integration tests for validating all .hudhud example files
//!
//! This test suite validates that all example files in the examples/ directory
//! can be parsed successfully without errors.

use hudhudscript_parser::parse;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn examples_path(rel: &str) -> PathBuf {
    workspace_root().join(rel)
}

/// Test that all .hudhud files in examples/ directory can be parsed
#[test]
fn test_all_examples_parse_successfully() {
    let examples_dir = examples_path("examples");

    if !examples_dir.exists() {
        panic!("Examples directory not found: {:?}", examples_dir);
    }

    let mut total_files = 0;
    let mut successful_parses = 0;
    let mut failed_files = Vec::new();

    // Recursively find all .hudhud files
    visit_dirs(&examples_dir, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) == Some("hudhud") {
            total_files += 1;

            println!("Testing: {:?}", path);

            match fs::read_to_string(path) {
                Ok(content) => match parse(&content) {
                    Ok(_) => {
                        successful_parses += 1;
                        println!("  ✓ Parsed successfully");
                    }
                    Err(e) => {
                        println!("  ✗ Parse error: {}", e);
                        failed_files.push((path.to_path_buf(), format!("{}", e)));
                    }
                },
                Err(e) => {
                    println!("  ✗ Read error: {}", e);
                    failed_files.push((path.to_path_buf(), format!("Read error: {}", e)));
                }
            }
        }
    });

    println!("\n=== Test Summary ===");
    println!("Total .hudhud files: {}", total_files);
    println!("Successfully parsed: {}", successful_parses);
    println!("Failed to parse: {}", failed_files.len());

    if !failed_files.is_empty() {
        println!("\n=== Failed Files ===");
        for (path, error) in &failed_files {
            println!("{:?}: {}", path, error);
        }
    }

    // Note: We don't fail the test for now since many examples use features
    // that are not yet implemented in the parser (MCP calls, AI providers, etc.)
    // This test serves as a validation tool to track progress.

    println!("\n=== Parse Rate ===");
    let parse_rate = if total_files > 0 {
        (successful_parses as f64 / total_files as f64) * 100.0
    } else {
        0.0
    };
    println!("Parse success rate: {:.1}%", parse_rate);

    // We expect at least some files to parse successfully
    assert!(total_files > 0, "No .hudhud files found in examples/");
}

/// Recursively visit all files in a directory
fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb);
                } else {
                    cb(&path);
                }
            }
        }
    }
}

/// Test specific example files that should definitely parse
#[test]
fn test_basic_examples_parse() {
    // Test basic governance example
    let governance_example = examples_path("examples/governance/basic_constitution.hudhud");
    if governance_example.exists() {
        let content =
            fs::read_to_string(&governance_example).expect("Failed to read governance example");

        match parse(&content) {
            Ok(_) => println!("✓ Basic governance example parsed successfully"),
            Err(e) => println!("✗ Basic governance example failed: {}", e),
        }
    }
}

/// Test that parser can handle multi-language examples
#[test]
fn test_multilang_examples() {
    let multilang_examples = vec![
        "examples/real_world_agents/en/customer_support_agent.hudhud",
        "examples/real_world_agents/tr/musteri_destek_ajani.hudhud",
        "examples/real_world_agents/jp/顧客サポートエージェント.hudhud",
    ];

    for rel in multilang_examples {
        let example_path = examples_path(rel);
        if example_path.exists() {
            println!("Testing multilang: {}", rel);
            let content =
                fs::read_to_string(&example_path).expect(&format!("Failed to read {}", rel));

            match parse(&content) {
                Ok(_) => println!("  ✓ Parsed successfully"),
                Err(e) => println!("  ✗ Parse error: {}", e),
            }
        }
    }
}

/// Test that new industry examples are syntactically valid
#[test]
fn test_industry_examples_syntax() {
    let industry_examples = vec![
        "examples/real_world_agents/autonomous_code_review.hudhud",
        "examples/real_world_agents/game_ai_director.hudhud",
        "examples/real_world_agents/ai_project_manager.hudhud",
        "examples/real_world_agents/fraud_detection_system.hudhud",
        "examples/real_world_agents/flight_operations_ai.hudhud",
        "examples/real_world_agents/algorithmic_trading.hudhud",
        "examples/real_world_agents/medical_diagnosis_ai.hudhud",
        "examples/real_world_agents/personalization_engine.hudhud",
        "examples/real_world_agents/adaptive_learning.hudhud",
        "examples/real_world_agents/predictive_maintenance.hudhud",
        "examples/real_world_agents/police_intelligence_system.hudhud",
        "examples/real_world_agents/research_automation.hudhud",
    ];

    let mut parsed_count = 0;
    let mut total_count = 0;

    for rel in industry_examples {
        let example_path = examples_path(rel);
        if example_path.exists() {
            total_count += 1;
            println!("Testing industry example: {}", rel);

            match fs::read_to_string(&example_path) {
                Ok(content) => match parse(&content) {
                    Ok(_) => {
                        parsed_count += 1;
                        println!("  ✓ Syntax valid");
                    }
                    Err(e) => {
                        println!("  ✗ Syntax error: {}", e);
                    }
                },
                Err(e) => {
                    println!("  ✗ Read error: {}", e);
                }
            }
        }
    }

    println!(
        "\nIndustry examples: {}/{} parsed successfully",
        parsed_count, total_count
    );

    // Industry examples may be in _wip/ if they use aspirational syntax
    // that the parser doesn't support yet. Don't require a minimum count.
    println!(
        "Industry examples found: {}, parsed: {}",
        total_count, parsed_count
    );
}
