//! Property-based tests for governance system
//! Migrated from hudhudscript-governance/src/property_tests/

use chrono::Utc;
use hudhudscript_governance::*;
use proptest::prelude::*;
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// STRATEGIES (inlined from property_tests/strategies.rs)
// ============================================================================

fn constitution_id_strategy() -> impl Strategy<Value = String> {
    (1u32..1000).prop_map(|n| format!("cons.{}", n))
}

fn law_id_strategy() -> impl Strategy<Value = String> {
    (1u32..100, 1u32..100)
        .prop_map(|(cons_num, law_num)| format!("cons{}.law{}", cons_num, law_num))
}

fn rule_id_strategy() -> impl Strategy<Value = String> {
    (1u32..1000).prop_map(|n| format!("rule.{}", n))
}

fn agent_id_strategy() -> impl Strategy<Value = String> {
    "[a-z]{3,10}[0-9]{1,3}".prop_map(|s| s)
}

fn swarm_id_strategy() -> impl Strategy<Value = String> {
    "[a-z]{5,10}".prop_map(|s| format!("swarm_{}", s))
}

fn constitution_strategy() -> impl Strategy<Value = Constitution> {
    (
        constitution_id_strategy(),
        "[a-zA-Z ]{5,20}",
        prop::option::of("[a-zA-Z ]{10,50}"),
    )
        .prop_map(|(id, name, description)| Constitution {
            id,
            name,
            description,
            laws: HashMap::new(),
            created_at: Utc::now(),
            version: 1,
        })
}

fn law_strategy() -> impl Strategy<Value = Law> {
    (
        (1u32..100, 1u32..100),
        "[a-zA-Z ]{5,20}",
        "[a-zA-Z ]{10,50}",
        prop::sample::select(vec![
            EnforcementLevel::Mandatory,
            EnforcementLevel::Advisory,
            EnforcementLevel::Optional,
        ]),
    )
        .prop_map(
            |((cons_num, law_num), name, description, enforcement_level)| {
                let id = format!("cons{}.law{}", cons_num, law_num);
                let constitution_id = format!("cons.{}", cons_num);
                Law {
                    id,
                    constitution_id,
                    name,
                    description,
                    enforcement_level,
                    conditions: vec![],
                }
            },
        )
}

fn constitution_with_laws_strategy() -> impl Strategy<Value = Constitution> {
    (1u32..100).prop_map(|cons_num| {
        let mut laws = HashMap::new();

        laws.insert(
            format!("cons{}.law1", cons_num),
            Law {
                id: format!("cons{}.law1", cons_num),
                constitution_id: format!("cons.{}", cons_num),
                name: "Data Size Limit".to_string(),
                description: "Data must be under 5000 bytes".to_string(),
                enforcement_level: EnforcementLevel::Mandatory,
                conditions: vec![Condition::LessThan {
                    field: "data_size".to_string(),
                    value: 5000.0,
                }],
            },
        );

        laws.insert(
            format!("cons{}.law2", cons_num),
            Law {
                id: format!("cons{}.law2", cons_num),
                constitution_id: format!("cons.{}", cons_num),
                name: "High Priority Recommended".to_string(),
                description: "Actions should have high priority".to_string(),
                enforcement_level: EnforcementLevel::Advisory,
                conditions: vec![Condition::Equals {
                    field: "priority".to_string(),
                    value: json!("high"),
                }],
            },
        );

        laws.insert(
            format!("cons{}.law3", cons_num),
            Law {
                id: format!("cons{}.law3", cons_num),
                constitution_id: format!("cons.{}", cons_num),
                name: "Metadata Optional".to_string(),
                description: "Actions may include metadata".to_string(),
                enforcement_level: EnforcementLevel::Optional,
                conditions: vec![Condition::Equals {
                    field: "has_metadata".to_string(),
                    value: json!(true),
                }],
            },
        );

        Constitution {
            id: format!("cons.{}", cons_num),
            name: format!("Test Constitution {}", cons_num),
            description: Some(format!("A test constitution {}", cons_num)),
            laws,
            created_at: Utc::now(),
            version: 1,
        }
    })
}

#[allow(dead_code)]
fn rule_strategy() -> impl Strategy<Value = Rule> {
    (rule_id_strategy(), "[a-zA-Z ]{5,20}", 0u32..100).prop_map(|(id, name, priority)| Rule {
        id,
        name,
        conditions: vec![],
        actions: vec![Action::Allow],
        priority,
    })
}

fn evaluation_context_strategy(
) -> impl Strategy<Value = hudhudscript_governance::enforcement::EvaluationContext> {
    (
        0u32..10000,
        prop::sample::select(vec!["low", "medium", "high", "critical"]),
        prop::bool::ANY,
        prop::sample::select(vec!["active", "inactive", "pending"]),
    )
        .prop_map(|(data_size, priority, has_metadata, status)| {
            let mut context = hudhudscript_governance::enforcement::EvaluationContext::new();
            context.insert("data_size".to_string(), json!(data_size));
            context.insert("priority".to_string(), json!(priority));
            context.insert("has_metadata".to_string(), json!(has_metadata));
            context.insert("status".to_string(), json!(status));
            context
        })
}

fn agent_role_strategy() -> impl Strategy<Value = AgentRole> {
    prop::sample::select(vec![
        AgentRole::Prosecutor,
        AgentRole::Judge,
        AgentRole::Executor,
        AgentRole::Member,
        AgentRole::Custom("DataValidator".to_string()),
        AgentRole::Custom("ComplianceChecker".to_string()),
    ])
}

type PermissionStr = String;

fn permissions_strategy() -> impl Strategy<Value = Vec<PermissionStr>> {
    prop::collection::vec(
        prop::sample::select(vec![
            "read".to_string(),
            "write".to_string(),
            "execute".to_string(),
            "propose_action".to_string(),
            "evaluate_compliance".to_string(),
            "make_decision".to_string(),
        ]),
        0..5,
    )
}

fn council_with_members_strategy() -> impl Strategy<Value = Council> {
    (
        "[a-z]{5,10}".prop_map(|s| format!("council_{}", s)),
        "[A-Z][a-z ]{5,20}",
        constitution_id_strategy(),
        prop::collection::vec(
            (
                agent_id_strategy(),
                agent_role_strategy(),
                permissions_strategy(),
            ),
            1..10,
        ),
    )
        .prop_map(|(id, name, constitution_id, member_data)| {
            use hudhudscript_governance::council::CouncilBuilder;
            use std::collections::HashSet;

            let mut unique_members = Vec::new();
            let mut seen_ids = HashSet::new();

            for (agent_id, role, permissions) in member_data {
                if seen_ids.insert(agent_id.clone()) {
                    unique_members.push((agent_id, role, permissions));
                }
            }

            let mut builder = CouncilBuilder::new(id, name, constitution_id);

            for (agent_id, role, permissions) in unique_members {
                builder = builder.add_member(agent_id, role, permissions).unwrap();
            }

            builder.build().unwrap()
        })
}

fn coordination_strategy_strategy() -> impl Strategy<Value = CoordinationStrategy> {
    prop::sample::select(vec![
        CoordinationStrategy::Parallel,
        CoordinationStrategy::Sequential,
        CoordinationStrategy::Competitive,
        CoordinationStrategy::Collaborative,
    ])
}

fn swarm_strategy() -> impl Strategy<Value = Swarm> {
    (
        swarm_id_strategy(),
        "[A-Z][a-z ]{5,20}",
        prop::collection::vec(agent_id_strategy(), 1..10),
        coordination_strategy_strategy(),
    )
        .prop_map(|(id, name, agents, strategy)| {
            use std::collections::HashSet;

            let unique_agents: Vec<String> = agents
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            Swarm::new(id, name, unique_agents, strategy)
        })
}

// ============================================================================
// CONSTITUTION TESTS
// ============================================================================

proptest! {
    #[test]
    fn property_constitution_integrity(constitution in constitution_strategy()) {
        let id_regex = regex::Regex::new(r"^cons\.\d+$").unwrap();
        prop_assert!(id_regex.is_match(&constitution.id), "Constitution ID must match format cons.\\d+");

        for (law_id, law) in &constitution.laws {
            let cons_num = constitution.id.strip_prefix("cons.").unwrap();
            let expected_prefix = format!("cons{}.law", cons_num);
            prop_assert!(
                law_id.starts_with(&expected_prefix),
                "Law ID {} must start with {}",
                law_id,
                expected_prefix
            );
            prop_assert_eq!(&law.constitution_id, &constitution.id, "Law must reference parent constitution");
        }
    }

    #[test]
    fn property_constitution_id_format(id in constitution_id_strategy()) {
        let regex = regex::Regex::new(r"^cons\.\d+$").unwrap();
        prop_assert!(regex.is_match(&id), "Constitution ID {} must match format ^cons\\.\\d+$", id);
    }

    #[test]
    fn property_constitution_metadata(constitution in constitution_strategy()) {
        prop_assert_eq!(constitution.version, 1, "New constitution must have version 1");
        prop_assert!(constitution.created_at <= Utc::now(), "Constitution timestamp must be in the past or present");
        prop_assert!(!constitution.id.is_empty(), "Constitution ID must not be empty");
        let regex = regex::Regex::new(r"^cons\.\d+$").unwrap();
        prop_assert!(regex.is_match(&constitution.id), "Constitution ID must match format");
    }

    #[test]
    fn property_constitution_serialization_roundtrip(constitution in constitution_strategy()) {
        let json = serde_json::to_string(&constitution).unwrap();
        let deserialized: Constitution = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(constitution.id, deserialized.id);
        prop_assert_eq!(constitution.name, deserialized.name);
        prop_assert_eq!(constitution.description, deserialized.description);
        prop_assert_eq!(constitution.version, deserialized.version);
    }
}

// ============================================================================
// LAW TESTS
// ============================================================================

proptest! {
    #[test]
    fn property_law_id_format(id in law_id_strategy()) {
        let regex = regex::Regex::new(r"^cons\d+\.law\d+$").unwrap();
        prop_assert!(regex.is_match(&id), "Law ID {} must match format ^cons\\d+\\.law\\d+$", id);
    }

    #[test]
    fn property_law_required_fields(law in law_strategy()) {
        prop_assert!(!law.name.is_empty(), "Law name must not be empty");
        prop_assert!(!law.description.is_empty(), "Law description must not be empty");
        prop_assert!(
            matches!(
                law.enforcement_level,
                EnforcementLevel::Mandatory | EnforcementLevel::Advisory | EnforcementLevel::Optional
            ),
            "Law enforcement level must be Mandatory, Advisory, or Optional"
        );
    }

    #[test]
    fn property_law_hierarchy_consistency(law in law_strategy()) {
        let law_id_parts: Vec<&str> = law.id.split('.').collect();
        if law_id_parts.len() == 2 {
            let cons_part = law_id_parts[0];
            let cons_num = cons_part.strip_prefix("cons").unwrap();
            let expected_constitution_id = format!("cons.{}", cons_num);
            prop_assert_eq!(
                law.constitution_id,
                expected_constitution_id,
                "Law constitution_id must match the number in its ID"
            );
        }
    }

    #[test]
    fn property_law_serialization_roundtrip(law in law_strategy()) {
        let json = serde_json::to_string(&law).unwrap();
        let deserialized: Law = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(law.id, deserialized.id);
        prop_assert_eq!(law.constitution_id, deserialized.constitution_id);
        prop_assert_eq!(law.name, deserialized.name);
        prop_assert_eq!(law.description, deserialized.description);
        prop_assert_eq!(law.enforcement_level, deserialized.enforcement_level);
    }
}

// ============================================================================
// ENFORCEMENT TESTS
// ============================================================================

proptest! {
    #[test]
    fn property_enforcement_determinism(
        constitution in constitution_with_laws_strategy(),
        context in evaluation_context_strategy()
    ) {
        use hudhudscript_governance::enforcement::enforce_constitution;

        let result1 = enforce_constitution(&constitution, &context, None);
        let result2 = enforce_constitution(&constitution, &context, None);
        let result3 = enforce_constitution(&constitution, &context, None);

        prop_assert_eq!(result1.allowed, result2.allowed, "Enforcement must be deterministic");
        prop_assert_eq!(result2.allowed, result3.allowed, "Enforcement must be deterministic");
        prop_assert_eq!(&result1.violations, &result2.violations, "Violations must be deterministic");
        prop_assert_eq!(&result2.violations, &result3.violations, "Violations must be deterministic");
        prop_assert_eq!(&result1.advisory_violations, &result2.advisory_violations, "Advisory violations must be deterministic");
        prop_assert_eq!(&result2.advisory_violations, &result3.advisory_violations, "Advisory violations must be deterministic");
    }

    #[test]
    fn property_mandatory_law_enforcement(
        constitution in constitution_with_laws_strategy(),
        data_size in 0u32..10000
    ) {
        use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};

        let mut context = EvaluationContext::new();
        context.insert("data_size".to_string(), json!(data_size));
        context.insert("priority".to_string(), json!("high"));
        context.insert("has_metadata".to_string(), json!(true));

        let result = enforce_constitution(&constitution, &context, None);

        if data_size < 5000 {
            prop_assert!(result.allowed, "Action should be allowed when mandatory laws are satisfied");
            prop_assert_eq!(result.violations.len(), 0, "No violations when mandatory laws are satisfied");
        } else {
            prop_assert!(!result.allowed, "Action should be denied when mandatory laws are violated");
            prop_assert!(!result.violations.is_empty(), "Violations list should not be empty");

            let mandatory_law_id = format!("cons{}.law1",
                constitution.id.strip_prefix("cons.").unwrap());
            prop_assert!(
                result.violations.contains(&mandatory_law_id),
                "Violations should include the mandatory law ID"
            );
        }
    }

    #[test]
    fn property_advisory_law_non_blocking(
        constitution in constitution_with_laws_strategy(),
        priority in prop::sample::select(vec!["low", "medium", "high", "critical"])
    ) {
        use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};

        let mut context = EvaluationContext::new();
        context.insert("data_size".to_string(), json!(1000));
        context.insert("priority".to_string(), json!(priority));
        context.insert("has_metadata".to_string(), json!(true));

        let result = enforce_constitution(&constitution, &context, None);

        prop_assert!(result.allowed, "Action should be allowed even if advisory laws are violated");
        prop_assert_eq!(result.violations.len(), 0, "No mandatory violations");

        if priority != "high" {
            let advisory_law_id = format!("cons{}.law2",
                constitution.id.strip_prefix("cons.").unwrap());
            prop_assert!(
                result.advisory_violations.contains(&advisory_law_id),
                "Advisory violations should be recorded"
            );
        } else {
            prop_assert_eq!(
                result.advisory_violations.len(), 0,
                "No advisory violations when advisory laws are satisfied"
            );
        }
    }

    #[test]
    fn property_enforcement_audit_logging(
        constitution in constitution_with_laws_strategy(),
        context in evaluation_context_strategy()
    ) {
        use hudhudscript_governance::enforcement::enforce_constitution;
        use hudhudscript_governance::audit::{AuditLogger, AuditEventType};

        let audit_logger = AuditLogger::new();

        let result = enforce_constitution(&constitution, &context, Some(&audit_logger));

        prop_assert_eq!(audit_logger.count(), 1, "Enforcement decision should be logged");

        let events = audit_logger.get_events();
        prop_assert_eq!(events.len(), 1, "Should have exactly one audit event");
        prop_assert_eq!(&events[0].event_type, &AuditEventType::EnforcementDecision, "Event type should be EnforcementDecision");

        match &events[0].data {
            hudhudscript_governance::audit::AuditEventData::Enforcement {
                constitution_id,
                allowed,
                violations,
                advisory_violations,
                ..
            } => {
                prop_assert_eq!(constitution_id, &constitution.id, "Constitution ID should match");
                prop_assert_eq!(allowed, &result.allowed, "Allowed status should match");
                prop_assert_eq!(violations, &result.violations, "Violations should match");
                prop_assert_eq!(advisory_violations, &result.advisory_violations, "Advisory violations should match");
            }
            _ => prop_assert!(false, "Expected Enforcement audit data"),
        }
    }

    #[test]
    fn property_optional_law_violations_ignored(
        constitution in constitution_with_laws_strategy(),
        has_metadata in prop::bool::ANY
    ) {
        use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};

        let mut context = EvaluationContext::new();
        context.insert("data_size".to_string(), json!(1000));
        context.insert("priority".to_string(), json!("high"));
        context.insert("has_metadata".to_string(), json!(has_metadata));

        let result = enforce_constitution(&constitution, &context, None);

        prop_assert!(result.allowed, "Action should be allowed regardless of optional law violations");
        prop_assert_eq!(result.violations.len(), 0, "No mandatory violations");
        prop_assert_eq!(result.advisory_violations.len(), 0, "No advisory violations");

        let optional_law_id = format!("cons{}.law3",
            constitution.id.strip_prefix("cons.").unwrap());
        prop_assert!(
            !result.violations.contains(&optional_law_id),
            "Optional law should not appear in violations"
        );
        prop_assert!(
            !result.advisory_violations.contains(&optional_law_id),
            "Optional law should not appear in advisory violations"
        );
    }

    #[test]
    fn property_multiple_mandatory_violations_reported(cons_num in 1u32..100) {
        use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};

        let mut laws = HashMap::new();

        laws.insert(
            format!("cons{}.law1", cons_num),
            Law {
                id: format!("cons{}.law1", cons_num),
                constitution_id: format!("cons.{}", cons_num),
                name: "Law 1".to_string(),
                description: "First mandatory law".to_string(),
                enforcement_level: EnforcementLevel::Mandatory,
                conditions: vec![Condition::Equals {
                    field: "field1".to_string(),
                    value: json!("value1"),
                }],
            },
        );

        laws.insert(
            format!("cons{}.law2", cons_num),
            Law {
                id: format!("cons{}.law2", cons_num),
                constitution_id: format!("cons.{}", cons_num),
                name: "Law 2".to_string(),
                description: "Second mandatory law".to_string(),
                enforcement_level: EnforcementLevel::Mandatory,
                conditions: vec![Condition::Equals {
                    field: "field2".to_string(),
                    value: json!("value2"),
                }],
            },
        );

        let constitution = Constitution {
            id: format!("cons.{}", cons_num),
            name: "Multi-Law Constitution".to_string(),
            description: None,
            laws,
            created_at: Utc::now(),
            version: 1,
        };

        let mut context = EvaluationContext::new();
        context.insert("field1".to_string(), json!("wrong1"));
        context.insert("field2".to_string(), json!("wrong2"));

        let result = enforce_constitution(&constitution, &context, None);

        prop_assert!(!result.allowed, "Action should be denied when multiple mandatory laws are violated");
        prop_assert_eq!(result.violations.len(), 2, "All mandatory violations should be reported");
        prop_assert!(
            result.violations.contains(&format!("cons{}.law1", cons_num)),
            "First violation should be reported"
        );
        prop_assert!(
            result.violations.contains(&format!("cons{}.law2", cons_num)),
            "Second violation should be reported"
        );
    }
}

// ============================================================================
// COUNCIL TESTS
// ============================================================================

proptest! {
    #[test]
    fn property_council_member_uniqueness(council in council_with_members_strategy()) {
        use std::collections::HashSet;

        let agent_ids: Vec<&String> = council.members.iter().map(|m| &m.agent_id).collect();
        let unique_ids: HashSet<&String> = agent_ids.iter().copied().collect();

        prop_assert_eq!(
            agent_ids.len(),
            unique_ids.len(),
            "Council must have unique agent IDs (no duplicates)"
        );

        let mut seen = HashSet::new();
        for member in &council.members {
            prop_assert!(
                seen.insert(&member.agent_id),
                "Duplicate agent ID found: {}",
                member.agent_id
            );
        }
    }

    #[test]
    fn property_council_builder_rejects_duplicates(
        id in "[a-z]{5,10}".prop_map(|s| format!("council_{}", s)),
        name in "[A-Z][a-z ]{5,20}",
        constitution_id in constitution_id_strategy(),
        agent_id in agent_id_strategy(),
        role1 in agent_role_strategy(),
        role2 in agent_role_strategy(),
        perms1 in permissions_strategy(),
        perms2 in permissions_strategy(),
    ) {
        use hudhudscript_governance::council::{CouncilBuilder, CouncilError};

        let result = CouncilBuilder::new(id, name, constitution_id)
            .add_member(agent_id.clone(), role1, perms1)
            .and_then(|builder| builder.add_member(agent_id.clone(), role2, perms2));

        prop_assert!(
            matches!(result, Err(CouncilError::DuplicateAgent(_))),
            "Adding duplicate agent should fail"
        );
    }

    #[test]
    fn property_agent_role_assignment(council in council_with_members_strategy()) {
        for member in &council.members {
            let is_valid_role = matches!(
                member.role,
                AgentRole::Prosecutor
                    | AgentRole::Judge
                    | AgentRole::Executor
                    | AgentRole::Member
                    | AgentRole::Custom(_)
            );

            prop_assert!(
                is_valid_role,
                "Agent {} must have a valid role",
                member.agent_id
            );
        }
    }

    #[test]
    fn property_all_predefined_roles_supported(
        id in "[a-z]{5,10}".prop_map(|s| format!("council_{}", s)),
        name in "[A-Z][a-z ]{5,20}",
        constitution_id in constitution_id_strategy(),
    ) {
        use hudhudscript_governance::council::CouncilBuilder;

        let council = CouncilBuilder::new(id, name, constitution_id)
            .add_member("agent1".to_string(), AgentRole::Prosecutor, vec![])
            .unwrap()
            .add_member("agent2".to_string(), AgentRole::Judge, vec![])
            .unwrap()
            .add_member("agent3".to_string(), AgentRole::Executor, vec![])
            .unwrap()
            .add_member("agent4".to_string(), AgentRole::Member, vec![])
            .unwrap()
            .build()
            .unwrap();

        prop_assert_eq!(council.members.len(), 4, "All predefined roles should be assignable");
        prop_assert_eq!(&council.members[0].role, &AgentRole::Prosecutor);
        prop_assert_eq!(&council.members[1].role, &AgentRole::Judge);
        prop_assert_eq!(&council.members[2].role, &AgentRole::Executor);
        prop_assert_eq!(&council.members[3].role, &AgentRole::Member);
    }

    #[test]
    fn property_custom_roles_supported(
        id in "[a-z]{5,10}".prop_map(|s| format!("council_{}", s)),
        name in "[A-Z][a-z ]{5,20}",
        constitution_id in constitution_id_strategy(),
        custom_role_name in "[A-Z][a-z]{5,15}",
    ) {
        use hudhudscript_governance::council::CouncilBuilder;

        let custom_role = AgentRole::Custom(custom_role_name.clone());

        let council = CouncilBuilder::new(id, name, constitution_id)
            .add_member("agent1".to_string(), custom_role.clone(), vec![])
            .unwrap()
            .build()
            .unwrap();

        prop_assert_eq!(council.members.len(), 1);
        prop_assert_eq!(&council.members[0].role, &custom_role);
    }

    #[test]
    fn property_role_permission_association(council in council_with_members_strategy()) {
        use hudhudscript_governance::role::RoleManager;

        let role_manager = RoleManager::new();

        for member in &council.members {
            let default_perms = role_manager.get_default_permissions(&member.role);

            if role_manager.is_predefined_role(&member.role) {
                prop_assert!(
                    !default_perms.is_empty(),
                    "Predefined role {:?} must have default permissions",
                    member.role
                );
            }

            let _ = &member.permissions;
        }
    }

    #[test]
    fn property_predefined_roles_have_permissions(
        id in "[a-z]{5,10}".prop_map(|s| format!("council_{}", s)),
        name in "[A-Z][a-z ]{5,20}",
        constitution_id in constitution_id_strategy(),
    ) {
        use hudhudscript_governance::council::CouncilBuilder;
        use hudhudscript_governance::role::RoleManager;

        let role_manager = RoleManager::new();

        let council = CouncilBuilder::new(id, name, constitution_id)
            .add_member(
                "agent1".to_string(),
                AgentRole::Prosecutor,
                role_manager.get_default_permissions(&AgentRole::Prosecutor),
            )
            .unwrap()
            .add_member(
                "agent2".to_string(),
                AgentRole::Judge,
                role_manager.get_default_permissions(&AgentRole::Judge),
            )
            .unwrap()
            .add_member(
                "agent3".to_string(),
                AgentRole::Executor,
                role_manager.get_default_permissions(&AgentRole::Executor),
            )
            .unwrap()
            .add_member(
                "agent4".to_string(),
                AgentRole::Member,
                role_manager.get_default_permissions(&AgentRole::Member),
            )
            .unwrap()
            .build()
            .unwrap();

        for member in &council.members {
            prop_assert!(
                !member.permissions.is_empty(),
                "Member with predefined role {:?} should have permissions",
                member.role
            );
        }

        let prosecutor = &council.members[0];
        prop_assert!(
            prosecutor.permissions.contains(&"propose_action".to_string()),
            "Prosecutor should have propose_action permission"
        );

        let judge = &council.members[1];
        prop_assert!(
            judge.permissions.contains(&"evaluate_compliance".to_string()),
            "Judge should have evaluate_compliance permission"
        );

        let executor = &council.members[2];
        prop_assert!(
            executor.permissions.contains(&"execute_decision".to_string()),
            "Executor should have execute_decision permission"
        );

        let member = &council.members[3];
        prop_assert!(
            member.permissions.contains(&"read_constitution".to_string()),
            "Member should have read_constitution permission"
        );
    }

    #[test]
    fn property_custom_roles_can_have_permissions(
        id in "[a-z]{5,10}".prop_map(|s| format!("council_{}", s)),
        name in "[A-Z][a-z ]{5,20}",
        constitution_id in constitution_id_strategy(),
        custom_role_name in "[A-Z][a-z]{5,15}",
        permissions in permissions_strategy(),
    ) {
        use hudhudscript_governance::council::CouncilBuilder;

        let custom_role = AgentRole::Custom(custom_role_name);

        let council = CouncilBuilder::new(id, name, constitution_id)
            .add_member("agent1".to_string(), custom_role.clone(), permissions.clone())
            .unwrap()
            .build()
            .unwrap();

        prop_assert_eq!(council.members.len(), 1);
        prop_assert_eq!(&council.members[0].permissions, &permissions);
    }

    #[test]
    fn property_council_state_consistency(council in council_with_members_strategy()) {
        use hudhudscript_governance::council::CouncilManager;

        let manager = CouncilManager::new(council.clone());

        prop_assert!(manager.is_active(), "Council should be active by default");
        prop_assert!(
            manager.validate_unique_agents().is_ok(),
            "Council should have unique agents"
        );

        for member in &council.members {
            prop_assert!(
                manager.has_member(&member.agent_id),
                "Member {} should be findable",
                member.agent_id
            );

            let found_member = manager.get_member(&member.agent_id);
            prop_assert!(found_member.is_some(), "Member should be retrievable");
            prop_assert_eq!(
                &found_member.unwrap().agent_id,
                &member.agent_id,
                "Retrieved member should match"
            );
        }
    }
}

// ============================================================================
// SWARM TESTS
// ============================================================================

proptest! {
    #[test]
    fn property_swarm_coordination_strategy_support(
        id in swarm_id_strategy(),
        name in "[A-Z][a-z ]{5,20}",
        agents in prop::collection::vec(agent_id_strategy(), 1..5),
    ) {
        use std::collections::HashSet;
        let unique_agents: Vec<String> = agents.into_iter().collect::<HashSet<_>>().into_iter().collect();

        let strategies = vec![
            CoordinationStrategy::Parallel,
            CoordinationStrategy::Sequential,
            CoordinationStrategy::Competitive,
            CoordinationStrategy::Collaborative,
        ];

        for strategy in strategies {
            let swarm = Swarm::new(
                id.clone(),
                name.clone(),
                unique_agents.clone(),
                strategy,
            );

            prop_assert_eq!(
                swarm.get_strategy(),
                strategy,
                "Swarm should support {:?} coordination strategy",
                strategy
            );
        }
    }

    #[test]
    fn property_swarm_strategy_can_be_changed(swarm in swarm_strategy()) {
        let mut swarm = swarm;
        let original_strategy = swarm.get_strategy();

        let new_strategy = match original_strategy {
            CoordinationStrategy::Parallel => CoordinationStrategy::Sequential,
            CoordinationStrategy::Sequential => CoordinationStrategy::Competitive,
            CoordinationStrategy::Competitive => CoordinationStrategy::Collaborative,
            CoordinationStrategy::Collaborative => CoordinationStrategy::Parallel,
        };

        swarm.set_strategy(new_strategy);
        prop_assert_eq!(swarm.get_strategy(), new_strategy, "Strategy should be updated");
    }

    #[test]
    fn property_swarm_shared_state_accessibility(
        swarm in swarm_strategy(),
        key in "[a-z]{3,10}",
        value in prop::num::i32::ANY,
    ) {
        let mut swarm = swarm;

        swarm.set_shared_state(key.clone(), json!(value));

        let retrieved = swarm.get_shared_state(&key);
        prop_assert!(retrieved.is_some(), "Shared state should be accessible");
        prop_assert_eq!(retrieved.unwrap(), &json!(value), "Shared state value should match");
    }

    #[test]
    fn property_swarm_shared_state_updates_visible(
        swarm in swarm_strategy(),
        key in "[a-z]{3,10}",
        value1 in prop::num::i32::ANY,
        value2 in prop::num::i32::ANY,
    ) {
        let mut swarm = swarm;

        swarm.set_shared_state(key.clone(), json!(value1));
        prop_assert_eq!(swarm.get_shared_state(&key), Some(&json!(value1)));

        swarm.set_shared_state(key.clone(), json!(value2));

        prop_assert_eq!(
            swarm.get_shared_state(&key),
            Some(&json!(value2)),
            "Updated shared state should be visible"
        );
    }

    #[test]
    fn property_swarm_shared_state_initialization(
        id in swarm_id_strategy(),
        name in "[A-Z][a-z ]{5,20}",
        agents in prop::collection::vec(agent_id_strategy(), 1..5),
        strategy in coordination_strategy_strategy(),
    ) {
        use std::collections::HashSet;
        let unique_agents: Vec<String> = agents.into_iter().collect::<HashSet<_>>().into_iter().collect();

        let swarm = Swarm::new(id, name, unique_agents, strategy);

        prop_assert!(
            swarm.shared_state.is_empty(),
            "Newly created swarm should have empty shared state"
        );
        prop_assert_eq!(
            swarm.shared_state.len(),
            0,
            "Shared state should have zero entries"
        );
    }

    #[test]
    fn property_shared_state_visibility(
        swarm in swarm_strategy(),
        updates in prop::collection::vec(
            ("[a-z]{3,10}", prop::num::i32::ANY),
            1..10
        ),
    ) {
        let mut swarm = swarm;

        for (key, value) in &updates {
            swarm.set_shared_state(key.clone(), json!(value));
        }

        for (key, value) in &updates {
            let retrieved = swarm.get_shared_state(key);
            prop_assert!(
                retrieved.is_some(),
                "Update to key '{}' should be visible",
                key
            );
            prop_assert_eq!(
                retrieved.unwrap(),
                &json!(value),
                "Value for key '{}' should match",
                key
            );
        }
    }

    #[test]
    fn property_atomic_shared_state_operations(
        swarm in swarm_strategy(),
        key in "[a-z]{3,10}",
        initial_value in 0i32..100,
        increment in 1i32..10,
    ) {
        let mut swarm = swarm;

        swarm.set_shared_state(key.clone(), json!(initial_value));

        let current = swarm.get_shared_state(&key).unwrap().as_i64().unwrap() as i32;
        let new_value = current + increment;
        swarm.set_shared_state(key.clone(), json!(new_value));

        let final_value = swarm.get_shared_state(&key).unwrap().as_i64().unwrap() as i32;
        prop_assert_eq!(
            final_value,
            initial_value + increment,
            "Read-modify-write should be atomic"
        );
    }

    #[test]
    fn property_swarm_agent_management(
        swarm in swarm_strategy(),
        new_agent in agent_id_strategy(),
    ) {
        let mut swarm = swarm;
        let initial_count = swarm.agent_count();

        if !swarm.has_agent(&new_agent) {
            let result = swarm.add_agent(new_agent.clone());
            prop_assert!(result.is_ok(), "Adding new agent should succeed");
            prop_assert_eq!(
                swarm.agent_count(),
                initial_count + 1,
                "Agent count should increase"
            );
            prop_assert!(swarm.has_agent(&new_agent), "New agent should be in swarm");
        }
    }

    #[test]
    fn property_swarm_agent_uniqueness(swarm in swarm_strategy()) {
        use std::collections::HashSet;
        let agent_set: HashSet<&String> = swarm.agents.iter().collect();
        prop_assert_eq!(
            agent_set.len(),
            swarm.agents.len(),
            "All agents in swarm should be unique"
        );
    }

    #[test]
    fn property_swarm_duplicate_agent_rejected(swarm in swarm_strategy()) {
        let mut swarm = swarm;

        if swarm.agents.is_empty() {
            return Ok(());
        }

        let existing_agent = swarm.agents[0].clone();
        let result = swarm.add_agent(existing_agent);

        prop_assert!(result.is_err(), "Adding duplicate agent should fail");
    }

    #[test]
    fn property_swarm_state_persistence(
        swarm in swarm_strategy(),
        state_data in prop::collection::vec(
            ("[a-z]{3,10}", prop::num::i32::ANY),
            1..10
        ),
    ) {
        let mut swarm = swarm;

        for (key, value) in &state_data {
            swarm.set_shared_state(key.clone(), json!(value));
        }

        let serialized = serde_json::to_string(&swarm).unwrap();
        let deserialized: Swarm = serde_json::from_str(&serialized).unwrap();

        for (key, value) in &state_data {
            let retrieved = deserialized.get_shared_state(key);
            prop_assert!(
                retrieved.is_some(),
                "State key '{}' should persist",
                key
            );
            prop_assert_eq!(
                retrieved.unwrap(),
                &json!(value),
                "State value for '{}' should persist",
                key
            );
        }
    }
}

// ============================================================================
// RESOURCE TESTS
// ============================================================================

proptest! {
    #[test]
    fn property_community_resource_sharing(
        resource_ids in prop::collection::vec("[a-z]{3,10}", 1..10),
    ) {
        use hudhudscript_governance::resources::{AllocationPolicy, Resource, ResourceManager};

        let mut manager = ResourceManager::new();

        for (i, id) in resource_ids.iter().enumerate() {
            let resource = Resource::new(
                id.clone(),
                format!("Resource {}", i),
                None,
                100,
                AllocationPolicy::FairShare,
            );
            manager.register_resource(resource).unwrap();
        }

        let result = manager.validate_resources(&resource_ids);
        prop_assert!(result.is_ok(), "All registered resources should validate");
    }

    #[test]
    fn property_resource_access_control(
        resource_id in "[a-z]{3,10}",
        agent_ids in prop::collection::vec("[a-z]{3,10}", 2..5),
    ) {
        use hudhudscript_governance::resources::{AllocationPolicy, Resource, ResourceManager};

        let mut manager = ResourceManager::new();
        let resource = Resource::new(
            resource_id.clone(),
            "Test Resource".to_string(),
            None,
            100,
            AllocationPolicy::FairShare,
        );
        manager.register_resource(resource).unwrap();

        manager.grant_access(&resource_id, agent_ids[0].clone()).unwrap();

        prop_assert!(
            manager.has_access(&resource_id, &agent_ids[0]),
            "Agent with granted access should have access"
        );

        for agent_id in &agent_ids[1..] {
            prop_assert!(
                !manager.has_access(&resource_id, agent_id),
                "Agent without granted access should not have access"
            );
        }
    }

    #[test]
    fn property_resource_usage_tracking(
        resource_id in "[a-z]{3,10}",
        agent_id in "[a-z]{3,10}",
        amounts in prop::collection::vec(1u32..20, 1..5),
    ) {
        use hudhudscript_governance::resources::{AllocationPolicy, Resource, ResourceManager};

        let mut manager = ResourceManager::new();
        let total_capacity: u32 = amounts.iter().sum::<u32>() + 100;
        let resource = Resource::new(
            resource_id.clone(),
            "Test Resource".to_string(),
            None,
            total_capacity,
            AllocationPolicy::FairShare,
        );
        manager.register_resource(resource).unwrap();
        manager.grant_access(&resource_id, agent_id.clone()).unwrap();

        for amount in &amounts {
            manager.allocate_resource(&resource_id, &agent_id, *amount).unwrap();
        }

        let usage = manager.get_agent_usage(&agent_id);
        prop_assert_eq!(
            usage.len(),
            amounts.len(),
            "Should track all allocations"
        );

        let total_usage = manager.get_total_usage(&resource_id, &agent_id);
        let expected_total: u32 = amounts.iter().sum();
        prop_assert_eq!(
            total_usage,
            expected_total,
            "Total usage should match sum of allocations"
        );
    }

    #[test]
    fn property_community_culture_definition(
        values in prop::collection::vec("[a-z]{3,15}", 1..5),
        norms in prop::collection::vec("[a-z]{3,15}", 1..5),
        style in prop::sample::select(vec![
            CommunicationStyle::Formal,
            CommunicationStyle::Informal,
            CommunicationStyle::Technical,
            CommunicationStyle::Collaborative,
        ]),
    ) {
        let culture = CommunityCulture::new(values.clone(), norms.clone(), style);

        prop_assert_eq!(culture.values, values, "Values should match");
        prop_assert_eq!(culture.norms, norms, "Norms should match");
        prop_assert_eq!(culture.communication_style, style, "Communication style should match");
    }
}

// ============================================================================
// AGENT INTEGRATION TESTS
// ============================================================================

proptest! {
    #[test]
    fn property_agent_reference_validation(
        agent_ids in prop::collection::vec("[a-z]{3,10}", 2..5),
    ) {
        use hudhudscript_governance::agent_integration::{AgentMembershipTracker, AgentRegistry};

        let mut registry = AgentRegistry::new();

        for agent_id in &agent_ids {
            registry.register_agent(agent_id.clone()).unwrap();
        }

        let result = registry.validate_agents(&agent_ids);
        prop_assert!(result.is_ok(), "All registered agents should validate");

        let mut invalid_ids = agent_ids.clone();
        invalid_ids.push("unregistered_agent".to_string());
        let result = registry.validate_agents(&invalid_ids);
        prop_assert!(result.is_err(), "Unregistered agent should fail validation");
    }

    #[test]
    fn property_agent_membership_queries(
        agent_id in "[a-z]{3,10}",
        council_id in "[a-z]{3,10}",
    ) {
        use hudhudscript_governance::agent_integration::AgentMembershipTracker;

        let mut tracker = AgentMembershipTracker::new();

        let council = Council {
            id: council_id.clone(),
            name: "Test Council".to_string(),
            constitution_id: "cons.1".to_string(),
            members: vec![
                AgentMember {
                    agent_id: agent_id.clone(),
                    role: AgentRole::Member,
                    joined_at: Utc::now(),
                    permissions: vec![],
                },
            ],
            rules: vec![],
            state: CouncilState {
                active: true,
                metadata: HashMap::new(),
            },
            governance_model: None,
        };

        tracker.track_council(&council);

        let councils = tracker.get_agent_councils(&agent_id);
        prop_assert_eq!(councils.len(), 1, "Agent should be in one council");
        prop_assert_eq!(&councils[0], &council_id, "Council ID should match");

        let role = tracker.get_agent_role_in_council(&agent_id, &council_id, &council);
        prop_assert_eq!(role, Some("Member".to_string()), "Role should be Member");
    }
}

// ============================================================================
// DEPENDENCY TESTS
// ============================================================================

proptest! {
    #[test]
    fn property_circular_dependency_prevention(
        constitution_ids in prop::collection::hash_set(constitution_id_strategy(), 2..5)
    ) {
        use hudhudscript_governance::dependency::DependencyGraph;

        let constitution_ids: Vec<String> = constitution_ids.into_iter().collect();

        let mut graph = DependencyGraph::new();

        for i in 0..constitution_ids.len() {
            let id = &constitution_ids[i];
            let references = if i > 0 {
                vec![constitution_ids[i - 1].clone()]
            } else {
                vec![]
            };
            graph.add_constitution(id.clone(), references);
        }

        prop_assert!(graph.validate_all().is_ok(), "Acyclic graph should be valid");

        let first_id = &constitution_ids[0];
        let last_id = &constitution_ids[constitution_ids.len() - 1];
        let result = graph.validate_no_cycle(first_id, &[last_id.clone()]);

        prop_assert!(result.is_err(), "Circular dependency should be detected");
    }

    #[test]
    fn property_self_reference_detection(id in constitution_id_strategy()) {
        use hudhudscript_governance::dependency::DependencyGraph;

        let graph = DependencyGraph::new();

        let result = graph.validate_no_cycle(&id, &[id.clone()]);

        prop_assert!(result.is_err(), "Self-reference should be detected as circular dependency");
    }

    #[test]
    fn property_no_false_positives_for_valid_hierarchies(
        constitution_ids in prop::collection::hash_set(constitution_id_strategy(), 1..10)
    ) {
        use hudhudscript_governance::dependency::DependencyGraph;

        let constitution_ids: Vec<String> = constitution_ids.into_iter().collect();

        let mut graph = DependencyGraph::new();

        for i in 0..constitution_ids.len() {
            let id = &constitution_ids[i];
            let references: Vec<String> = if i > 0 {
                constitution_ids[0..i]
                    .iter()
                    .take(2)
                    .cloned()
                    .collect()
            } else {
                vec![]
            };
            graph.add_constitution(id.clone(), references);
        }

        prop_assert!(graph.validate_all().is_ok(), "Valid hierarchy should not be rejected");
    }

    #[test]
    fn property_arbitrary_depth_support(depth in 1usize..20) {
        use hudhudscript_governance::dependency::DependencyGraph;

        let mut graph = DependencyGraph::new();

        let mut constitution_ids = Vec::new();
        for i in 0..=depth {
            constitution_ids.push(format!("cons.{}", i));
        }

        for i in 0..=depth {
            let id = &constitution_ids[i];
            let references = if i > 0 {
                vec![constitution_ids[i - 1].clone()]
            } else {
                vec![]
            };
            graph.add_constitution(id.clone(), references);
        }

        prop_assert!(graph.validate_all().is_ok(), "Arbitrary depth chain should be valid");

        let calculated_depth = graph.get_depth(&constitution_ids[depth]);
        prop_assert_eq!(
            calculated_depth,
            depth,
            "Depth should be correctly calculated for arbitrary depth hierarchies"
        );
    }

    #[test]
    fn property_diamond_hierarchy_support(base_num in 1u32..1000) {
        use hudhudscript_governance::dependency::DependencyGraph;

        let mut graph = DependencyGraph::new();

        let constitution_ids = [format!("cons.{}", base_num),
            format!("cons.{}", base_num + 1),
            format!("cons.{}", base_num + 2),
            format!("cons.{}", base_num + 3)];

        graph.add_constitution(constitution_ids[0].clone(), vec![]);
        graph.add_constitution(constitution_ids[1].clone(), vec![constitution_ids[0].clone()]);
        graph.add_constitution(constitution_ids[2].clone(), vec![constitution_ids[0].clone()]);
        graph.add_constitution(
            constitution_ids[3].clone(),
            vec![constitution_ids[1].clone(), constitution_ids[2].clone()],
        );

        prop_assert!(graph.validate_all().is_ok(), "Diamond hierarchy should be valid");

        let depth = graph.get_depth(&constitution_ids[3]);
        prop_assert_eq!(depth, 2, "Diamond hierarchy should have depth 2");
    }

    #[test]
    fn property_multiple_roots_support(
        constitution_ids in prop::collection::hash_set(constitution_id_strategy(), 3..8)
    ) {
        use hudhudscript_governance::dependency::DependencyGraph;

        let constitution_ids: Vec<String> = constitution_ids.into_iter().collect();

        let mut graph = DependencyGraph::new();

        let mid = constitution_ids.len() / 2;

        for i in 0..mid {
            let id = &constitution_ids[i];
            let references = if i > 0 {
                vec![constitution_ids[i - 1].clone()]
            } else {
                vec![]
            };
            graph.add_constitution(id.clone(), references);
        }

        for i in mid..constitution_ids.len() {
            let id = &constitution_ids[i];
            let references = if i > mid {
                vec![constitution_ids[i - 1].clone()]
            } else {
                vec![]
            };
            graph.add_constitution(id.clone(), references);
        }

        prop_assert!(graph.validate_all().is_ok(), "Multiple independent trees should be valid");
    }
}
