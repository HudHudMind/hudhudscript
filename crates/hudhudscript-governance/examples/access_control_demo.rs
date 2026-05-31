//! Access Control Demo
//!
//! Demonstrates the permission-based access control system

use hudhudscript_governance::access_control::{AccessControl, Permission};

fn main() {
    println!("=== Access Control System Demo ===\n");

    // Create access control manager
    let mut ac = AccessControl::with_cache_limit(5000);
    println!("1. Created access control manager");
    println!("   - Max cache size: 5000 items");
    println!("   - Strict validation: enabled\n");

    // Grant permissions to different agents
    println!("2. Granting permissions to agents:");

    ac.grant_permission("admin".to_string(), Permission::CreateConstitution);
    ac.grant_permission("admin".to_string(), Permission::ModifyConstitution);
    ac.grant_permission("admin".to_string(), Permission::DeleteConstitution);
    ac.grant_permission("admin".to_string(), Permission::CreateCouncil);
    ac.grant_permission("admin".to_string(), Permission::EnforceConstitution);
    println!("   ✓ admin: Full governance permissions");

    ac.grant_permission("developer".to_string(), Permission::CreateRule);
    ac.grant_permission("developer".to_string(), Permission::ModifyRule);
    ac.grant_permission("developer".to_string(), Permission::ViewGovernance);
    println!("   ✓ developer: Rule management + view");

    ac.grant_permission("auditor".to_string(), Permission::ViewGovernance);
    ac.grant_permission("auditor".to_string(), Permission::AuditGovernance);
    println!("   ✓ auditor: View + audit only");

    ac.grant_permission("user".to_string(), Permission::ViewGovernance);
    println!("   ✓ user: View only\n");

    // Check permissions
    println!("3. Checking permissions:");

    let agents = vec!["admin", "developer", "auditor", "user"];
    let permissions = vec![
        Permission::CreateConstitution,
        Permission::CreateRule,
        Permission::ViewGovernance,
        Permission::AuditGovernance,
    ];

    for agent in &agents {
        println!("\n   Agent: {}", agent);
        for perm in &permissions {
            let has_perm = ac.has_permission(agent, perm);
            let symbol = if has_perm { "✓" } else { "✗" };
            println!("     {} {:?}", symbol, perm);
        }
    }

    // Verify permissions (with error handling)
    println!("\n4. Verifying permissions (with error handling):");

    match ac.verify_permission("admin", &Permission::CreateConstitution) {
        Ok(_) => println!("   ✓ admin can create constitutions"),
        Err(e) => println!("   ✗ admin cannot create constitutions: {}", e),
    }

    match ac.verify_permission("user", &Permission::CreateConstitution) {
        Ok(_) => println!("   ✓ user can create constitutions"),
        Err(e) => println!("   ✗ user cannot create constitutions: {}", e),
    }

    // Validate IDs
    println!("\n5. Validating governance IDs:");

    let test_ids = vec![
        ("cons.123", "constitution"),
        ("cons123.law456", "law"),
        ("rule.789", "rule"),
        ("invalid", "constitution"),
    ];

    for (id, id_type) in test_ids {
        let result = match id_type {
            "constitution" => ac.validate_constitution_id(id),
            "law" => ac.validate_law_id(id),
            "rule" => ac.validate_rule_id(id),
            _ => Err(
                hudhudscript_governance::error::GovernanceError::InvalidConfiguration(
                    "Unknown type".to_string(),
                ),
            ),
        };

        match result {
            Ok(validated) => println!("   ✓ {} ID '{}' -> '{}'", id_type, id, validated),
            Err(e) => println!("   ✗ {} ID '{}': {}", id_type, id, e),
        }
    }

    // Cache size validation
    println!("\n6. Cache size validation:");

    let test_sizes = vec![1000, 4999, 5000, 5001];
    for size in test_sizes {
        match ac.validate_cache_size(size) {
            Ok(_) => println!("   ✓ Cache size {} is within limit", size),
            Err(e) => println!("   ✗ Cache size {}: {}", size, e),
        }
    }

    // Revoke permissions
    println!("\n7. Revoking permissions:");

    let revoked = ac.revoke_permission("developer", &Permission::ModifyRule);
    if revoked {
        println!("   ✓ Revoked ModifyRule from developer");
    }

    let has_perm = ac.has_permission("developer", &Permission::ModifyRule);
    println!("   Developer can modify rules: {}", has_perm);

    // Get all permissions for an agent
    println!("\n8. Listing all permissions:");

    for agent in &agents {
        let perms = ac.get_permissions(agent);
        println!("   {}: {} permissions", agent, perms.len());
        for perm in perms {
            println!("     - {:?}", perm);
        }
    }

    // Strict mode control
    println!("\n9. Strict validation mode:");
    println!("   Current mode: strict");

    ac.set_strict_validation(false);
    println!("   Changed to: permissive");

    // In permissive mode, invalid IDs are sanitized but not rejected
    match ac.validate_constitution_id("invalid-id") {
        Ok(sanitized) => println!("   Permissive mode: 'invalid-id' -> '{}'", sanitized),
        Err(e) => println!("   Error: {}", e),
    }

    ac.set_strict_validation(true);
    println!("   Changed back to: strict");

    // Statistics
    println!("\n10. Access control statistics:");
    println!("   - Total agents with permissions: {}", ac.agent_count());
    println!("   - Max cache size: {}", ac.max_cache_size());

    println!("\n=== Demo Complete ===");
    println!("\nKey Features Demonstrated:");
    println!("✓ Permission granting and revoking");
    println!("✓ Permission verification with error handling");
    println!("✓ ID validation and sanitization");
    println!("✓ Cache size limits (DoS prevention)");
    println!("✓ Strict vs permissive validation modes");
    println!("✓ Permission listing and statistics");
}
