//! End-to-end governance system tests

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

#[test]
fn test_simple_governance_structures() {
    let source = r#"
        constitution TestConstitution {
            description: "Test",
            laws: []
        }
        
        law TestLaw {
            description: "Test law",
            enforcement: mandatory,
            rules: []
        }
        
        council TestCouncil {
            constitution: "test",
            members: [],
            rules: []
        }
        
        rule TestRule: {
            conditions: [],
            actions: [],
            priority: 10
        }
        
        swarm TestSwarm {
            agents: ["agent1", "agent2"],
            strategy: parallel
        }
        
        community TestCommunity: {
            members: ["agent1"],
            councils: [],
            culture: {
                values: ["test"],
                norms: ["test"],
                communication_style: formal
            }
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());

    // Verify all structures were created
    assert!(interpreter.get_variable("TestConstitution").is_ok());
    assert!(interpreter.get_variable("TestLaw").is_ok());
    assert!(interpreter.get_variable("TestCouncil").is_ok());
    assert!(interpreter.get_variable("TestRule").is_ok());
    assert!(interpreter.get_variable("TestSwarm").is_ok());
    assert!(interpreter.get_variable("TestCommunity").is_ok());
}

#[test]
fn test_complete_governance_workflow() {
    let source = r#"
        constitution DataGovernance {
            description: "Data governance framework",
            laws: [
                {
                    name: "Privacy",
                    description: "Privacy protection",
                    enforcement: mandatory,
                    rules: ["rule1"]
                }
            ]
        }
        
        rule PrivacyRule: {
            conditions: [
                { type: "equals", field: "data_type", value: "personal" }
            ],
            actions: [
                { type: "require", permission: "consent" }
            ],
            priority: 100
        }
        
        council GovernanceCouncil {
            constitution: "DataGovernance",
            members: [
                { agent: "judge1", role: "Judge" },
                { agent: "executor1", role: "Executor" }
            ],
            rules: ["rule1"]
        }
        
        swarm ProcessingSwarm {
            agents: ["agent1", "agent2", "agent3"],
            strategy: parallel
        }
        
        community DataCommunity: {
            members: ["judge1", "executor1", "agent1"],
            councils: ["GovernanceCouncil"],
            culture: {
                values: ["transparency", "quality"],
                norms: ["testing", "documentation"],
                communication_style: technical
            }
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());

    // Verify constitution
    let constitution = interpreter.get_variable("DataGovernance").unwrap();
    if let Some(obj) = constitution.as_object() {
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("description"));
        assert!(obj.contains_key("laws"));
    } else {
        panic!("Expected constitution object");
    }

    // Verify rule
    let rule = interpreter.get_variable("PrivacyRule").unwrap();
    if let Some(obj) = rule.as_object() {
        assert!(obj.contains_key("priority"));
        assert!(obj.contains_key("conditions"));
        assert!(obj.contains_key("actions"));
    } else {
        panic!("Expected rule object");
    }

    // Verify council
    let council = interpreter.get_variable("GovernanceCouncil").unwrap();
    if let Some(obj) = council.as_object() {
        assert!(obj.contains_key("constitution"));
        assert!(obj.contains_key("members"));
    } else {
        panic!("Expected council object");
    }

    // Verify swarm
    let swarm = interpreter.get_variable("ProcessingSwarm").unwrap();
    if let Some(obj) = swarm.as_object() {
        assert!(obj.contains_key("agents"));
        assert!(obj.contains_key("strategy"));
    } else {
        panic!("Expected swarm object");
    }

    // Verify community
    let community = interpreter.get_variable("DataCommunity").unwrap();
    if let Some(obj) = community.as_object() {
        assert!(obj.contains_key("members"));
        assert!(obj.contains_key("councils"));
        assert!(obj.contains_key("culture"));
    } else {
        panic!("Expected community object");
    }
}

#[test]
fn test_multilang_turkish_governance() {
    let source = r#"
        anayasa TestAnayasa {
            description: "Test anayasa",
            laws: []
        }

        konsey TestKonsey {
            constitution: "test",
            members: [],
            rules: []
        }

        sürü TestSürü {
            agents: ["ajan1", "ajan2"],
            strategy: paralel
        }
    "#;

    let statements = parse(source).expect("Failed to parse Turkish");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(
        result.is_ok(),
        "Failed to execute Turkish: {:?}",
        result.err()
    );

    assert!(interpreter.get_variable("TestAnayasa").is_ok());
    assert!(interpreter.get_variable("TestKonsey").is_ok());
    assert!(interpreter.get_variable("TestSürü").is_ok());
}

#[test]
fn test_complex_rules_and_conditions() {
    let source = r#"
        rule ComplexRule: {
            conditions: [
                { type: "equals", field: "status", value: "active" },
                { type: "greater_than", field: "priority", value: 50 }
            ],
            actions: [
                { type: "allow" },
                { type: "notify", recipient: "admin" },
                { type: "execute", function: "validate" }
            ],
            priority: 75
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok());

    let rule = interpreter.get_variable("ComplexRule").unwrap();
    if let Some(obj) = rule.as_object() {
        // Check conditions array
        if let Some(conditions) = obj.get("conditions").and_then(|v| v.as_array()) {
            assert_eq!(conditions.len(), 2);
        } else {
            panic!("Expected conditions array");
        }

        // Check actions array
        if let Some(actions) = obj.get("actions").and_then(|v| v.as_array()) {
            assert_eq!(actions.len(), 3);
        } else {
            panic!("Expected actions array");
        }

        // Check priority
        if let Some(v) = obj.get("priority") {
            if let Some(priority) = v.as_number() {
                assert_eq!(priority, 75.0);
            } else {
                panic!("Expected priority number");
            }
        } else {
            panic!("Expected priority number");
        }
    } else {
        panic!("Expected rule object");
    }
}

#[test]
fn test_nested_culture_in_community() {
    let source = r#"
        community RichCommunity: {
            members: ["agent1", "agent2", "agent3"],
            councils: ["council1", "council2"],
            culture: {
                values: ["innovation", "collaboration", "excellence"],
                norms: ["code review", "testing", "documentation", "security"],
                communication_style: collaborative
            }
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok());

    let community = interpreter.get_variable("RichCommunity").unwrap();
    if let Some(obj) = community.as_object() {
        // Check culture object
        if let Some(culture) = obj.get("culture").and_then(|v| v.as_object()) {
            // Check values array
            if let Some(values) = culture.get("values").and_then(|v| v.as_array()) {
                assert_eq!(values.len(), 3);
            } else {
                panic!("Expected values array");
            }

            // Check norms array
            if let Some(norms) = culture.get("norms").and_then(|v| v.as_array()) {
                assert_eq!(norms.len(), 4);
            } else {
                panic!("Expected norms array");
            }

            // Check communication style
            assert!(culture.contains_key("communication_style"));
        } else {
            panic!("Expected culture object");
        }
    } else {
        panic!("Expected community object");
    }
}
