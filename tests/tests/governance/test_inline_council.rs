use hudhudscript_governance::council::{CouncilBuilder, CouncilError, CouncilManager};
use hudhudscript_governance::*;

#[test]
fn test_council_builder_basic() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    assert_eq!(council.id, "council1");
    assert_eq!(council.name, "Test Council");
    assert_eq!(council.constitution_id, "cons.1");
    assert_eq!(council.members.len(), 0);
    assert_eq!(council.rules.len(), 0);
    assert!(council.state.active);
}

#[test]
fn test_council_builder_with_members() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_member(
        "agent1".to_string(),
        AgentRole::Prosecutor,
        vec!["read".to_string()],
    )
    .unwrap()
    .add_member(
        "agent2".to_string(),
        AgentRole::Judge,
        vec!["read".to_string(), "write".to_string()],
    )
    .unwrap()
    .build()
    .unwrap();

    assert_eq!(council.members.len(), 2);
    assert_eq!(council.members[0].agent_id, "agent1");
    assert_eq!(council.members[0].role, AgentRole::Prosecutor);
    assert_eq!(council.members[1].agent_id, "agent2");
    assert_eq!(council.members[1].role, AgentRole::Judge);
}

#[test]
fn test_council_builder_duplicate_agent() {
    let result = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_member(
        "agent1".to_string(),
        AgentRole::Prosecutor,
        vec!["read".to_string()],
    )
    .unwrap()
    .add_member(
        "agent1".to_string(),
        AgentRole::Judge,
        vec!["write".to_string()],
    );

    assert!(matches!(result, Err(CouncilError::DuplicateAgent(_))));
}

#[test]
fn test_council_builder_with_rules() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_rule("rule.1".to_string())
    .add_rule("rule.2".to_string())
    .build()
    .unwrap();

    assert_eq!(council.rules.len(), 2);
    assert_eq!(council.rules[0], "rule.1");
    assert_eq!(council.rules[1], "rule.2");
}

#[test]
fn test_council_builder_with_constitution_validator() {
    let result = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.999".to_string(),
    )
    .with_constitution_validator(|id| id == "cons.1")
    .build();

    assert!(matches!(result, Err(CouncilError::ConstitutionNotFound(_))));
}

#[test]
fn test_council_builder_valid_constitution() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .with_constitution_validator(|id| id == "cons.1")
    .build()
    .unwrap();

    assert_eq!(council.constitution_id, "cons.1");
}

#[test]
fn test_council_manager_add_member() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);

    manager
        .add_member(
            "agent1".to_string(),
            AgentRole::Member,
            vec!["read".to_string()],
        )
        .unwrap();

    assert_eq!(manager.council().members.len(), 1);
    assert!(manager.has_member(&"agent1".to_string()));
}

#[test]
fn test_council_manager_remove_member() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_member(
        "agent1".to_string(),
        AgentRole::Member,
        vec!["read".to_string()],
    )
    .unwrap()
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);

    manager.remove_member(&"agent1".to_string()).unwrap();
    assert_eq!(manager.council().members.len(), 0);
}

#[test]
fn test_council_manager_update_role() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_member(
        "agent1".to_string(),
        AgentRole::Member,
        vec!["read".to_string()],
    )
    .unwrap()
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);

    manager
        .update_member_role(&"agent1".to_string(), AgentRole::Prosecutor)
        .unwrap();

    let member = manager.get_member(&"agent1".to_string()).unwrap();
    assert_eq!(member.role, AgentRole::Prosecutor);
}

#[test]
fn test_council_manager_update_permissions() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_member(
        "agent1".to_string(),
        AgentRole::Member,
        vec!["read".to_string()],
    )
    .unwrap()
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);

    manager
        .update_member_permissions(
            &"agent1".to_string(),
            vec![
                "read".to_string(),
                "write".to_string(),
                "execute".to_string(),
            ],
        )
        .unwrap();

    let member = manager.get_member(&"agent1".to_string()).unwrap();
    assert_eq!(member.permissions.len(), 3);
}

#[test]
fn test_council_manager_get_members_by_role() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_member(
        "agent1".to_string(),
        AgentRole::Prosecutor,
        vec!["read".to_string()],
    )
    .unwrap()
    .add_member(
        "agent2".to_string(),
        AgentRole::Judge,
        vec!["read".to_string()],
    )
    .unwrap()
    .add_member(
        "agent3".to_string(),
        AgentRole::Prosecutor,
        vec!["read".to_string()],
    )
    .unwrap()
    .build()
    .unwrap();

    let manager = CouncilManager::new(council);

    let prosecutors = manager.get_members_by_role(&AgentRole::Prosecutor);
    assert_eq!(prosecutors.len(), 2);

    let judges = manager.get_members_by_role(&AgentRole::Judge);
    assert_eq!(judges.len(), 1);
}

#[test]
fn test_council_manager_add_remove_rules() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);

    manager.add_rule("rule.1".to_string());
    manager.add_rule("rule.2".to_string());
    assert_eq!(manager.council().rules.len(), 2);

    manager.remove_rule(&"rule.1".to_string());
    assert_eq!(manager.council().rules.len(), 1);
    assert_eq!(manager.council().rules[0], "rule.2");
}

#[test]
fn test_council_manager_active_state() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);

    assert!(manager.is_active());

    manager.set_active(false);
    assert!(!manager.is_active());

    manager.set_active(true);
    assert!(manager.is_active());
}

#[test]
fn test_council_manager_validate_unique_agents() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_member(
        "agent1".to_string(),
        AgentRole::Member,
        vec!["read".to_string()],
    )
    .unwrap()
    .add_member(
        "agent2".to_string(),
        AgentRole::Member,
        vec!["read".to_string()],
    )
    .unwrap()
    .build()
    .unwrap();

    let manager = CouncilManager::new(council);
    assert!(manager.validate_unique_agents().is_ok());
}

#[test]
fn test_council_builder_add_members_batch() {
    let members = vec![
        (
            "agent1".to_string(),
            AgentRole::Prosecutor,
            vec!["read".to_string()],
        ),
        (
            "agent2".to_string(),
            AgentRole::Judge,
            vec!["write".to_string()],
        ),
        (
            "agent3".to_string(),
            AgentRole::Executor,
            vec!["execute".to_string()],
        ),
    ];

    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_members(members)
    .unwrap()
    .build()
    .unwrap();

    assert_eq!(council.members.len(), 3);
}

#[test]
fn test_council_error_display() {
    let e1 = CouncilError::ConstitutionNotFound("cons.1".to_string());
    assert!(format!("{}", e1).contains("Constitution not found"));
    assert!(format!("{}", e1).contains("cons.1"));

    let e2 = CouncilError::DuplicateAgent("agent1".to_string());
    assert!(format!("{}", e2).contains("Duplicate agent"));
    assert!(format!("{}", e2).contains("agent1"));

    let e3 = CouncilError::AgentNotFound("agent2".to_string());
    assert!(format!("{}", e3).contains("Agent not found"));
    assert!(format!("{}", e3).contains("agent2"));

    let e4 = CouncilError::InvalidRole("bad_role".to_string());
    assert!(format!("{}", e4).contains("Invalid role"));
    assert!(format!("{}", e4).contains("bad_role"));
}

#[test]
fn test_council_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(CouncilError::AgentNotFound("a".to_string()));
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_council_manager_remove_nonexistent_member() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);
    let result = manager.remove_member(&"nonexistent".to_string());
    assert!(matches!(result, Err(CouncilError::AgentNotFound(_))));
}

#[test]
fn test_council_manager_update_role_nonexistent() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);
    let result = manager.update_member_role(&"nonexistent".to_string(), AgentRole::Judge);
    assert!(matches!(result, Err(CouncilError::AgentNotFound(_))));
}

#[test]
fn test_council_manager_update_permissions_nonexistent() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);
    let result =
        manager.update_member_permissions(&"nonexistent".to_string(), vec!["read".to_string()]);
    assert!(matches!(result, Err(CouncilError::AgentNotFound(_))));
}

#[test]
fn test_council_manager_has_member_false() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let manager = CouncilManager::new(council);
    assert!(!manager.has_member(&"nobody".to_string()));
}

#[test]
fn test_council_manager_get_member_none() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let manager = CouncilManager::new(council);
    assert!(manager.get_member(&"nobody".to_string()).is_none());
}

#[test]
fn test_council_manager_council_mut() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);
    manager.council_mut().name = "Updated Name".to_string();
    assert_eq!(manager.council().name, "Updated Name");
}

#[test]
fn test_council_manager_add_duplicate_member() {
    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_member("agent1".to_string(), AgentRole::Member, vec![])
    .unwrap()
    .build()
    .unwrap();

    let mut manager = CouncilManager::new(council);
    let result = manager.add_member("agent1".to_string(), AgentRole::Judge, vec![]);
    assert!(matches!(result, Err(CouncilError::DuplicateAgent(_))));
}

#[test]
fn test_council_builder_add_rules_batch() {
    let rules = vec![
        "rule.1".to_string(),
        "rule.2".to_string(),
        "rule.3".to_string(),
    ];

    let council = CouncilBuilder::new(
        "council1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .add_rules(rules)
    .build()
    .unwrap();

    assert_eq!(council.rules.len(), 3);
}
