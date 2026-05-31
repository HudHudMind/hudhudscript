//! Audit Logging Demo
//!
//! This example demonstrates the comprehensive audit logging capabilities
//! of the governance system, including enforcement decisions, constitution
//! modifications, agent role changes, and cache access patterns.

use chrono::Utc;
use hudhudscript_governance::audit::AuditLogger;
use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};
use hudhudscript_governance::{AgentRole, Condition, Constitution, EnforcementLevel, Law};
use serde_json::json;
use std::collections::HashMap;

fn main() {
    println!("=== Audit Logging Demo ===\n");

    // Create an audit logger
    let audit_logger = AuditLogger::new();

    // 1. Log enforcement decisions
    println!("1. Logging enforcement decisions...");
    demo_enforcement_logging(&audit_logger);

    // 2. Log constitution modifications
    println!("\n2. Logging constitution modifications...");
    demo_constitution_modification_logging(&audit_logger);

    // 3. Log agent role changes
    println!("\n3. Logging agent role changes...");
    demo_role_change_logging(&audit_logger);

    // 4. Log cache access patterns
    println!("\n4. Logging cache access patterns...");
    demo_cache_access_logging(&audit_logger);

    // 5. Export audit logs
    println!("\n5. Exporting audit logs...");
    demo_audit_export(&audit_logger);

    // 6. Query audit logs
    println!("\n6. Querying audit logs...");
    demo_audit_queries(&audit_logger);

    println!("\n=== Demo Complete ===");
}

fn demo_enforcement_logging(audit_logger: &AuditLogger) {
    // Create a test constitution
    let mut laws = HashMap::new();
    laws.insert(
        "cons1.law1".to_string(),
        Law {
            id: "cons1.law1".to_string(),
            constitution_id: "cons.1".to_string(),
            name: "Data Size Limit".to_string(),
            description: "Data must be under 1000 bytes".to_string(),
            enforcement_level: EnforcementLevel::Mandatory,
            conditions: vec![Condition::LessThan {
                field: "data_size".to_string(),
                value: 1000.0,
            }],
        },
    );

    let constitution = Constitution {
        id: "cons.1".to_string(),
        name: "Data Governance".to_string(),
        description: Some("Governance rules for data processing".to_string()),
        laws,
        created_at: Utc::now(),
        version: 1,
    };

    // Test compliant action
    let mut context1 = EvaluationContext::new();
    context1.insert("data_size".to_string(), json!(500));

    let result1 = enforce_constitution(&constitution, &context1, Some(audit_logger));
    println!("  ✓ Compliant action: allowed = {}", result1.allowed);

    // Test non-compliant action
    let mut context2 = EvaluationContext::new();
    context2.insert("data_size".to_string(), json!(1500));

    let result2 = enforce_constitution(&constitution, &context2, Some(audit_logger));
    println!(
        "  ✗ Non-compliant action: allowed = {}, violations = {:?}",
        result2.allowed, result2.violations
    );
}

fn demo_constitution_modification_logging(audit_logger: &AuditLogger) {
    audit_logger.log_constitution_modification(
        "cons.1".to_string(),
        "law_added".to_string(),
        1,
        2,
        "Added new data protection law".to_string(),
    );
    println!("  ✓ Logged constitution modification: law_added (v1 → v2)");

    audit_logger.log_constitution_modification(
        "cons.1".to_string(),
        "law_updated".to_string(),
        2,
        3,
        "Updated enforcement level for existing law".to_string(),
    );
    println!("  ✓ Logged constitution modification: law_updated (v2 → v3)");
}

fn demo_role_change_logging(audit_logger: &AuditLogger) {
    // New member joining
    audit_logger.log_agent_role_change(
        "agent1".to_string(),
        None,
        AgentRole::Member,
        "council1".to_string(),
    );
    println!("  ✓ Logged new member: agent1 → Member");

    // Role promotion
    audit_logger.log_agent_role_change(
        "agent1".to_string(),
        Some(AgentRole::Member),
        AgentRole::Judge,
        "council1".to_string(),
    );
    println!("  ✓ Logged role change: agent1 Member → Judge");

    // Custom role assignment
    audit_logger.log_agent_role_change(
        "agent2".to_string(),
        None,
        AgentRole::Custom("DataValidator".to_string()),
        "council1".to_string(),
    );
    println!("  ✓ Logged custom role: agent2 → DataValidator");
}

fn demo_cache_access_logging(audit_logger: &AuditLogger) {
    // Cache hits
    audit_logger.log_cache_access("resolve".to_string(), "cons.1".to_string(), true);
    println!("  ✓ Cache hit: resolve cons.1");

    audit_logger.log_cache_access("resolve".to_string(), "rule.1".to_string(), true);
    println!("  ✓ Cache hit: resolve rule.1");

    // Cache miss
    audit_logger.log_cache_access("resolve".to_string(), "cons.999".to_string(), false);
    println!("  ✗ Cache miss: resolve cons.999");

    // Store operation
    audit_logger.log_cache_access("store".to_string(), "cons.2".to_string(), true);
    println!("  ✓ Cache store: cons.2");
}

fn demo_audit_export(audit_logger: &AuditLogger) {
    // Export to pretty JSON
    match audit_logger.export_to_json() {
        Ok(json) => {
            println!("  ✓ Exported {} events to JSON", audit_logger.count());
            println!("  JSON size: {} bytes", json.len());
        }
        Err(e) => println!("  ✗ Export failed: {}", e),
    }

    // Export to compact JSON
    match audit_logger.export_to_json_compact() {
        Ok(json) => {
            println!("  ✓ Exported to compact JSON");
            println!("  Compact JSON size: {} bytes", json.len());
        }
        Err(e) => println!("  ✗ Compact export failed: {}", e),
    }
}

fn demo_audit_queries(audit_logger: &AuditLogger) {
    use hudhudscript_governance::audit::AuditEventType;

    // Query by type
    let enforcement_events = audit_logger.get_events_by_type(AuditEventType::EnforcementDecision);
    println!(
        "  ✓ Enforcement decisions: {} events",
        enforcement_events.len()
    );

    let modification_events = audit_logger.get_events_by_type(AuditEventType::ConstitutionModified);
    println!(
        "  ✓ Constitution modifications: {} events",
        modification_events.len()
    );

    let role_events = audit_logger.get_events_by_type(AuditEventType::AgentRoleChanged);
    println!("  ✓ Role changes: {} events", role_events.len());

    let cache_events = audit_logger.get_events_by_type(AuditEventType::CacheAccess);
    println!("  ✓ Cache accesses: {} events", cache_events.len());

    // Total events
    println!("\n  Total audit events: {}", audit_logger.count());
}
