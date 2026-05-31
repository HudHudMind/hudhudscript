//! Integration tests for governance system in interpreter

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;
use std::collections::HashMap;

#[test]
fn test_interpret_constitution() {
    let source = r#"
        constitution DataGovernance {
            description: "Data handling rules",
            laws: [
                {
                    name: "Privacy Law",
                    description: "Protect user privacy",
                    enforcement: mandatory,
                    rules: ["rule.1", "rule.2"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());

    // Check that constitution is stored in environment
    let const_val = interpreter.get_variable("DataGovernance");
    assert!(
        const_val.is_ok(),
        "Failed to get constitution: {:?}",
        const_val.err()
    );

    if let Ok(val) = const_val {
        if let Some(obj) = val.as_object() {
            assert_eq!(
                obj.get("name"),
                Some(&Value16::string("DataGovernance".to_string()))
            );
            assert_eq!(
                obj.get("description"),
                Some(&Value16::string("Data handling rules".to_string()))
            );

            if let Some(v) = obj.get("laws") {
                if let Some(laws) = v.as_array() {
                    assert_eq!(laws.len(), 1);
                } else {
                    panic!("Expected laws array");
                }
            } else {
                panic!("Expected laws array");
            }
        } else {
            panic!("Expected constitution object");
        }
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_interpret_law() {
    let source = r#"
        law PrivacyLaw {
            description: "Ensure user data privacy",
            enforcement: mandatory,
            rules: ["rule.1", "rule.2"]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok());

    let law_val = interpreter.get_variable("PrivacyLaw");
    assert!(law_val.is_ok());

    if let Ok(val) = law_val {
        if let Some(obj) = val.as_object() {
            assert_eq!(
                obj.get("name"),
                Some(&Value16::string("PrivacyLaw".to_string()))
            );
            assert_eq!(
                obj.get("enforcement_level"),
                Some(&Value16::string("mandatory".to_string()))
            );
        }
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_interpret_council() {
    let source = r#"
        council SecurityCouncil {
            constitution: "cons.1",
            members: [
                { agent: "agent.1", role: "Judge" },
                { agent: "agent.2", role: "Executor" }
            ],
            rules: ["rule.1"]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok());

    let council_val = interpreter.get_variable("SecurityCouncil");
    assert!(council_val.is_ok());

    if let Ok(val) = council_val {
        if let Some(obj) = val.as_object() {
            assert_eq!(
                obj.get("constitution"),
                Some(&Value16::string("cons.1".to_string()))
            );

            if let Some(v) = obj.get("members") {
                if let Some(members) = v.as_array() {
                    assert_eq!(members.len(), 2);
                }
            }
        }
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_interpret_rule() {
    let source = r#"
        rule AccessControl: {
            conditions: [
                { type: "equals", field: "role", value: "admin" }
            ],
            actions: [
                { type: "allow" }
            ],
            priority: 10
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok());

    let rule_val = interpreter.get_variable("AccessControl");
    assert!(rule_val.is_ok());

    if let Ok(val) = rule_val {
        if let Some(obj) = val.as_object() {
            assert_eq!(obj.get("priority"), Some(&Value16::number(10.0)));

            if let Some(v) = obj.get("conditions") {
                if let Some(conditions) = v.as_array() {
                    assert_eq!(conditions.len(), 1);
                }
            }

            if let Some(v) = obj.get("actions") {
                if let Some(actions) = v.as_array() {
                    assert_eq!(actions.len(), 1);
                }
            }
        }
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_interpret_swarm() {
    let source = r#"
        swarm DataProcessors {
            agents: ["agent.1", "agent.2", "agent.3"],
            strategy: parallel
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok());

    let swarm_val = interpreter.get_variable("DataProcessors");
    assert!(swarm_val.is_ok());

    if let Ok(val) = swarm_val {
        if let Some(obj) = val.as_object() {
            assert_eq!(
                obj.get("strategy"),
                Some(&Value16::string("parallel".to_string()))
            );

            if let Some(v) = obj.get("agents") {
                if let Some(agents) = v.as_array() {
                    assert_eq!(agents.len(), 3);
                }
            }
        }
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_interpret_community() {
    let source = r#"
        community DataScience: {
            members: ["agent.1", "agent.2"],
            councils: ["council.1"],
            culture: {
                values: ["collaboration", "transparency"],
                norms: ["peer review", "open source"],
                communication_style: technical
            }
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok());

    let community_val = interpreter.get_variable("DataScience");
    assert!(community_val.is_ok());

    if let Ok(val) = community_val {
        if let Some(obj) = val.as_object() {
            if let Some(v) = obj.get("members") {
                if let Some(members) = v.as_array() {
                    assert_eq!(members.len(), 2);
                }
            }

            if let Some(v) = obj.get("culture") {
                if let Some(culture) = v.as_object() {
                    assert_eq!(
                        culture.get("communication_style"),
                        Some(&Value16::string("technical".to_string()))
                    );

                    if let Some(v) = culture.get("values") {
                        if let Some(values) = v.as_array() {
                            assert_eq!(values.len(), 2);
                        }
                    }
                }
            }
        }
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_governance_workflow() {
    let source = r#"
        constitution DataGovernance {
            description: "Data handling rules",
            laws: [
                {
                    name: "Privacy Law",
                    description: "Protect user privacy",
                    enforcement: mandatory,
                    rules: []
                }
            ]
        }
        
        council SecurityCouncil {
            constitution: "DataGovernance",
            members: [
                { agent: "agent.1", role: "Judge" }
            ],
            rules: []
        }
        
        swarm DataProcessors {
            agents: ["agent.1", "agent.2"],
            strategy: parallel
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok());

    // Verify all three governance structures are created
    assert!(interpreter.get_variable("DataGovernance").is_ok());
    assert!(interpreter.get_variable("SecurityCouncil").is_ok());
    assert!(interpreter.get_variable("DataProcessors").is_ok());
}
