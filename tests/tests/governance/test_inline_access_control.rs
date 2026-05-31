use hudhudscript_governance::access_control::{AccessControl, Permission};

#[test]
fn test_access_control_creation() {
    let ac = AccessControl::new();
    assert_eq!(ac.max_cache_size(), 10000);
    assert_eq!(ac.agent_count(), 0);
}

#[test]
fn test_access_control_with_cache_limit() {
    let ac = AccessControl::with_cache_limit(5000);
    assert_eq!(ac.max_cache_size(), 5000);
}

#[test]
fn test_grant_permission() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);

    assert!(ac.has_permission("agent1", &Permission::CreateConstitution));
    assert!(!ac.has_permission("agent1", &Permission::ModifyConstitution));
}

#[test]
fn test_revoke_permission() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);

    assert!(ac.has_permission("agent1", &Permission::CreateConstitution));

    let revoked = ac.revoke_permission("agent1", &Permission::CreateConstitution);
    assert!(revoked);
    assert!(!ac.has_permission("agent1", &Permission::CreateConstitution));
}

#[test]
fn test_revoke_nonexistent_permission() {
    let mut ac = AccessControl::new();
    let revoked = ac.revoke_permission("agent1", &Permission::CreateConstitution);
    assert!(!revoked);
}

#[test]
fn test_get_permissions() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    ac.grant_permission("agent1".to_string(), Permission::ModifyConstitution);

    let perms = ac.get_permissions("agent1");
    assert_eq!(perms.len(), 2);
    assert!(perms.contains(&Permission::CreateConstitution));
    assert!(perms.contains(&Permission::ModifyConstitution));
}

#[test]
fn test_get_permissions_empty() {
    let ac = AccessControl::new();
    let perms = ac.get_permissions("agent1");
    assert_eq!(perms.len(), 0);
}

#[test]
fn test_verify_permission_success() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);

    let result = ac.verify_permission("agent1", &Permission::CreateConstitution);
    assert!(result.is_ok());
}

#[test]
fn test_verify_permission_failure() {
    let ac = AccessControl::new();
    let result = ac.verify_permission("agent1", &Permission::CreateConstitution);
    assert!(result.is_err());
}

#[test]
fn test_validate_constitution_id() {
    let ac = AccessControl::new();
    let result = ac.validate_constitution_id("cons.123");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cons.123");
}

#[test]
fn test_validate_constitution_id_invalid() {
    let ac = AccessControl::new();
    let result = ac.validate_constitution_id("invalid");
    assert!(result.is_err());
}

#[test]
fn test_validate_constitution_id_sanitized() {
    let ac = AccessControl::new();
    // Null byte is removed by sanitization, resulting in valid "cons.123"
    let result = ac.validate_constitution_id("cons.123\0");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cons.123");
}

#[test]
fn test_validate_law_id() {
    let ac = AccessControl::new();
    let result = ac.validate_law_id("cons123.law456");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cons123.law456");
}

#[test]
fn test_validate_rule_id() {
    let ac = AccessControl::new();
    let result = ac.validate_rule_id("rule.789");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "rule.789");
}

#[test]
fn test_validate_cache_size_ok() {
    let ac = AccessControl::with_cache_limit(1000);
    let result = ac.validate_cache_size(500);
    assert!(result.is_ok());
}

#[test]
fn test_validate_cache_size_exceeded() {
    let ac = AccessControl::with_cache_limit(1000);
    let result = ac.validate_cache_size(1000);
    assert!(result.is_err());
}

#[test]
fn test_clear_agent_permissions() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    ac.grant_permission("agent1".to_string(), Permission::ModifyConstitution);

    assert_eq!(ac.get_permissions("agent1").len(), 2);

    ac.clear_agent_permissions("agent1");
    assert_eq!(ac.get_permissions("agent1").len(), 0);
}

#[test]
fn test_clear_all_permissions() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    ac.grant_permission("agent2".to_string(), Permission::ModifyConstitution);

    assert_eq!(ac.agent_count(), 2);

    ac.clear_all_permissions();
    assert_eq!(ac.agent_count(), 0);
}

#[test]
fn test_strict_validation_mode() {
    let mut ac = AccessControl::new();
    assert!(ac.strict_validation);

    ac.set_strict_validation(false);
    assert!(!ac.strict_validation);

    // With strict validation off, invalid IDs should still be sanitized
    let result = ac.validate_constitution_id("invalid");
    assert!(result.is_ok()); // No validation, just sanitization
}

#[test]
fn test_multiple_agents() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    ac.grant_permission("agent2".to_string(), Permission::ModifyConstitution);
    ac.grant_permission("agent3".to_string(), Permission::DeleteConstitution);

    assert_eq!(ac.agent_count(), 3);
    assert!(ac.has_permission("agent1", &Permission::CreateConstitution));
    assert!(ac.has_permission("agent2", &Permission::ModifyConstitution));
    assert!(ac.has_permission("agent3", &Permission::DeleteConstitution));
}

#[test]
fn test_validate_law_id_invalid() {
    let ac = AccessControl::new();
    let result = ac.validate_law_id("bad-id");
    assert!(result.is_err());
}

#[test]
fn test_validate_rule_id_invalid() {
    let ac = AccessControl::new();
    let result = ac.validate_rule_id("bad-id");
    assert!(result.is_err());
}

#[test]
fn test_default_trait() {
    let ac = AccessControl::default();
    assert_eq!(ac.max_cache_size(), 10000);
    assert_eq!(ac.agent_count(), 0);
}

#[test]
fn test_all_permission_variants_exist() {
    // Ensure all permission enum variants can be granted and checked
    let mut ac = AccessControl::new();
    let perms = vec![
        Permission::CreateConstitution,
        Permission::ModifyConstitution,
        Permission::DeleteConstitution,
        Permission::CreateCouncil,
        Permission::ModifyCouncil,
        Permission::DeleteCouncil,
        Permission::CreateRule,
        Permission::ModifyRule,
        Permission::DeleteRule,
        Permission::CreateSwarm,
        Permission::ModifySwarm,
        Permission::DeleteSwarm,
        Permission::CreateCommunity,
        Permission::ModifyCommunity,
        Permission::DeleteCommunity,
        Permission::ViewGovernance,
        Permission::EnforceConstitution,
        Permission::AuditGovernance,
    ];
    for perm in &perms {
        ac.grant_permission("agent_all".to_string(), perm.clone());
    }
    assert_eq!(ac.get_permissions("agent_all").len(), perms.len());
}

#[test]
fn test_revoke_permission_when_agent_has_other_perms() {
    let mut ac = AccessControl::new();
    ac.grant_permission("a1".to_string(), Permission::CreateConstitution);
    ac.grant_permission("a1".to_string(), Permission::ModifyConstitution);

    ac.revoke_permission("a1", &Permission::CreateConstitution);
    assert!(!ac.has_permission("a1", &Permission::CreateConstitution));
    assert!(ac.has_permission("a1", &Permission::ModifyConstitution));
}
