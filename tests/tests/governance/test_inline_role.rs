use hudhudscript_governance::role::{RoleError, RoleManager};
use hudhudscript_governance::*;

#[test]
fn test_role_manager_creation() {
    let manager = RoleManager::new();

    // Verify default permissions are set
    let prosecutor_perms = manager.get_default_permissions(&AgentRole::Prosecutor);
    assert!(prosecutor_perms.contains(&"propose_action".to_string()));

    let judge_perms = manager.get_default_permissions(&AgentRole::Judge);
    assert!(judge_perms.contains(&"evaluate_compliance".to_string()));

    let executor_perms = manager.get_default_permissions(&AgentRole::Executor);
    assert!(executor_perms.contains(&"execute_decision".to_string()));

    let member_perms = manager.get_default_permissions(&AgentRole::Member);
    assert!(member_perms.contains(&"read_constitution".to_string()));
}

#[test]
fn test_parse_role_predefined() {
    let manager = RoleManager::new();

    assert_eq!(manager.parse_role("Prosecutor"), AgentRole::Prosecutor);
    assert_eq!(manager.parse_role("Judge"), AgentRole::Judge);
    assert_eq!(manager.parse_role("Executor"), AgentRole::Executor);
    assert_eq!(manager.parse_role("Member"), AgentRole::Member);
}

#[test]
fn test_parse_role_custom() {
    let manager = RoleManager::new();

    let custom = manager.parse_role("DataValidator");
    assert_eq!(custom, AgentRole::Custom("DataValidator".to_string()));

    let another_custom = manager.parse_role("ComplianceChecker");
    assert_eq!(
        another_custom,
        AgentRole::Custom("ComplianceChecker".to_string())
    );
}

#[test]
fn test_parse_role_invalid_defaults_to_member() {
    let manager = RoleManager::new();

    // Empty string defaults to Member
    assert_eq!(manager.parse_role(""), AgentRole::Member);

    // Whitespace-only defaults to Member
    assert_eq!(manager.parse_role("   "), AgentRole::Member);
}

#[test]
fn test_validate_role() {
    let manager = RoleManager::new();

    // All roles are valid
    assert_eq!(
        manager.validate_role(AgentRole::Prosecutor),
        AgentRole::Prosecutor
    );
    assert_eq!(
        manager.validate_role(AgentRole::Custom("Test".to_string())),
        AgentRole::Custom("Test".to_string())
    );
}

#[test]
fn test_get_default_permissions() {
    let manager = RoleManager::new();

    let prosecutor_perms = manager.get_default_permissions(&AgentRole::Prosecutor);
    assert_eq!(prosecutor_perms.len(), 3);
    assert!(prosecutor_perms.contains(&"propose_action".to_string()));

    let judge_perms = manager.get_default_permissions(&AgentRole::Judge);
    assert_eq!(judge_perms.len(), 4);
    assert!(judge_perms.contains(&"make_decision".to_string()));

    let executor_perms = manager.get_default_permissions(&AgentRole::Executor);
    assert_eq!(executor_perms.len(), 3);
    assert!(executor_perms.contains(&"execute_decision".to_string()));

    let member_perms = manager.get_default_permissions(&AgentRole::Member);
    assert_eq!(member_perms.len(), 2);
    assert!(member_perms.contains(&"read_constitution".to_string()));
}

#[test]
fn test_custom_role_permissions() {
    let mut manager = RoleManager::new();

    // Custom roles have no default permissions
    let custom_perms = manager.get_default_permissions(&AgentRole::Custom("Test".to_string()));
    assert_eq!(custom_perms.len(), 0);

    // Set custom role permissions
    manager.set_custom_role_permissions(
        "DataValidator".to_string(),
        vec!["validate_data".to_string(), "read_data".to_string()],
    );

    let perms = manager.get_custom_role_permissions("DataValidator");
    assert_eq!(perms.len(), 2);
    assert!(perms.contains(&"validate_data".to_string()));
}

#[test]
fn test_has_permission() {
    let manager = RoleManager::new();

    assert!(manager.has_permission(&AgentRole::Prosecutor, "propose_action"));
    assert!(!manager.has_permission(&AgentRole::Member, "propose_action"));

    assert!(manager.has_permission(&AgentRole::Judge, "make_decision"));
    assert!(!manager.has_permission(&AgentRole::Executor, "make_decision"));
}

#[test]
fn test_has_permission_custom_role() {
    let mut manager = RoleManager::new();

    manager.set_custom_role_permissions("Validator".to_string(), vec!["validate".to_string()]);

    let custom_role = AgentRole::Custom("Validator".to_string());
    assert!(manager.has_permission(&custom_role, "validate"));
    assert!(!manager.has_permission(&custom_role, "execute"));
}

#[test]
fn test_role_name() {
    let manager = RoleManager::new();

    assert_eq!(manager.role_name(&AgentRole::Prosecutor), "Prosecutor");
    assert_eq!(manager.role_name(&AgentRole::Judge), "Judge");
    assert_eq!(manager.role_name(&AgentRole::Executor), "Executor");
    assert_eq!(manager.role_name(&AgentRole::Member), "Member");
    assert_eq!(
        manager.role_name(&AgentRole::Custom("Test".to_string())),
        "Test"
    );
}

#[test]
fn test_is_predefined_role() {
    let manager = RoleManager::new();

    assert!(manager.is_predefined_role(&AgentRole::Prosecutor));
    assert!(manager.is_predefined_role(&AgentRole::Judge));
    assert!(manager.is_predefined_role(&AgentRole::Executor));
    assert!(manager.is_predefined_role(&AgentRole::Member));
    assert!(!manager.is_predefined_role(&AgentRole::Custom("Test".to_string())));
}

#[test]
fn test_list_predefined_roles() {
    let manager = RoleManager::new();
    let roles = manager.list_predefined_roles();

    assert_eq!(roles.len(), 4);
    assert!(roles.contains(&AgentRole::Prosecutor));
    assert!(roles.contains(&AgentRole::Judge));
    assert!(roles.contains(&AgentRole::Executor));
    assert!(roles.contains(&AgentRole::Member));
}

#[test]
fn test_list_custom_roles() {
    let mut manager = RoleManager::new();

    // Initially no custom roles
    assert_eq!(manager.list_custom_roles().len(), 0);

    // Add custom roles
    manager.set_custom_role_permissions("Validator".to_string(), vec![]);
    manager.set_custom_role_permissions("Auditor".to_string(), vec![]);

    let custom_roles = manager.list_custom_roles();
    assert_eq!(custom_roles.len(), 2);
    assert!(custom_roles.contains(&"Validator".to_string()));
    assert!(custom_roles.contains(&"Auditor".to_string()));
}

#[test]
fn test_role_manager_default() {
    let manager = RoleManager::default();

    // Should be same as new()
    let perms = manager.get_default_permissions(&AgentRole::Prosecutor);
    assert!(perms.contains(&"propose_action".to_string()));
}

#[test]
fn test_role_error_display() {
    let e1 = RoleError::InvalidRole("bad".to_string());
    assert!(format!("{}", e1).contains("Invalid role"));
    assert!(format!("{}", e1).contains("bad"));

    let e2 = RoleError::RoleNotFound("missing".to_string());
    assert!(format!("{}", e2).contains("Role not found"));
    assert!(format!("{}", e2).contains("missing"));

    let e3 = RoleError::PermissionNotFound("perm".to_string());
    assert!(format!("{}", e3).contains("Permission not found"));
    assert!(format!("{}", e3).contains("perm"));
}

#[test]
fn test_role_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(RoleError::InvalidRole("x".to_string()));
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_get_custom_role_permissions_missing() {
    let manager = RoleManager::new();
    let perms = manager.get_custom_role_permissions("NonexistentRole");
    assert!(perms.is_empty());
}

#[test]
fn test_has_permission_custom_role_not_registered() {
    let manager = RoleManager::new();
    let custom_role = AgentRole::Custom("Unknown".to_string());
    assert!(!manager.has_permission(&custom_role, "any_permission"));
}

#[test]
fn test_all_predefined_roles_share_read_constitution() {
    let manager = RoleManager::new();
    for role in &[
        AgentRole::Prosecutor,
        AgentRole::Judge,
        AgentRole::Executor,
        AgentRole::Member,
    ] {
        assert!(
            manager.has_permission(role, "read_constitution"),
            "Role {:?} should have read_constitution",
            role
        );
    }
}

#[test]
fn test_all_predefined_roles_share_read_laws() {
    let manager = RoleManager::new();
    for role in &[
        AgentRole::Prosecutor,
        AgentRole::Judge,
        AgentRole::Executor,
        AgentRole::Member,
    ] {
        assert!(
            manager.has_permission(role, "read_laws"),
            "Role {:?} should have read_laws",
            role
        );
    }
}
