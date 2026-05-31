use hudhudscript_governance::council::{CouncilBuilder, CouncilError, CouncilManager};
use hudhudscript_governance::*;

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilBuilder::new / build
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn builder_basic_id() {
    let council = CouncilBuilder::new("c1".to_string(), "Test".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    assert_eq!(council.id, "c1");
}

#[test]
fn builder_basic_name() {
    let council = CouncilBuilder::new(
        "c1".to_string(),
        "Test Council".to_string(),
        "cons.1".to_string(),
    )
    .build()
    .unwrap();
    assert_eq!(council.name, "Test Council");
}

#[test]
fn builder_basic_constitution_id() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    assert_eq!(council.constitution_id, "cons.1");
}

#[test]
fn builder_basic_empty_members() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    assert_eq!(council.members.len(), 0);
}

#[test]
fn builder_basic_active_by_default() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    assert!(council.state.active);
}

#[test]
fn builder_basic_empty_rules() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    assert_eq!(council.rules.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilBuilder::add_member
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn builder_add_member_agent_id() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member(
            "agent1".to_string(),
            AgentRole::Prosecutor,
            vec!["read".to_string()],
        )
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(council.members[0].agent_id, "agent1");
}

#[test]
fn builder_add_member_role() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("agent1".to_string(), AgentRole::Judge, vec![])
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(council.members[0].role, AgentRole::Judge);
}

#[test]
fn builder_add_member_permissions() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member(
            "a1".to_string(),
            AgentRole::Member,
            vec!["read".to_string(), "write".to_string()],
        )
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(council.members[0].permissions, vec!["read", "write"]);
}

#[test]
fn builder_add_member_duplicate_rejected() {
    let result = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("a1".to_string(), AgentRole::Prosecutor, vec![])
        .unwrap()
        .add_member("a1".to_string(), AgentRole::Judge, vec![]);
    assert!(matches!(result, Err(CouncilError::DuplicateAgent(_))));
}

#[test]
fn builder_add_member_executor_role() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member(
            "a1".to_string(),
            AgentRole::Executor,
            vec!["execute".to_string()],
        )
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(council.members[0].role, AgentRole::Executor);
}

#[test]
fn builder_add_member_custom_role() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member(
            "a1".to_string(),
            AgentRole::Custom("auditor".to_string()),
            vec![],
        )
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(
        council.members[0].role,
        AgentRole::Custom("auditor".to_string())
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilBuilder::add_members (batch)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn builder_add_members_batch() {
    let members = vec![
        (
            "a1".to_string(),
            AgentRole::Prosecutor,
            vec!["r".to_string()],
        ),
        ("a2".to_string(), AgentRole::Judge, vec!["w".to_string()]),
        ("a3".to_string(), AgentRole::Executor, vec!["x".to_string()]),
    ];
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_members(members)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(council.members.len(), 3);
}

#[test]
fn builder_add_members_batch_duplicate_rejected() {
    let members = vec![
        ("a1".to_string(), AgentRole::Prosecutor, vec![]),
        ("a1".to_string(), AgentRole::Judge, vec![]),
    ];
    let result = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_members(members);
    assert!(matches!(result, Err(CouncilError::DuplicateAgent(_))));
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilBuilder::add_rule / add_rules
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn builder_add_rule() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_rule("rule.1".to_string())
        .add_rule("rule.2".to_string())
        .build()
        .unwrap();
    assert_eq!(council.rules.len(), 2);
    assert_eq!(council.rules[0], "rule.1");
    assert_eq!(council.rules[1], "rule.2");
}

#[test]
fn builder_add_rules_batch() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_rules(vec!["r1".to_string(), "r2".to_string(), "r3".to_string()])
        .build()
        .unwrap();
    assert_eq!(council.rules.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilBuilder::with_constitution_validator
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn builder_constitution_validator_rejects() {
    let result = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.999".to_string())
        .with_constitution_validator(|id| id == "cons.1")
        .build();
    assert!(matches!(result, Err(CouncilError::ConstitutionNotFound(_))));
}

#[test]
fn builder_constitution_validator_accepts() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .with_constitution_validator(|id| id == "cons.1")
        .build()
        .unwrap();
    assert_eq!(council.constitution_id, "cons.1");
}

#[test]
fn builder_no_validator_always_accepts() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.999".to_string())
        .build()
        .unwrap();
    assert_eq!(council.constitution_id, "cons.999");
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::new / council
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_new_wraps_council() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    assert_eq!(mgr.council().id, "c1");
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::add_member
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_add_member_success() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.add_member(
        "agent1".to_string(),
        AgentRole::Member,
        vec!["read".to_string()],
    )
    .unwrap();
    assert_eq!(mgr.council().members.len(), 1);
}

#[test]
fn manager_add_member_duplicate_rejected() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("a1".to_string(), AgentRole::Member, vec![])
        .unwrap()
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    let result = mgr.add_member("a1".to_string(), AgentRole::Judge, vec![]);
    assert!(matches!(result, Err(CouncilError::DuplicateAgent(_))));
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::remove_member
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_remove_member_success() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("a1".to_string(), AgentRole::Member, vec![])
        .unwrap()
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.remove_member(&"a1".to_string()).unwrap();
    assert_eq!(mgr.council().members.len(), 0);
}

#[test]
fn manager_remove_member_not_found() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    let result = mgr.remove_member(&"nobody".to_string());
    assert!(matches!(result, Err(CouncilError::AgentNotFound(_))));
}

#[test]
fn manager_remove_preserves_others() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("a1".to_string(), AgentRole::Member, vec![])
        .unwrap()
        .add_member("a2".to_string(), AgentRole::Judge, vec![])
        .unwrap()
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.remove_member(&"a1".to_string()).unwrap();
    assert_eq!(mgr.council().members.len(), 1);
    assert!(mgr.has_member(&"a2".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::has_member / get_member
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_has_member_true() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("a1".to_string(), AgentRole::Member, vec![])
        .unwrap()
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    assert!(mgr.has_member(&"a1".to_string()));
}

#[test]
fn manager_has_member_false() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    assert!(!mgr.has_member(&"a1".to_string()));
}

#[test]
fn manager_get_member_some() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member(
            "a1".to_string(),
            AgentRole::Prosecutor,
            vec!["perm".to_string()],
        )
        .unwrap()
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    let member = mgr.get_member(&"a1".to_string()).unwrap();
    assert_eq!(member.agent_id, "a1");
    assert_eq!(member.role, AgentRole::Prosecutor);
}

#[test]
fn manager_get_member_none() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    assert!(mgr.get_member(&"a1".to_string()).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::update_member_role
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_update_role_success() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("a1".to_string(), AgentRole::Member, vec![])
        .unwrap()
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.update_member_role(&"a1".to_string(), AgentRole::Judge)
        .unwrap();
    assert_eq!(
        mgr.get_member(&"a1".to_string()).unwrap().role,
        AgentRole::Judge
    );
}

#[test]
fn manager_update_role_not_found() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    let result = mgr.update_member_role(&"a1".to_string(), AgentRole::Judge);
    assert!(matches!(result, Err(CouncilError::AgentNotFound(_))));
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::update_member_permissions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_update_permissions_success() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member(
            "a1".to_string(),
            AgentRole::Member,
            vec!["read".to_string()],
        )
        .unwrap()
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.update_member_permissions(
        &"a1".to_string(),
        vec!["read".to_string(), "write".to_string(), "admin".to_string()],
    )
    .unwrap();
    assert_eq!(
        mgr.get_member(&"a1".to_string()).unwrap().permissions.len(),
        3
    );
}

#[test]
fn manager_update_permissions_not_found() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    let result = mgr.update_member_permissions(&"a1".to_string(), vec!["read".to_string()]);
    assert!(matches!(result, Err(CouncilError::AgentNotFound(_))));
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::get_members_by_role
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_get_members_by_role() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("a1".to_string(), AgentRole::Prosecutor, vec![])
        .unwrap()
        .add_member("a2".to_string(), AgentRole::Judge, vec![])
        .unwrap()
        .add_member("a3".to_string(), AgentRole::Prosecutor, vec![])
        .unwrap()
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    assert_eq!(mgr.get_members_by_role(&AgentRole::Prosecutor).len(), 2);
    assert_eq!(mgr.get_members_by_role(&AgentRole::Judge).len(), 1);
    assert_eq!(mgr.get_members_by_role(&AgentRole::Executor).len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::add_rule / remove_rule
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_add_rule() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.add_rule("rule.1".to_string());
    mgr.add_rule("rule.2".to_string());
    assert_eq!(mgr.council().rules.len(), 2);
}

#[test]
fn manager_remove_rule() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_rule("rule.1".to_string())
        .add_rule("rule.2".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.remove_rule(&"rule.1".to_string());
    assert_eq!(mgr.council().rules.len(), 1);
    assert_eq!(mgr.council().rules[0], "rule.2");
}

#[test]
fn manager_remove_rule_nonexistent_is_noop() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_rule("rule.1".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.remove_rule(&"rule.99".to_string());
    assert_eq!(mgr.council().rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::set_active / is_active
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_active_by_default() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    assert!(mgr.is_active());
}

#[test]
fn manager_set_active_false() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.set_active(false);
    assert!(!mgr.is_active());
}

#[test]
fn manager_set_active_toggle() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mut mgr = CouncilManager::new(council);
    mgr.set_active(false);
    assert!(!mgr.is_active());
    mgr.set_active(true);
    assert!(mgr.is_active());
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilManager::validate_unique_agents
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn manager_validate_unique_agents_ok() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .add_member("a1".to_string(), AgentRole::Member, vec![])
        .unwrap()
        .add_member("a2".to_string(), AgentRole::Judge, vec![])
        .unwrap()
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    assert!(mgr.validate_unique_agents().is_ok());
}

#[test]
fn manager_validate_unique_agents_empty_ok() {
    let council = CouncilBuilder::new("c1".to_string(), "T".to_string(), "cons.1".to_string())
        .build()
        .unwrap();
    let mgr = CouncilManager::new(council);
    assert!(mgr.validate_unique_agents().is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// CouncilError Display
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn council_error_constitution_not_found_display() {
    let err = CouncilError::ConstitutionNotFound("cons.99".to_string());
    assert!(err.to_string().contains("Constitution not found: cons.99"));
}

#[test]
fn council_error_duplicate_agent_display() {
    let err = CouncilError::DuplicateAgent("a1".to_string());
    assert!(err
        .to_string()
        .contains("Duplicate agent ID in council: a1"));
}

#[test]
fn council_error_agent_not_found_display() {
    let err = CouncilError::AgentNotFound("a1".to_string());
    assert!(err.to_string().contains("Agent not found in council: a1"));
}

#[test]
fn council_error_invalid_role_display() {
    let err = CouncilError::InvalidRole("bad_role".to_string());
    assert!(err.to_string().contains("Invalid role: bad_role"));
}

#[test]
fn council_error_is_std_error() {
    let err = CouncilError::DuplicateAgent("a1".to_string());
    let _: &dyn std::error::Error = &err;
}

#[test]
fn council_error_eq() {
    assert_eq!(
        CouncilError::DuplicateAgent("a1".to_string()),
        CouncilError::DuplicateAgent("a1".to_string())
    );
    assert_ne!(
        CouncilError::DuplicateAgent("a1".to_string()),
        CouncilError::AgentNotFound("a1".to_string())
    );
}
