use chrono::Utc;
use hudhudscript_governance::*;
use std::collections::HashMap;

#[test]
fn test_constitution_creation() {
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test Constitution".to_string(),
        description: Some("A test constitution".to_string()),
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    assert_eq!(constitution.id, "cons.1");
    assert_eq!(constitution.name, "Test Constitution");
    assert_eq!(constitution.version, 1);
}

#[test]
fn test_law_creation() {
    let law = Law {
        id: "cons1.law1".to_string(),
        constitution_id: "cons.1".to_string(),
        name: "Test Law".to_string(),
        description: "A test law".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    };

    assert_eq!(law.id, "cons1.law1");
    assert_eq!(law.constitution_id, "cons.1");
    assert_eq!(law.enforcement_level, EnforcementLevel::Mandatory);
}

#[test]
fn test_rule_creation() {
    let rule = Rule {
        id: "rule.1".to_string(),
        name: "Test Rule".to_string(),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    };

    assert_eq!(rule.id, "rule.1");
    assert_eq!(rule.priority, 10);
    assert_eq!(rule.actions.len(), 1);
}

#[test]
fn test_council_creation() {
    let council = Council {
        id: "council1".to_string(),
        name: "Test Council".to_string(),
        constitution_id: "cons.1".to_string(),
        members: vec![],
        rules: vec![],
        state: CouncilState {
            active: true,
            metadata: HashMap::new(),
        },
        governance_model: None,
    };

    assert_eq!(council.id, "council1");
    assert_eq!(council.constitution_id, "cons.1");
    assert!(council.state.active);
}

#[test]
fn test_agent_member_creation() {
    let member = AgentMember {
        agent_id: "agent1".to_string(),
        role: AgentRole::Prosecutor,
        joined_at: Utc::now(),
        permissions: vec!["read".to_string(), "write".to_string()],
    };

    assert_eq!(member.agent_id, "agent1");
    assert_eq!(member.role, AgentRole::Prosecutor);
    assert_eq!(member.permissions.len(), 2);
}

#[test]
fn test_swarm_creation() {
    let swarm = Swarm {
        id: "swarm1".to_string(),
        name: "Test Swarm".to_string(),
        agents: vec!["agent1".to_string(), "agent2".to_string()],
        coordination_strategy: CoordinationStrategy::Parallel,
        shared_state: HashMap::new(),
    };

    assert_eq!(swarm.id, "swarm1");
    assert_eq!(swarm.agents.len(), 2);
    assert_eq!(swarm.coordination_strategy, CoordinationStrategy::Parallel);
}

#[test]
fn test_community_creation() {
    let community = Community {
        id: "community1".to_string(),
        name: "Test Community".to_string(),
        members: vec!["agent1".to_string()],
        councils: vec!["council1".to_string()],
        shared_resources: vec![],
        culture: CommunityCulture {
            values: vec!["collaboration".to_string()],
            norms: vec!["respect".to_string()],
            communication_style: CommunicationStyle::Collaborative,
        },
    };

    assert_eq!(community.id, "community1");
    assert_eq!(community.members.len(), 1);
    assert_eq!(
        community.culture.communication_style,
        CommunicationStyle::Collaborative
    );
}

#[test]
fn test_condition_types() {
    let equals = Condition::Equals {
        field: "status".to_string(),
        value: serde_json::json!("active"),
    };

    let greater_than = Condition::GreaterThan {
        field: "count".to_string(),
        value: 10.0,
    };

    let between = Condition::Between {
        field: "age".to_string(),
        min: 18.0,
        max: 65.0,
    };

    // Just verify they can be created
    assert!(matches!(equals, Condition::Equals { .. }));
    assert!(matches!(greater_than, Condition::GreaterThan { .. }));
    assert!(matches!(between, Condition::Between { .. }));
}

#[test]
fn test_action_types() {
    let actions = [
        Action::Allow,
        Action::Deny,
        Action::Require {
            permission: "admin".to_string(),
        },
        Action::Execute {
            task: "validate".to_string(),
        },
        Action::Notify {
            message: "Action logged".to_string(),
        },
    ];

    assert_eq!(actions.len(), 5);
}

#[test]
fn test_serialization() {
    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Test".to_string(),
        description: None,
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    };

    let json = serde_json::to_string(&constitution).unwrap();
    let deserialized: Constitution = serde_json::from_str(&json).unwrap();

    assert_eq!(constitution.id, deserialized.id);
    assert_eq!(constitution.name, deserialized.name);
}

// ============================================================================
// Governance Model Tests
// ============================================================================

#[test]
fn test_governance_model_democracy() {
    let model = GovernanceModel::democracy();

    assert_eq!(model.model_type, GovernanceModelType::Democracy);
    assert_eq!(model.constitution_compliance, 1.0);
    assert_eq!(model.law_flexibility, 0.0);
    assert_eq!(model.rule_enforcement, 1.0);
    assert!(matches!(
        model.decision_strategy,
        DecisionStrategy::Majority
    ));
}

#[test]
fn test_governance_model_monarchy() {
    let model = GovernanceModel::monarchy("king".to_string());

    assert_eq!(model.model_type, GovernanceModelType::Monarchy);
    assert_eq!(model.constitution_compliance, 0.0);
    assert_eq!(model.law_flexibility, 1.0);
    assert_eq!(model.rule_enforcement, 0.5);

    let privileges = model.has_privileges("king").unwrap();
    assert!(privileges.bypass_constitution);
    assert!(privileges.modify_laws);
    assert!(privileges.override_rules);
    assert_eq!(privileges.voting_weight, 10.0);
}

#[test]
fn test_governance_model_parliamentary() {
    let model = GovernanceModel::parliamentary();

    assert_eq!(model.model_type, GovernanceModelType::Parliamentary);
    assert_eq!(model.constitution_compliance, 1.0);
    assert_eq!(model.law_flexibility, 0.2);
    assert_eq!(model.rule_enforcement, 0.9);
}

#[test]
fn test_governance_model_technocracy() {
    let model = GovernanceModel::technocracy();

    assert_eq!(model.model_type, GovernanceModelType::Technocracy);
    assert_eq!(model.constitution_compliance, 0.8);
    assert_eq!(model.law_flexibility, 0.4);
    assert_eq!(model.rule_enforcement, 0.95);

    match model.decision_strategy {
        DecisionStrategy::DataDriven { ref metric } => {
            assert_eq!(metric, "efficiency");
        }
        _ => panic!("Expected DataDriven strategy"),
    }
}

#[test]
fn test_governance_model_anarchy() {
    let model = GovernanceModel::anarchy();

    assert_eq!(model.model_type, GovernanceModelType::Anarchy);
    assert_eq!(model.constitution_compliance, 0.0);
    assert_eq!(model.law_flexibility, 1.0);
    assert_eq!(model.rule_enforcement, 0.0);
}

#[test]
fn test_governance_model_chaos() {
    let model = GovernanceModel::chaos();

    assert_eq!(model.model_type, GovernanceModelType::Chaos);
    // Chaos has random coefficients, just check they're in valid range
    assert!(model.constitution_compliance >= 0.0 && model.constitution_compliance <= 1.0);
    assert!(model.law_flexibility >= 0.0 && model.law_flexibility <= 1.0);
    assert!(model.rule_enforcement >= 0.0 && model.rule_enforcement <= 1.0);
}

#[test]
fn test_governance_model_coefficient_clamping() {
    let model = GovernanceModel::new(
        GovernanceModelType::Hybrid,
        1.5,  // Should be clamped to 1.0
        -0.5, // Should be clamped to 0.0
        0.5,
        DecisionStrategy::Consensus,
    );

    assert_eq!(model.constitution_compliance, 1.0);
    assert_eq!(model.law_flexibility, 0.0);
    assert_eq!(model.rule_enforcement, 0.5);
}

#[test]
fn test_governance_model_add_special_role() {
    let mut model = GovernanceModel::democracy();

    model.add_special_role(
        "president".to_string(),
        RolePrivileges {
            bypass_constitution: false,
            modify_laws: true,
            override_rules: false,
            voting_weight: 2.0,
        },
    );

    let privileges = model.has_privileges("president").unwrap();
    assert!(!privileges.bypass_constitution);
    assert!(privileges.modify_laws);
    assert_eq!(privileges.voting_weight, 2.0);
}

#[test]
fn test_governance_model_default() {
    let model = GovernanceModel::default();

    // Default should be democracy
    assert_eq!(model.model_type, GovernanceModelType::Democracy);
    assert_eq!(model.constitution_compliance, 1.0);
}

#[test]
fn test_decision_strategy_types() {
    let strategies = [
        DecisionStrategy::Majority,
        DecisionStrategy::Supermajority,
        DecisionStrategy::Unanimous,
        DecisionStrategy::SingleAuthority {
            authority_role: "leader".to_string(),
        },
        DecisionStrategy::WeightedVoting,
        DecisionStrategy::Consensus,
        DecisionStrategy::Random,
        DecisionStrategy::DataDriven {
            metric: "performance".to_string(),
        },
    ];

    // Just verify all variants can be created
    assert_eq!(strategies.len(), 8);
}

#[test]
fn test_governance_model_serialization() {
    let model = GovernanceModel::monarchy("emperor".to_string());

    let json = serde_json::to_string(&model).unwrap();
    let deserialized: GovernanceModel = serde_json::from_str(&json).unwrap();

    assert_eq!(model.model_type, deserialized.model_type);
    assert_eq!(
        model.constitution_compliance,
        deserialized.constitution_compliance
    );
    assert_eq!(model.law_flexibility, deserialized.law_flexibility);
}

#[test]
fn test_council_with_governance_model() {
    let model = GovernanceModel::parliamentary();

    let council = Council {
        id: "council1".to_string(),
        name: "Parliament".to_string(),
        constitution_id: "cons.1".to_string(),
        members: vec![],
        rules: vec![],
        state: CouncilState {
            active: true,
            metadata: HashMap::new(),
        },
        governance_model: Some(model.clone()),
    };

    assert!(council.governance_model.is_some());
    let council_model = council.governance_model.unwrap();
    assert_eq!(council_model.model_type, GovernanceModelType::Parliamentary);
}

#[test]
fn test_role_privileges_creation() {
    let privileges = RolePrivileges {
        bypass_constitution: true,
        modify_laws: true,
        override_rules: false,
        voting_weight: 5.0,
    };

    assert!(privileges.bypass_constitution);
    assert!(privileges.modify_laws);
    assert!(!privileges.override_rules);
    assert_eq!(privileges.voting_weight, 5.0);
}
