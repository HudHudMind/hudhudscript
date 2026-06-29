//! Parser declaration tests (issue #238)
//!
//! Verify that each declaration type can be parsed and executed correctly.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::{ObjMap, Value16};
use hudhudscript_parser::parse;

fn run(source: &str) -> Interpreter {
    let ast = parse(source).unwrap_or_else(|e| panic!("Parse error:\n{e}"));
    let mut interpreter = Interpreter::new();
    interpreter
        .execute(&ast)
        .unwrap_or_else(|e| panic!("Runtime error:\n{e:?}"));
    interpreter
}

// ── Helper: extract an Object from the environment ──────────────────────────

fn get_obj(interp: &Interpreter, name: &str) -> ObjMap {
    let val = interp
        .get_variable(name)
        .unwrap_or_else(|_| panic!("Variable '{name}' not found"));
    if let Some(obj) = val.as_object() {
        obj.clone()
    } else {
        panic!("Expected Object for '{name}', got {val:?}");
    }
}

fn assert_str_field(obj: &ObjMap, key: &str, expected: &str) {
    match obj.get(key) {
        Some(v) => {
            if let Some(s) = v.as_str() {
                assert_eq!(s, expected, "Field '{key}' mismatch");
            } else {
                panic!("Expected String for field '{key}', got {:?}", v);
            }
        }
        None => panic!("Field '{key}' not found"),
    }
}

fn assert_num_field(obj: &ObjMap, key: &str, expected: f64) {
    match obj.get(key) {
        Some(v) => {
            if let Some(n) = v.as_number() {
                assert!(
                    (n - expected).abs() < f64::EPSILON,
                    "Field '{key}': expected {expected}, got {n}"
                );
            } else {
                panic!("Expected Number for field '{key}', got {:?}", v);
            }
        }
        None => panic!("Field '{key}' not found"),
    }
}

// ── 2. Resource ─────────────────────────────────────────────────────────────

#[test]
fn test_resource_declaration() {
    let interp = run(r#"
        resource my_res {
            server: "s",
            uri: "file://x"
        }
    "#);

    let obj = get_obj(&interp, "my_res");
    assert_str_field(&obj, "server", "s");
    assert_str_field(&obj, "uri", "file://x");
}

// ── 3. Provider ─────────────────────────────────────────────────────────────

#[test]
fn test_provider_declaration() {
    let interp = run(r#"
        provider my_prov {
            type: "openai",
            model: "gpt-4"
        }
    "#);

    let obj = get_obj(&interp, "my_prov");
    assert_str_field(&obj, "type", "openai");
    assert_str_field(&obj, "model", "gpt-4");
    // Provider also injects its own name
    assert_str_field(&obj, "name", "my_prov");
}

// ── 4. Action ───────────────────────────────────────────────────────────────

#[test]
fn test_action_declaration() {
    let interp = run(r#"
        action my_action {
            priority: 5
        }
    "#);

    let obj = get_obj(&interp, "my_action");
    assert_num_field(&obj, "priority", 5.0);
}

// ── 5. Task keyword removed (#497) — use action instead ─────────────────────

#[test]
fn test_task_replaced_by_action() {
    let interp = run(r#"
        action my_task {
            priority: 1
        }
    "#);

    let obj = get_obj(&interp, "my_task");
    assert_num_field(&obj, "priority", 1.0);
}

// ── 6. Role ─────────────────────────────────────────────────────────────────

#[test]
fn test_role_declaration() {
    let interp = run(r#"
        role Fighter {
            strength: 10
        }
    "#);

    let obj = get_obj(&interp, "Fighter");
    assert_str_field(&obj, "__type", "role");
    assert_str_field(&obj, "name", "Fighter");
    assert_num_field(&obj, "strength", 10.0);
}

#[test]
fn test_role_with_capabilities() {
    // Note: `can` keyword is a silent pest rule, so capability parsing
    // currently falls through to identifier-based field parsing.
    // This test verifies the role declaration with key-value fields works.
    let interp = run(r#"
        role Admin {
            level: 5
        }
    "#);

    let obj = get_obj(&interp, "Admin");
    assert_str_field(&obj, "__type", "role");
    assert_num_field(&obj, "level", 5.0);
}

// ── 7. Subject ──────────────────────────────────────────────────────────────

#[test]
fn test_subject_declaration_basic() {
    // Basic subject declaration — verifies __type and name are set
    let interp = run(r#"
        subject Player {
            level: 1
        }
    "#);

    let obj = get_obj(&interp, "Player");
    assert_str_field(&obj, "__type", "subject");
    assert_str_field(&obj, "name", "Player");
}

#[test]
fn test_subject_with_fields() {
    // Subject with additional key-value fields
    let interp = run(r#"
        subject Enemy {
            damage: 25,
            speed: 3
        }
    "#);

    let obj = get_obj(&interp, "Enemy");
    assert_str_field(&obj, "__type", "subject");
}

// ── 8. Relation ─────────────────────────────────────────────────────────────

#[test]
fn test_relation_declaration() {
    let interp = run(r#"
        relation A <-> B {
            trust: 50
        }
    "#);

    let obj = get_obj(&interp, "A_B");
    assert_str_field(&obj, "__type", "relation");
    assert_str_field(&obj, "subject_a", "A");
    assert_str_field(&obj, "subject_b", "B");
    assert_num_field(&obj, "trust", 50.0);
}

// ── 9. Effect ───────────────────────────────────────────────────────────────

#[test]
fn test_effect_declaration() {
    let interp = run(r#"
        effect on Damage() {
            let x = 1;
        }
    "#);

    let obj = get_obj(&interp, "effect_on_Damage");
    assert_str_field(&obj, "__type", "effect");
    assert_str_field(&obj, "event", "Damage");
}

// ── 10. Store ───────────────────────────────────────────────────────────────

#[test]
fn test_store_declaration() {
    let interp = run(r#"
        store my_store {
            dimensions: 128
        }
    "#);

    let obj = get_obj(&interp, "my_store");
    assert_str_field(&obj, "__type", "store");
    assert_str_field(&obj, "name", "my_store");
    assert_num_field(&obj, "dimensions", 128.0);
}

// ── 11. Swarm ───────────────────────────────────────────────────────────────

#[test]
fn test_swarm_declaration() {
    let interp = run(r#"
        swarm MySwarm {
            agents: ["A", "B"],
            strategy: parallel
        }
    "#);

    let obj = get_obj(&interp, "MySwarm");
    assert_str_field(&obj, "name", "MySwarm");
    assert_str_field(&obj, "strategy", "parallel");

    match obj.get("agents") {
        Some(v) => {
            if let Some(arr) = v.as_array() {
                assert_eq!(arr.len(), 2);
                let a = arr[0]
                    .as_str()
                    .unwrap_or_else(|| panic!("Expected String element"));
                let b = arr[1]
                    .as_str()
                    .unwrap_or_else(|| panic!("Expected String element"));
                assert_eq!(a, "A");
                assert_eq!(b, "B");
            } else {
                panic!("Expected Array for 'agents', got {:?}", v);
            }
        }
        None => panic!("Expected Array for 'agents', got None"),
    }
}

// ── 12. Council ─────────────────────────────────────────────────────────────

#[test]
fn test_council_declaration() {
    let interp = run(r#"
        council MyCouncil {
            constitution: "c1"
        }
    "#);

    let obj = get_obj(&interp, "MyCouncil");
    assert_str_field(&obj, "name", "MyCouncil");
    assert_str_field(&obj, "constitution", "c1");
}

// ── 13. Import (ES6) — parse-only ───────────────────────────────────────────

#[test]
fn test_import_es6_parse_succeeds() {
    let source = r#"import math from "./test_module""#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "ES6 import should parse successfully, got: {:?}",
        result.err()
    );
}

// ── 14. Constitution — parse-only ───────────────────────────────────────────

#[test]
fn test_constitution_parse_succeeds() {
    let source = r#"
        constitution BasicRules {
            description: "Test constitution"
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Constitution should parse successfully, got: {:?}",
        result.err()
    );
}

// ── 15. Mixed declarations in one program ───────────────────────────────────

#[test]
fn test_mixed_declarations() {
    // Fix #489: tool_decl removed — tool part of this test was removed
    let interp = run(r#"
        resource config_res {
            server: "local",
            uri: "file://config.json"
        }

        provider my_llm {
            type: "openai",
            model: "gpt-4"
        }

        action deploy {
            priority: 10
        }

        role Admin {
            level: 99
        }

        subject System {
            speed: 1
        }

        store knowledge {
            dimensions: 256
        }
    "#);

    // Verify all declarations are accessible
    let res_obj = get_obj(&interp, "config_res");
    assert_str_field(&res_obj, "uri", "file://config.json");

    let prov_obj = get_obj(&interp, "my_llm");
    assert_str_field(&prov_obj, "model", "gpt-4");

    let action_obj = get_obj(&interp, "deploy");
    assert_num_field(&action_obj, "priority", 10.0);

    let role_obj = get_obj(&interp, "Admin");
    assert_str_field(&role_obj, "__type", "role");
    assert_num_field(&role_obj, "level", 99.0);

    let subj_obj = get_obj(&interp, "System");
    assert_str_field(&subj_obj, "__type", "subject");

    let store_obj = get_obj(&interp, "knowledge");
    assert_str_field(&store_obj, "__type", "store");
    assert_num_field(&store_obj, "dimensions", 256.0);
}
