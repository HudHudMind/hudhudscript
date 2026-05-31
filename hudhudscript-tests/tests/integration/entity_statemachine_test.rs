// Tests for grammar rules that were defined in base.pest but previously
// unimplemented in the parser (GitHub issue #157).
//
// Covered rules: entity_decl, statemachine_decl, event_decl,
//                contract_decl, treaty_decl

use hudhudscript_parser::parse;

// ============================================================================
// entity_decl
// ============================================================================

#[test]
fn test_parse_entity_with_data_fields() {
    let source = r#"
        entity Player {
            data health: Number = 100
            data name: String
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse entity declaration: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    assert!(!stmts.is_empty(), "No statements parsed from entity decl");
}

#[test]
fn test_parse_entity_empty_body() {
    let source = "entity Empty {}";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse empty entity declaration: {:?}",
        result.err()
    );
}

// ============================================================================
// statemachine_decl
// ============================================================================

#[test]
fn test_parse_statemachine() {
    let source = r#"
        statemachine TrafficLight {
            state Red {
                on event(timer) -> Green,
            }
            state Green {
                on event(timer) -> Yellow,
            }
            state Yellow {
                on event(timer) -> Red,
            }
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse statemachine declaration: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    assert!(
        !stmts.is_empty(),
        "No statements parsed from statemachine decl"
    );
}

#[test]
fn test_parse_statemachine_empty() {
    let source = "statemachine Empty {}";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse empty statemachine: {:?}",
        result.err()
    );
}

// ============================================================================
// event_decl
// ============================================================================

#[test]
fn test_parse_event() {
    let source = r#"
        event UserCreated {
            userId: String,
            email: String,
            timestamp: Number,
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse event declaration: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    assert!(!stmts.is_empty(), "No statements parsed from event decl");
}

#[test]
fn test_parse_event_empty_body() {
    let source = "event Ping {}";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse empty event declaration: {:?}",
        result.err()
    );
}

// ============================================================================
// contract_decl
// ============================================================================

#[test]
fn test_parse_contract() {
    let source = r#"
        contract ServiceAgreement: {
            parties: ["AgentA", "AgentB"],
            duration: 365,
            terms: "Standard service terms apply",
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse contract declaration: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    assert!(!stmts.is_empty(), "No statements parsed from contract decl");
}

#[test]
fn test_parse_contract_empty() {
    let source = "contract EmptyContract: {}";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse empty contract: {:?}",
        result.err()
    );
}

// ============================================================================
// treaty_decl
// ============================================================================

#[test]
fn test_parse_treaty() {
    let source = r#"
        treaty PeaceTreaty: {
            signatories: ["NationA", "NationB"],
            effective: "2024-01-01",
            duration: "indefinite",
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse treaty declaration: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    assert!(!stmts.is_empty(), "No statements parsed from treaty decl");
}

#[test]
fn test_parse_treaty_empty() {
    let source = "treaty EmptyTreaty: {}";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse empty treaty: {:?}",
        result.err()
    );
}

// ============================================================================
// Combination: multiple formerly-unimplemented rules in one program
// ============================================================================

#[test]
fn test_parse_mixed_previously_unimplemented_rules() {
    let source = r#"
        entity Sensor {
            data value: Number = 0
        }

        event SensorUpdated {
            sensorId: String,
            newValue: Number,
        }

        statemachine SensorFSM {
            state Idle {
                on event(activate) -> Active,
            }
            state Active {
                on event(deactivate) -> Idle,
            }
        }

        contract DataSharingAgreement: {
            owner: "AgentA",
            consumer: "AgentB",
        }

        treaty DataProtocolTreaty: {
            parties: ["AgentA", "AgentB"],
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse mixed previously-unimplemented rules: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    assert_eq!(
        stmts.len(),
        5,
        "Expected 5 top-level statements, got {}",
        stmts.len()
    );
}
