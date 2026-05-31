use hudhudscript_governance::constitution::{ConstitutionError, ConstitutionManager};
use hudhudscript_governance::*;

// ═══════════════════════════════════════════════════════════════════════════════
// ConstitutionManager::new
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn new_sets_version_1() {
    let mgr = ConstitutionManager::new(
        "cons.1".to_string(),
        "Alpha".to_string(),
        Some("desc".to_string()),
    );
    assert_eq!(mgr.current_version(), 1);
}

#[test]
fn new_sets_id() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "Alpha".to_string(), None);
    assert_eq!(mgr.current().id, "cons.1");
}

#[test]
fn new_sets_name() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "Alpha".to_string(), None);
    assert_eq!(mgr.current().name, "Alpha");
}

#[test]
fn new_sets_description_some() {
    let mgr = ConstitutionManager::new(
        "cons.1".to_string(),
        "A".to_string(),
        Some("desc".to_string()),
    );
    assert_eq!(mgr.current().description, Some("desc".to_string()));
}

#[test]
fn new_sets_description_none() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "A".to_string(), None);
    assert_eq!(mgr.current().description, None);
}

#[test]
fn new_has_empty_laws() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "A".to_string(), None);
    assert_eq!(mgr.current().laws.len(), 0);
}

#[test]
fn new_is_in_history() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "A".to_string(), None);
    assert!(mgr.get_version(1).is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// available_versions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn available_versions_starts_with_one() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    assert_eq!(mgr.available_versions(), vec![1]);
}

#[test]
fn available_versions_after_modify() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.name = "V2".to_string());
    assert_eq!(mgr.available_versions(), vec![1, 2]);
}

#[test]
fn available_versions_sorted_after_many_modifications() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    for i in 2..=5 {
        mgr.modify(|c| c.name = format!("V{}", i));
    }
    assert_eq!(mgr.available_versions(), vec![1, 2, 3, 4, 5]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// modify
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn modify_increments_version() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    let v = mgr.modify(|c| c.name = "V2".to_string());
    assert_eq!(v, 2);
    assert_eq!(mgr.current_version(), 2);
}

#[test]
fn modify_applies_mutation() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.name = "Updated".to_string());
    assert_eq!(mgr.current().name, "Updated");
}

#[test]
fn modify_preserves_history() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.name = "V2".to_string());
    mgr.modify(|c| c.name = "V3".to_string());
    assert_eq!(mgr.get_version(1).unwrap().name, "V1");
    assert_eq!(mgr.get_version(2).unwrap().name, "V2");
    assert_eq!(mgr.get_version(3).unwrap().name, "V3");
}

#[test]
fn modify_preserves_id() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.name = "V2".to_string());
    assert_eq!(mgr.current().id, "cons.1");
}

#[test]
fn modify_description_persists() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.description = Some("new desc".to_string()));
    assert_eq!(mgr.current().description, Some("new desc".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// add_law
// ═══════════════════════════════════════════════════════════════════════════════

fn make_law(id: &str, cons_id: &str) -> Law {
    Law {
        id: id.to_string(),
        constitution_id: cons_id.to_string(),
        name: format!("Law {}", id),
        description: "test law".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    }
}

#[test]
fn add_law_increments_version() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    let v = mgr.add_law(make_law("cons1.law1", "cons.1"));
    assert_eq!(v, 2);
}

#[test]
fn add_law_stores_law() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    mgr.add_law(make_law("cons1.law1", "cons.1"));
    assert_eq!(mgr.current().laws.len(), 1);
    assert!(mgr.current().laws.contains_key("cons1.law1"));
}

#[test]
fn add_law_preserves_previous_laws() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    mgr.add_law(make_law("cons1.law1", "cons.1"));
    mgr.add_law(make_law("cons1.law2", "cons.1"));
    assert_eq!(mgr.current().laws.len(), 2);
}

#[test]
fn add_law_with_advisory_level() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    let mut law = make_law("cons1.law1", "cons.1");
    law.enforcement_level = EnforcementLevel::Advisory;
    mgr.add_law(law);
    let stored = mgr.current().laws.get("cons1.law1").unwrap();
    assert_eq!(stored.enforcement_level, EnforcementLevel::Advisory);
}

#[test]
fn add_law_with_conditions() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    let mut law = make_law("cons1.law1", "cons.1");
    law.conditions = vec![Condition::Equals {
        field: "status".to_string(),
        value: serde_json::json!("active"),
    }];
    mgr.add_law(law);
    let stored = mgr.current().laws.get("cons1.law1").unwrap();
    assert_eq!(stored.conditions.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// remove_law
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn remove_law_success() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    mgr.add_law(make_law("cons1.law1", "cons.1"));
    let v = mgr.remove_law(&"cons1.law1".to_string()).unwrap();
    assert_eq!(v, 3);
    assert_eq!(mgr.current().laws.len(), 0);
}

#[test]
fn remove_law_not_found() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    let result = mgr.remove_law(&"cons1.law99".to_string());
    assert!(matches!(result, Err(ConstitutionError::LawNotFound(_))));
}

#[test]
fn remove_law_preserves_other_laws() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    mgr.add_law(make_law("cons1.law1", "cons.1"));
    mgr.add_law(make_law("cons1.law2", "cons.1"));
    mgr.remove_law(&"cons1.law1".to_string()).unwrap();
    assert_eq!(mgr.current().laws.len(), 1);
    assert!(mgr.current().laws.contains_key("cons1.law2"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// update_law
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn update_law_modifies_name() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    mgr.add_law(make_law("cons1.law1", "cons.1"));
    mgr.update_law(&"cons1.law1".to_string(), |l| {
        l.name = "Updated".to_string();
    })
    .unwrap();
    assert_eq!(
        mgr.current().laws.get("cons1.law1").unwrap().name,
        "Updated"
    );
}

#[test]
fn update_law_modifies_enforcement_level() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    mgr.add_law(make_law("cons1.law1", "cons.1"));
    let v = mgr
        .update_law(&"cons1.law1".to_string(), |l| {
            l.enforcement_level = EnforcementLevel::Advisory;
        })
        .unwrap();
    assert_eq!(v, 3);
    let updated = mgr.current().laws.get("cons1.law1").unwrap();
    assert_eq!(updated.enforcement_level, EnforcementLevel::Advisory);
}

#[test]
fn update_nonexistent_law_error() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    let result = mgr.update_law(&"cons1.law99".to_string(), |l| {
        l.name = "X".to_string();
    });
    assert!(matches!(result, Err(ConstitutionError::LawNotFound(_))));
}

// ═══════════════════════════════════════════════════════════════════════════════
// rollback
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn rollback_to_version_1() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.name = "V2".to_string());
    mgr.modify(|c| c.name = "V3".to_string());
    mgr.rollback(1).unwrap();
    assert_eq!(mgr.current_version(), 1);
    assert_eq!(mgr.current().name, "V1");
}

#[test]
fn rollback_to_version_2() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.name = "V2".to_string());
    mgr.modify(|c| c.name = "V3".to_string());
    mgr.rollback(2).unwrap();
    assert_eq!(mgr.current_version(), 2);
    assert_eq!(mgr.current().name, "V2");
}

#[test]
fn rollback_invalid_version() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    let result = mgr.rollback(99);
    assert!(matches!(result, Err(ConstitutionError::InvalidVersion(99))));
}

#[test]
fn rollback_preserves_laws_at_that_version() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    mgr.add_law(make_law("cons1.law1", "cons.1")); // v2
    mgr.add_law(make_law("cons1.law2", "cons.1")); // v3
    mgr.rollback(2).unwrap();
    assert_eq!(mgr.current().laws.len(), 1);
    assert!(mgr.current().laws.contains_key("cons1.law1"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// rollback_previous
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn rollback_previous_success() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.name = "V2".to_string());
    mgr.modify(|c| c.name = "V3".to_string());
    let v = mgr.rollback_previous().unwrap();
    assert_eq!(v, 2);
    assert_eq!(mgr.current().name, "V2");
}

#[test]
fn rollback_previous_at_v1_errors() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    let result = mgr.rollback_previous();
    assert!(matches!(result, Err(ConstitutionError::NoPreviousVersion)));
}

#[test]
fn rollback_previous_from_v2_to_v1() {
    let mut mgr = ConstitutionManager::new("cons.1".to_string(), "V1".to_string(), None);
    mgr.modify(|c| c.name = "V2".to_string());
    let v = mgr.rollback_previous().unwrap();
    assert_eq!(v, 1);
    assert_eq!(mgr.current().name, "V1");
}

// ═══════════════════════════════════════════════════════════════════════════════
// get_version / get_version_timestamp
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn get_version_existent() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    assert!(mgr.get_version(1).is_some());
}

#[test]
fn get_version_nonexistent() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    assert!(mgr.get_version(99).is_none());
}

#[test]
fn get_version_timestamp_existent() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    assert!(mgr.get_version_timestamp(1).is_some());
}

#[test]
fn get_version_timestamp_nonexistent() {
    let mgr = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);
    assert!(mgr.get_version_timestamp(99).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConstitutionError Display
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn constitution_error_not_found_display() {
    let err = ConstitutionError::NotFound("cons.99".to_string());
    assert!(err.to_string().contains("Constitution not found: cons.99"));
}

#[test]
fn constitution_error_invalid_version_display() {
    let err = ConstitutionError::InvalidVersion(42);
    assert!(err.to_string().contains("Invalid version number: 42"));
}

#[test]
fn constitution_error_law_not_found_display() {
    let err = ConstitutionError::LawNotFound("law.99".to_string());
    assert!(err.to_string().contains("Law not found: law.99"));
}

#[test]
fn constitution_error_no_previous_version_display() {
    let err = ConstitutionError::NoPreviousVersion;
    assert!(err.to_string().contains("No previous version available"));
}

#[test]
fn constitution_error_is_std_error() {
    let err = ConstitutionError::NoPreviousVersion;
    let _: &dyn std::error::Error = &err;
}

#[test]
fn constitution_error_eq() {
    assert_eq!(
        ConstitutionError::NoPreviousVersion,
        ConstitutionError::NoPreviousVersion
    );
    assert_eq!(
        ConstitutionError::InvalidVersion(1),
        ConstitutionError::InvalidVersion(1)
    );
    assert_ne!(
        ConstitutionError::InvalidVersion(1),
        ConstitutionError::InvalidVersion(2)
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Comprehensive bundled tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_constitution_comprehensive_state() {
    // Verify all properties of a newly created ConstitutionManager at once
    let mgr = ConstitutionManager::new(
        "cons.comprehensive".to_string(),
        "Comprehensive Test".to_string(),
        Some("A thorough description".to_string()),
    );

    let current = mgr.current();
    assert_eq!(current.id, "cons.comprehensive");
    assert_eq!(current.name, "Comprehensive Test");
    assert_eq!(
        current.description,
        Some("A thorough description".to_string())
    );
    assert_eq!(current.version, 1);
    assert!(current.laws.is_empty());
    assert_eq!(mgr.current_version(), 1);
    assert_eq!(mgr.available_versions(), vec![1]);
    assert!(mgr.get_version(1).is_some());
    assert!(mgr.get_version(2).is_none());
    assert!(mgr.get_version_timestamp(1).is_some());

    // Verify the version 1 snapshot matches current
    let v1 = mgr.get_version(1).unwrap();
    assert_eq!(v1.id, current.id);
    assert_eq!(v1.name, current.name);
    assert_eq!(v1.description, current.description);
    assert_eq!(v1.laws.len(), current.laws.len());
}

#[test]
fn test_amendment_lifecycle() {
    // Full lifecycle: create -> add laws -> update law -> remove law -> verify history
    let mut mgr = ConstitutionManager::new(
        "cons.lifecycle".to_string(),
        "Lifecycle Test".to_string(),
        None,
    );

    // Phase 1: Add two laws (v2, v3)
    let v2 = mgr.add_law(Law {
        id: "lifecycle.law1".to_string(),
        constitution_id: "cons.lifecycle".to_string(),
        name: "First Law".to_string(),
        description: "mandatory law".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![Condition::Equals {
            field: "type".to_string(),
            value: serde_json::json!("action"),
        }],
    });
    assert_eq!(v2, 2);

    let v3 = mgr.add_law(Law {
        id: "lifecycle.law2".to_string(),
        constitution_id: "cons.lifecycle".to_string(),
        name: "Second Law".to_string(),
        description: "advisory law".to_string(),
        enforcement_level: EnforcementLevel::Advisory,
        conditions: vec![],
    });
    assert_eq!(v3, 3);
    assert_eq!(mgr.current().laws.len(), 2);

    // Phase 2: Update a law (v4)
    let v4 = mgr
        .update_law(&"lifecycle.law1".to_string(), |l| {
            l.enforcement_level = EnforcementLevel::Optional;
            l.name = "First Law (Amended)".to_string();
            l.conditions.push(Condition::GreaterThan {
                field: "priority".to_string(),
                value: 5.0,
            });
        })
        .unwrap();
    assert_eq!(v4, 4);

    let updated_law = mgr.current().laws.get("lifecycle.law1").unwrap();
    assert_eq!(updated_law.enforcement_level, EnforcementLevel::Optional);
    assert_eq!(updated_law.name, "First Law (Amended)");
    assert_eq!(updated_law.conditions.len(), 2);

    // Phase 3: Remove a law (v5)
    let v5 = mgr.remove_law(&"lifecycle.law2".to_string()).unwrap();
    assert_eq!(v5, 5);
    assert_eq!(mgr.current().laws.len(), 1);
    assert!(!mgr.current().laws.contains_key("lifecycle.law2"));
    assert!(mgr.current().laws.contains_key("lifecycle.law1"));

    // Phase 4: Verify full history is intact
    assert_eq!(mgr.available_versions(), vec![1, 2, 3, 4, 5]);
    assert_eq!(mgr.get_version(1).unwrap().laws.len(), 0);
    assert_eq!(mgr.get_version(2).unwrap().laws.len(), 1);
    assert_eq!(mgr.get_version(3).unwrap().laws.len(), 2);
    assert_eq!(mgr.get_version(4).unwrap().laws.len(), 2);
    assert_eq!(mgr.get_version(5).unwrap().laws.len(), 1);

    // History version 2 should have the original law name
    assert_eq!(
        mgr.get_version(2)
            .unwrap()
            .laws
            .get("lifecycle.law1")
            .unwrap()
            .name,
        "First Law"
    );
}

#[test]
fn test_rollback_full_cycle() {
    // Test rollback behaviors: forward modifications, rollback, then modify again
    let mut mgr = ConstitutionManager::new(
        "cons.rollback".to_string(),
        "V1-Original".to_string(),
        Some("original".to_string()),
    );

    // Build up versions with laws
    mgr.modify(|c| c.name = "V2-Modified".to_string());
    mgr.add_law(make_law("rb.law1", "cons.rollback")); // v3
    mgr.modify(|c| c.description = Some("v4 desc".to_string())); // v4
    assert_eq!(mgr.current_version(), 4);
    assert_eq!(mgr.current().laws.len(), 1);

    // Rollback to v2 (before law was added)
    mgr.rollback(2).unwrap();
    assert_eq!(mgr.current_version(), 2);
    assert_eq!(mgr.current().name, "V2-Modified");
    assert_eq!(mgr.current().laws.len(), 0);
    // History should still contain all versions
    assert!(mgr.get_version(3).is_some());
    assert!(mgr.get_version(4).is_some());

    // rollback_previous from v2 -> v1
    let v = mgr.rollback_previous().unwrap();
    assert_eq!(v, 1);
    assert_eq!(mgr.current().name, "V1-Original");
    assert_eq!(mgr.current().description, Some("original".to_string()));

    // Rollback to invalid version
    let err = mgr.rollback(999);
    assert!(matches!(err, Err(ConstitutionError::InvalidVersion(999))));

    // Rollback_previous at v1 should error
    let err = mgr.rollback_previous();
    assert!(matches!(err, Err(ConstitutionError::NoPreviousVersion)));

    // After rollback to v1, we can still rollback forward to v4
    mgr.rollback(4).unwrap();
    assert_eq!(mgr.current_version(), 4);
    assert_eq!(mgr.current().laws.len(), 1);
    assert_eq!(mgr.current().description, Some("v4 desc".to_string()));
}

#[test]
fn test_governance_error_cases() {
    let mut mgr =
        ConstitutionManager::new("cons.errors".to_string(), "Error Test".to_string(), None);

    // remove_law on empty constitution
    let err = mgr.remove_law(&"nonexistent".to_string());
    assert!(matches!(err, Err(ConstitutionError::LawNotFound(ref id)) if id == "nonexistent"));

    // update_law on empty constitution
    let err = mgr.update_law(&"nonexistent".to_string(), |l| l.name = "X".to_string());
    assert!(matches!(err, Err(ConstitutionError::LawNotFound(ref id)) if id == "nonexistent"));

    // rollback to version 0 (never exists)
    let err = mgr.rollback(0);
    assert!(matches!(err, Err(ConstitutionError::InvalidVersion(0))));

    // rollback_previous at v1
    let err = mgr.rollback_previous();
    assert!(matches!(err, Err(ConstitutionError::NoPreviousVersion)));

    // Add a law, then try to remove it twice
    mgr.add_law(make_law("err.law1", "cons.errors"));
    mgr.remove_law(&"err.law1".to_string()).unwrap();
    let err = mgr.remove_law(&"err.law1".to_string());
    assert!(matches!(err, Err(ConstitutionError::LawNotFound(_))));

    // All error variants have proper Display output (non-empty)
    let errors = vec![
        ConstitutionError::NotFound("x".to_string()),
        ConstitutionError::InvalidVersion(0),
        ConstitutionError::LawNotFound("y".to_string()),
        ConstitutionError::NoPreviousVersion,
    ];
    for err in &errors {
        let msg = err.to_string();
        assert!(!msg.is_empty(), "Error display should not be empty");
        // All should implement std::error::Error
        let _: &dyn std::error::Error = err;
    }
}

#[test]
fn test_law_management_comprehensive() {
    // Test adding, querying, and updating laws with various enforcement levels and conditions
    let mut mgr =
        ConstitutionManager::new("cons.laws".to_string(), "Law Management".to_string(), None);

    // Add laws with different enforcement levels
    let mandatory_law = Law {
        id: "law.mandatory".to_string(),
        constitution_id: "cons.laws".to_string(),
        name: "Mandatory Rule".to_string(),
        description: "must follow".to_string(),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![
            Condition::Equals {
                field: "status".to_string(),
                value: serde_json::json!("active"),
            },
            Condition::GreaterThan {
                field: "score".to_string(),
                value: 0.5,
            },
        ],
    };

    let advisory_law = Law {
        id: "law.advisory".to_string(),
        constitution_id: "cons.laws".to_string(),
        name: "Advisory Rule".to_string(),
        description: "should follow".to_string(),
        enforcement_level: EnforcementLevel::Advisory,
        conditions: vec![],
    };

    let optional_law = Law {
        id: "law.optional".to_string(),
        constitution_id: "cons.laws".to_string(),
        name: "Optional Rule".to_string(),
        description: "may follow".to_string(),
        enforcement_level: EnforcementLevel::Optional,
        conditions: vec![Condition::In {
            field: "category".to_string(),
            values: vec![serde_json::json!("a"), serde_json::json!("b")],
        }],
    };

    mgr.add_law(mandatory_law);
    mgr.add_law(advisory_law);
    mgr.add_law(optional_law);

    assert_eq!(mgr.current_version(), 4); // v1 + 3 adds
    assert_eq!(mgr.current().laws.len(), 3);

    // Verify each law's properties
    let m = mgr.current().laws.get("law.mandatory").unwrap();
    assert_eq!(m.enforcement_level, EnforcementLevel::Mandatory);
    assert_eq!(m.conditions.len(), 2);
    assert_eq!(m.constitution_id, "cons.laws");

    let a = mgr.current().laws.get("law.advisory").unwrap();
    assert_eq!(a.enforcement_level, EnforcementLevel::Advisory);
    assert!(a.conditions.is_empty());

    let o = mgr.current().laws.get("law.optional").unwrap();
    assert_eq!(o.enforcement_level, EnforcementLevel::Optional);
    assert_eq!(o.conditions.len(), 1);

    // Update advisory to mandatory
    mgr.update_law(&"law.advisory".to_string(), |l| {
        l.enforcement_level = EnforcementLevel::Mandatory;
        l.description = "now mandatory".to_string();
    })
    .unwrap();

    let updated = mgr.current().laws.get("law.advisory").unwrap();
    assert_eq!(updated.enforcement_level, EnforcementLevel::Mandatory);
    assert_eq!(updated.description, "now mandatory");
    // Other laws unchanged
    assert_eq!(
        mgr.current()
            .laws
            .get("law.mandatory")
            .unwrap()
            .enforcement_level,
        EnforcementLevel::Mandatory,
    );
    assert_eq!(
        mgr.current()
            .laws
            .get("law.optional")
            .unwrap()
            .enforcement_level,
        EnforcementLevel::Optional,
    );
}

#[test]
fn test_version_timestamps_ordering() {
    // Verify that timestamps exist for all versions and are retrievable
    let mut mgr =
        ConstitutionManager::new("cons.time".to_string(), "Timestamp Test".to_string(), None);

    mgr.modify(|c| c.name = "V2".to_string());
    mgr.modify(|c| c.name = "V3".to_string());
    mgr.modify(|c| c.name = "V4".to_string());

    // All versions should have timestamps
    for v in 1..=4 {
        assert!(
            mgr.get_version_timestamp(v).is_some(),
            "Version {} should have a timestamp",
            v,
        );
        assert!(mgr.get_version(v).is_some(), "Version {} should exist", v,);
    }

    // Non-existent versions should return None
    assert!(mgr.get_version_timestamp(5).is_none());
    assert!(mgr.get_version_timestamp(0).is_none());
    assert!(mgr.get_version(5).is_none());
    assert!(mgr.get_version(0).is_none());

    // Timestamps should be non-decreasing (v1 <= v2 <= v3 <= v4)
    let t1 = mgr.get_version_timestamp(1).unwrap();
    let t2 = mgr.get_version_timestamp(2).unwrap();
    let t3 = mgr.get_version_timestamp(3).unwrap();
    let t4 = mgr.get_version_timestamp(4).unwrap();
    assert!(t1 <= t2);
    assert!(t2 <= t3);
    assert!(t3 <= t4);
}

#[test]
fn test_modify_preserves_complete_state() {
    // Ensure modify only changes what the closure changes, preserving everything else
    let mut mgr = ConstitutionManager::new(
        "cons.preserve".to_string(),
        "Preserve Test".to_string(),
        Some("original desc".to_string()),
    );

    // Add a law first
    mgr.add_law(make_law("preserve.law1", "cons.preserve"));

    // Modify only the name
    mgr.modify(|c| c.name = "New Name".to_string());

    let current = mgr.current();
    assert_eq!(current.name, "New Name");
    assert_eq!(current.id, "cons.preserve"); // id preserved
    assert_eq!(current.description, Some("original desc".to_string())); // desc preserved
    assert_eq!(current.laws.len(), 1); // law preserved
    assert!(current.laws.contains_key("preserve.law1")); // specific law preserved
    assert_eq!(current.version, 3); // v1 -> add_law(v2) -> modify(v3)

    // Modify only the description
    mgr.modify(|c| c.description = None);
    assert_eq!(mgr.current().name, "New Name"); // name preserved
    assert_eq!(mgr.current().description, None); // desc changed
    assert_eq!(mgr.current().laws.len(), 1); // law preserved
}
