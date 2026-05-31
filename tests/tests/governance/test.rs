//! Integration tests for governance system parsing

use hudhudscript_ast::{Decl, Stmt};
use hudhudscript_parser::parse;

#[test]
fn test_parse_constitution() {
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

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse constitution: {:?}",
        result.err()
    );

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Constitution {
            name,
            description,
            laws,
            ..
        }) => {
            assert_eq!(name, "DataGovernance");
            assert_eq!(description.as_ref().unwrap(), "Data handling rules");
            assert_eq!(laws.len(), 1);
            assert_eq!(laws[0].name, "Privacy Law");
            assert_eq!(laws[0].enforcement_level, "mandatory");
        }
        _ => panic!("Expected Constitution declaration"),
    }
}

#[test]
fn test_parse_law() {
    let source = r#"
        law PrivacyLaw {
            name: "Privacy Protection",
            description: "Ensure user data privacy",
            enforcement: mandatory,
            rules: ["rule.1", "rule.2"]
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse law: {:?}", result.err());

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Law {
            name,
            description,
            enforcement_level,
            rules,
            ..
        }) => {
            assert_eq!(name, "PrivacyLaw");
            assert_eq!(description, "Ensure user data privacy");
            assert_eq!(enforcement_level, "mandatory");
            assert_eq!(rules.len(), 2);
        }
        _ => panic!("Expected Law declaration"),
    }
}

#[test]
fn test_parse_council() {
    let source = r#"
        council SecurityCouncil {
            name: "Security Council",
            constitution: "cons.1",
            members: [
                { agent: "agent.1", role: "Judge" },
                { agent: "agent.2", role: "Executor" }
            ],
            rules: ["rule.1"]
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse council: {:?}",
        result.err()
    );

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Council {
            name,
            constitution,
            members,
            rules,
            ..
        }) => {
            assert_eq!(name, "SecurityCouncil");
            assert_eq!(constitution, "cons.1");
            assert_eq!(members.len(), 2);
            assert_eq!(members[0].agent_id, "agent.1");
            assert_eq!(members[0].role, "Judge");
            assert_eq!(rules.len(), 1);
        }
        _ => panic!("Expected Council declaration"),
    }
}

#[test]
fn test_parse_rule() {
    let source = r#"
        rule AccessControl: {
            name: "Access Control Rule",
            conditions: [
                { type: "equals", field: "role", value: "admin" }
            ],
            actions: [
                { type: "allow" }
            ],
            priority: 10
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse rule: {:?}", result.err());

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Rule {
            name,
            conditions,
            actions,
            priority,
            ..
        }) => {
            assert_eq!(name, "AccessControl");
            assert_eq!(conditions.len(), 1);
            assert_eq!(conditions[0].condition_type, "equals");
            assert_eq!(conditions[0].field, "role");
            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].action_type, "allow");
            assert_eq!(*priority, 10);
        }
        _ => panic!("Expected Rule declaration"),
    }
}

#[test]
fn test_parse_swarm() {
    let source = r#"
        swarm DataProcessors {
            name: "Data Processing Swarm",
            agents: ["agent.1", "agent.2", "agent.3"],
            strategy: parallel
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse swarm: {:?}", result.err());

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Swarm {
            name,
            agents,
            strategy,
            ..
        }) => {
            assert_eq!(name, "DataProcessors");
            assert_eq!(agents.len(), 3);
            assert_eq!(strategy, "parallel");
        }
        _ => panic!("Expected Swarm declaration"),
    }
}

#[test]
fn test_parse_community() {
    let source = r#"
        community DataScience: {
            name: "Data Science Community",
            members: ["agent.1", "agent.2"],
            councils: ["council.1"],
            culture: {
                values: ["collaboration", "transparency"],
                norms: ["peer review", "open source"],
                communication_style: technical
            }
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse community: {:?}",
        result.err()
    );

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Community {
            name,
            members,
            councils,
            culture,
            ..
        }) => {
            assert_eq!(name, "DataScience");
            assert_eq!(members.len(), 2);
            assert_eq!(councils.len(), 1);
            assert_eq!(culture.values.len(), 2);
            assert_eq!(culture.norms.len(), 2);
            assert_eq!(culture.communication_style, "technical");
        }
        _ => panic!("Expected Community declaration"),
    }
}

#[test]
fn test_parse_multilang_constitution_turkish() {
    let source = r#"
        anayasa VeriYönetişimi {
            description: "Veri işleme kuralları",
            laws: [
                {
                    name: "Gizlilik Yasası",
                    description: "Kullanıcı gizliliğini koru",
                    enforcement: zorunlu,
                    rules: ["kural.1"]
                }
            ]
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Turkish constitution: {:?}",
        result.err()
    );

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Constitution { name, laws, .. }) => {
            assert_eq!(name, "VeriYönetişimi");
            assert_eq!(laws.len(), 1);
            assert_eq!(laws[0].enforcement_level, "mandatory");
        }
        _ => panic!("Expected Constitution declaration"),
    }
}

#[test]
fn test_parse_multilang_swarm_japanese() {
    let source = r#"
        sürü DataProcessors {
            name: "Veri İşleme Sürüsü",
            agents: ["ajan.1", "ajan.2"],
            strategy: paralel
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Turkish swarm: {:?}",
        result.err()
    );

    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Decl(Decl::Swarm {
            name,
            agents,
            strategy,
            ..
        }) => {
            assert_eq!(name, "DataProcessors");
            assert_eq!(agents.len(), 2);
            assert_eq!(strategy, "parallel");
        }
        _ => panic!("Expected Swarm declaration"),
    }
}
