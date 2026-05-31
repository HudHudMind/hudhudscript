//! Role Management Demo
//!
//! This example demonstrates the agent role system in the governance framework.
//! It shows how to:
//! - Create a role manager
//! - Parse predefined and custom roles
//! - Get default permissions for roles
//! - Define custom roles with permissions
//! - Check role permissions
//!
//! Run with: cargo run --example role_management_demo

use hudhudscript_governance::role::RoleManager;
use hudhudscript_governance::AgentRole;

fn main() {
    println!("=== Agent Role Management Demo ===\n");

    // Create a role manager
    let mut manager = RoleManager::new();

    // 1. Demonstrate predefined roles
    println!("1. Predefined Roles:");
    println!("   Available roles: {:?}", manager.list_predefined_roles());
    println!();

    // 2. Parse role strings
    println!("2. Parsing Role Strings:");
    let prosecutor = manager.parse_role("Prosecutor");
    println!("   'Prosecutor' -> {:?}", prosecutor);

    let judge = manager.parse_role("Judge");
    println!("   'Judge' -> {:?}", judge);

    let custom = manager.parse_role("DataValidator");
    println!("   'DataValidator' -> {:?}", custom);

    let invalid = manager.parse_role("");
    println!("   '' (empty) -> {:?} (defaults to Member)", invalid);
    println!();

    // 3. Get default permissions
    println!("3. Default Permissions:");
    let prosecutor_perms = manager.get_default_permissions(&AgentRole::Prosecutor);
    println!("   Prosecutor: {:?}", prosecutor_perms);

    let judge_perms = manager.get_default_permissions(&AgentRole::Judge);
    println!("   Judge: {:?}", judge_perms);

    let executor_perms = manager.get_default_permissions(&AgentRole::Executor);
    println!("   Executor: {:?}", executor_perms);

    let member_perms = manager.get_default_permissions(&AgentRole::Member);
    println!("   Member: {:?}", member_perms);
    println!();

    // 4. Define custom roles
    println!("4. Custom Roles:");
    manager.set_custom_role_permissions(
        "DataValidator".to_string(),
        vec![
            "validate_data".to_string(),
            "read_data".to_string(),
            "read_constitution".to_string(),
        ],
    );
    println!("   Defined 'DataValidator' role with permissions");

    manager.set_custom_role_permissions(
        "ComplianceChecker".to_string(),
        vec![
            "check_compliance".to_string(),
            "read_laws".to_string(),
            "read_constitution".to_string(),
            "generate_report".to_string(),
        ],
    );
    println!("   Defined 'ComplianceChecker' role with permissions");

    let custom_roles = manager.list_custom_roles();
    println!("   Custom roles: {:?}", custom_roles);
    println!();

    // 5. Check permissions
    println!("5. Permission Checks:");
    println!(
        "   Prosecutor has 'propose_action': {}",
        manager.has_permission(&AgentRole::Prosecutor, "propose_action")
    );
    println!(
        "   Member has 'propose_action': {}",
        manager.has_permission(&AgentRole::Member, "propose_action")
    );
    println!(
        "   Judge has 'make_decision': {}",
        manager.has_permission(&AgentRole::Judge, "make_decision")
    );

    let validator_role = AgentRole::Custom("DataValidator".to_string());
    println!(
        "   DataValidator has 'validate_data': {}",
        manager.has_permission(&validator_role, "validate_data")
    );
    println!(
        "   DataValidator has 'execute_decision': {}",
        manager.has_permission(&validator_role, "execute_decision")
    );
    println!();

    // 6. Role information
    println!("6. Role Information:");
    println!(
        "   Prosecutor is predefined: {}",
        manager.is_predefined_role(&AgentRole::Prosecutor)
    );
    println!(
        "   DataValidator is predefined: {}",
        manager.is_predefined_role(&validator_role)
    );
    println!(
        "   Role name for Prosecutor: {}",
        manager.role_name(&AgentRole::Prosecutor)
    );
    println!(
        "   Role name for DataValidator: {}",
        manager.role_name(&validator_role)
    );
    println!();

    // 7. Demonstrate role usage in council context
    println!("7. Council Context Example:");
    println!("   Creating a Data Governance Council with various roles:");
    println!("   - Agent 'alice' as Prosecutor (proposes data policies)");
    println!("   - Agent 'bob' as Judge (evaluates policy compliance)");
    println!("   - Agent 'charlie' as Executor (implements policies)");
    println!("   - Agent 'diana' as DataValidator (validates data quality)");
    println!("   - Agent 'eve' as Member (general participant)");
    println!();

    println!("=== Demo Complete ===");
}
