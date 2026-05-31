//! Comprehensive governance tests covering: access_control, agent_integration,
//! community, resources, role, swarm.

use hudhudscript_governance::access_control::{AccessControl, Permission};
use hudhudscript_governance::agent_integration::{
    validate_community_agents, validate_council_agents, validate_swarm_agents,
    AgentMembershipTracker, AgentRegistry,
};
use hudhudscript_governance::resources::{AllocationPolicy, Resource, ResourceManager};
use hudhudscript_governance::role::{RoleError, RoleManager};
use hudhudscript_governance::swarm::SwarmError;
use hudhudscript_governance::*;
use serde_json::json;
use std::collections::HashSet;

// ===============================================================================
// AccessControl — construction
// ===============================================================================

#[test]
fn access_control_new_empty() {
    let ac = AccessControl::new();
    assert_eq!(ac.agent_count(), 0);
}

#[test]
fn access_control_default() {
    let ac = AccessControl::default();
    assert_eq!(ac.agent_count(), 0);
}

#[test]
fn access_control_with_cache_limit() {
    let ac = AccessControl::with_cache_limit(500);
    assert_eq!(ac.max_cache_size(), 500);
}

#[test]
fn access_control_default_cache_size() {
    let ac = AccessControl::new();
    assert_eq!(ac.max_cache_size(), 10000);
}

// ===============================================================================
// AccessControl — grant/revoke/check permissions
// ===============================================================================

#[test]
fn grant_permission() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    assert!(ac.has_permission("agent1", &Permission::CreateConstitution));
}

#[test]
fn has_permission_false_by_default() {
    let ac = AccessControl::new();
    assert!(!ac.has_permission("agent1", &Permission::CreateConstitution));
}

#[test]
fn revoke_permission_success() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateCouncil);
    let revoked = ac.revoke_permission("agent1", &Permission::CreateCouncil);
    assert!(revoked);
    assert!(!ac.has_permission("agent1", &Permission::CreateCouncil));
}

#[test]
fn revoke_permission_not_found() {
    let mut ac = AccessControl::new();
    let revoked = ac.revoke_permission("agent1", &Permission::CreateCouncil);
    assert!(!revoked);
}

#[test]
fn revoke_permission_wrong_permission() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    let revoked = ac.revoke_permission("agent1", &Permission::DeleteConstitution);
    assert!(!revoked);
    assert!(ac.has_permission("agent1", &Permission::CreateConstitution));
}

#[test]
fn get_permissions_empty() {
    let ac = AccessControl::new();
    let perms = ac.get_permissions("agent1");
    assert!(perms.is_empty());
}

#[test]
fn get_permissions_multiple() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    ac.grant_permission("agent1".to_string(), Permission::ModifyConstitution);
    let perms = ac.get_permissions("agent1");
    assert_eq!(perms.len(), 2);
}

#[test]
fn verify_permission_ok() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::ViewGovernance);
    assert!(ac
        .verify_permission("agent1", &Permission::ViewGovernance)
        .is_ok());
}

#[test]
fn verify_permission_denied() {
    let ac = AccessControl::new();
    assert!(ac
        .verify_permission("agent1", &Permission::ViewGovernance)
        .is_err());
}

// ===============================================================================
// AccessControl — clear operations
// ===============================================================================

#[test]
fn clear_agent_permissions() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    ac.grant_permission("agent1".to_string(), Permission::DeleteConstitution);
    ac.clear_agent_permissions("agent1");
    assert_eq!(ac.agent_count(), 0);
    assert!(ac.get_permissions("agent1").is_empty());
}

#[test]
fn clear_all_permissions() {
    let mut ac = AccessControl::new();
    ac.grant_permission("agent1".to_string(), Permission::CreateConstitution);
    ac.grant_permission("agent2".to_string(), Permission::DeleteCouncil);
    ac.clear_all_permissions();
    assert_eq!(ac.agent_count(), 0);
}

#[test]
fn agent_count() {
    let mut ac = AccessControl::new();
    ac.grant_permission("a1".to_string(), Permission::CreateRule);
    ac.grant_permission("a2".to_string(), Permission::ModifyRule);
    assert_eq!(ac.agent_count(), 2);
}

// ===============================================================================
// AccessControl — strict validation & ID validation
// ===============================================================================

#[test]
fn set_strict_validation() {
    let mut ac = AccessControl::new();
    ac.set_strict_validation(false);
    // With strict off, any ID should pass
    let result = ac.validate_constitution_id("anything");
    assert!(result.is_ok());
}

#[test]
fn validate_constitution_id_valid() {
    let ac = AccessControl::new();
    let result = ac.validate_constitution_id("cons.1");
    assert!(result.is_ok());
}

#[test]
fn validate_constitution_id_invalid_strict() {
    let ac = AccessControl::new();
    let result = ac.validate_constitution_id("invalid!!!");
    assert!(result.is_err());
}

#[test]
fn validate_law_id_valid() {
    let ac = AccessControl::new();
    let result = ac.validate_law_id("cons1.law1");
    assert!(result.is_ok());
}

#[test]
fn validate_rule_id_valid() {
    let ac = AccessControl::new();
    let result = ac.validate_rule_id("rule.1");
    assert!(result.is_ok());
}

#[test]
fn validate_cache_size_within_limit() {
    let ac = AccessControl::new();
    assert!(ac.validate_cache_size(100).is_ok());
}

#[test]
fn validate_cache_size_exceeds_limit() {
    let ac = AccessControl::with_cache_limit(100);
    assert!(ac.validate_cache_size(100).is_err());
    assert!(ac.validate_cache_size(200).is_err());
}

// ===============================================================================
// AccessControl — all Permission variants
// ===============================================================================

#[test]
fn all_permission_variants_grant_check() {
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
    for (i, perm) in perms.iter().enumerate() {
        let agent_id = format!("agent_{}", i);
        ac.grant_permission(agent_id.clone(), perm.clone());
        assert!(ac.has_permission(&agent_id, perm));
    }
}

// ===============================================================================
// AgentRegistry
// ===============================================================================

#[test]
fn agent_registry_new_empty() {
    let reg = AgentRegistry::new();
    assert!(reg.get_all_agents().is_empty());
}

#[test]
fn agent_registry_register() {
    let mut reg = AgentRegistry::new();
    assert!(reg.register_agent("agent1".to_string()).is_ok());
    assert!(reg.agent_exists(&"agent1".to_string()));
}

#[test]
fn agent_registry_register_duplicate() {
    let mut reg = AgentRegistry::new();
    reg.register_agent("agent1".to_string()).unwrap();
    assert!(reg.register_agent("agent1".to_string()).is_err());
}

#[test]
fn agent_registry_unregister() {
    let mut reg = AgentRegistry::new();
    reg.register_agent("agent1".to_string()).unwrap();
    assert!(reg.unregister_agent(&"agent1".to_string()).is_ok());
    assert!(!reg.agent_exists(&"agent1".to_string()));
}

#[test]
fn agent_registry_unregister_not_found() {
    let mut reg = AgentRegistry::new();
    assert!(reg.unregister_agent(&"ghost".to_string()).is_err());
}

#[test]
fn agent_registry_validate_all_exist() {
    let mut reg = AgentRegistry::new();
    reg.register_agent("a1".to_string()).unwrap();
    reg.register_agent("a2".to_string()).unwrap();
    assert!(reg
        .validate_agents(&["a1".to_string(), "a2".to_string()])
        .is_ok());
}

#[test]
fn agent_registry_validate_missing() {
    let mut reg = AgentRegistry::new();
    reg.register_agent("a1".to_string()).unwrap();
    assert!(reg
        .validate_agents(&["a1".to_string(), "a2".to_string()])
        .is_err());
}

#[test]
fn agent_registry_get_all() {
    let mut reg = AgentRegistry::new();
    reg.register_agent("a1".to_string()).unwrap();
    reg.register_agent("a2".to_string()).unwrap();
    let all = reg.get_all_agents();
    assert_eq!(all.len(), 2);
}

// ===============================================================================
// AgentMembershipTracker
// ===============================================================================

#[test]
fn membership_tracker_new_empty() {
    let tracker = AgentMembershipTracker::new();
    assert!(!tracker.is_member_of_any(&"agent1".to_string()));
}

#[test]
fn membership_tracker_no_councils() {
    let tracker = AgentMembershipTracker::new();
    assert!(tracker.get_agent_councils(&"agent1".to_string()).is_empty());
}

#[test]
fn membership_tracker_no_swarms() {
    let tracker = AgentMembershipTracker::new();
    assert!(tracker.get_agent_swarms(&"agent1".to_string()).is_empty());
}

#[test]
fn membership_tracker_no_communities() {
    let tracker = AgentMembershipTracker::new();
    assert!(tracker
        .get_agent_communities(&"agent1".to_string())
        .is_empty());
}

// ===============================================================================
// Community
// ===============================================================================

fn make_culture() -> CommunityCulture {
    CommunityCulture::new(
        vec!["collaboration".to_string()],
        vec!["respect".to_string()],
        CommunicationStyle::Collaborative,
    )
}

#[test]
fn community_new_ok() {
    let c = Community::new(
        "comm.1".to_string(),
        "Test Community".to_string(),
        vec!["a1".to_string(), "a2".to_string()],
        vec!["council.1".to_string()],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert_eq!(c.members.len(), 2);
    assert_eq!(c.name, "Test Community");
}

#[test]
fn community_new_duplicate_members_fail() {
    let result = Community::new(
        "comm.1".to_string(),
        "Bad".to_string(),
        vec!["a1".to_string(), "a1".to_string()],
        vec![],
        vec![],
        make_culture(),
    );
    assert!(result.is_err());
}

#[test]
fn community_add_member() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec![],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.add_member("a2".to_string()).is_ok());
    assert_eq!(c.members.len(), 2);
}

#[test]
fn community_add_member_duplicate() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec![],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.add_member("a1".to_string()).is_err());
}

#[test]
fn community_remove_member() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string(), "a2".to_string()],
        vec![],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.remove_member(&"a1".to_string()).is_ok());
    assert_eq!(c.members.len(), 1);
}

#[test]
fn community_remove_member_not_found() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec![],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.remove_member(&"ghost".to_string()).is_err());
}

#[test]
fn community_is_member() {
    let c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec![],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.is_member(&"a1".to_string()));
    assert!(!c.is_member(&"a2".to_string()));
}

#[test]
fn community_add_council() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec![],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.add_council("council.1".to_string()).is_ok());
    assert_eq!(c.councils.len(), 1);
}

#[test]
fn community_add_council_duplicate() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec!["council.1".to_string()],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.add_council("council.1".to_string()).is_err());
}

#[test]
fn community_remove_council() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec!["council.1".to_string()],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.remove_council(&"council.1".to_string()).is_ok());
    assert!(c.councils.is_empty());
}

#[test]
fn community_remove_council_not_found() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec![],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert!(c.remove_council(&"missing".to_string()).is_err());
}

#[test]
fn community_get_communication_style() {
    let c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec![],
        vec![],
        make_culture(),
    )
    .unwrap();
    assert_eq!(
        c.get_communication_style(),
        CommunicationStyle::Collaborative
    );
}

#[test]
fn community_update_culture() {
    let mut c = Community::new(
        "c1".to_string(),
        "C".to_string(),
        vec!["a1".to_string()],
        vec![],
        vec![],
        CommunityCulture::new(vec![], vec![], CommunicationStyle::Formal),
    )
    .unwrap();
    c.update_culture(CommunityCulture::new(
        vec!["new_value".to_string()],
        vec![],
        CommunicationStyle::Technical,
    ));
    assert_eq!(c.get_communication_style(), CommunicationStyle::Technical);
}

// ===============================================================================
// CommunityCulture
// ===============================================================================

#[test]
fn community_culture_new() {
    let culture = CommunityCulture::new(
        vec!["transparency".to_string()],
        vec!["honesty".to_string()],
        CommunicationStyle::Formal,
    );
    assert_eq!(culture.values.len(), 1);
    assert_eq!(culture.norms.len(), 1);
    assert_eq!(culture.communication_style, CommunicationStyle::Formal);
}

#[test]
fn community_culture_add_value() {
    let mut culture = CommunityCulture::new(vec![], vec![], CommunicationStyle::Informal);
    culture.add_value("trust".to_string());
    assert_eq!(culture.values.len(), 1);
    culture.add_value("trust".to_string()); // duplicate
    assert_eq!(culture.values.len(), 1);
}

#[test]
fn community_culture_add_norm() {
    let mut culture = CommunityCulture::new(vec![], vec![], CommunicationStyle::Technical);
    culture.add_norm("be kind".to_string());
    assert_eq!(culture.norms.len(), 1);
    culture.add_norm("be kind".to_string()); // duplicate
    assert_eq!(culture.norms.len(), 1);
}

#[test]
fn community_culture_set_communication_style() {
    let mut culture = CommunityCulture::new(vec![], vec![], CommunicationStyle::Formal);
    culture.set_communication_style(CommunicationStyle::Collaborative);
    assert_eq!(
        culture.communication_style,
        CommunicationStyle::Collaborative
    );
}

// ===============================================================================
// Resources — Resource
// ===============================================================================

#[test]
fn resource_new() {
    let r = Resource::new(
        "r1".to_string(),
        "CPU".to_string(),
        Some("CPU cores".to_string()),
        10,
        AllocationPolicy::FairShare,
    );
    assert_eq!(r.capacity, 10);
    assert_eq!(r.available, 10);
    assert_eq!(r.allocation_policy, AllocationPolicy::FairShare);
}

#[test]
fn resource_allocate_ok() {
    let mut r = Resource::new(
        "r1".to_string(),
        "mem".to_string(),
        None,
        100,
        AllocationPolicy::FirstComeFirstServed,
    );
    assert!(r.allocate(50).is_ok());
    assert_eq!(r.available, 50);
}

#[test]
fn resource_allocate_exceeds() {
    let mut r = Resource::new(
        "r1".to_string(),
        "mem".to_string(),
        None,
        100,
        AllocationPolicy::FirstComeFirstServed,
    );
    assert!(r.allocate(101).is_err());
}

#[test]
fn resource_release_ok() {
    let mut r = Resource::new(
        "r1".to_string(),
        "mem".to_string(),
        None,
        100,
        AllocationPolicy::FirstComeFirstServed,
    );
    r.allocate(50).unwrap();
    assert!(r.release(50).is_ok());
    assert_eq!(r.available, 100);
}

#[test]
fn resource_release_exceeds_capacity() {
    let mut r = Resource::new(
        "r1".to_string(),
        "mem".to_string(),
        None,
        100,
        AllocationPolicy::FirstComeFirstServed,
    );
    assert!(r.release(1).is_err());
}

#[test]
fn resource_is_available() {
    let r = Resource::new(
        "r1".to_string(),
        "mem".to_string(),
        None,
        10,
        AllocationPolicy::PriorityBased,
    );
    assert!(r.is_available(10));
    assert!(r.is_available(5));
    assert!(!r.is_available(11));
}

// ===============================================================================
// Resources — ResourceManager
// ===============================================================================

#[test]
fn resource_manager_new() {
    let rm = ResourceManager::new();
    assert!(rm.validate_resources(&[]).is_ok());
}

#[test]
fn resource_manager_default() {
    let rm = ResourceManager::default();
    assert!(rm.validate_resources(&[]).is_ok());
}

#[test]
fn resource_manager_register() {
    let mut rm = ResourceManager::new();
    let r = Resource::new(
        "r1".to_string(),
        "CPU".to_string(),
        None,
        10,
        AllocationPolicy::FairShare,
    );
    assert!(rm.register_resource(r).is_ok());
}

#[test]
fn resource_manager_register_duplicate() {
    let mut rm = ResourceManager::new();
    let r = Resource::new(
        "r1".to_string(),
        "CPU".to_string(),
        None,
        10,
        AllocationPolicy::FairShare,
    );
    rm.register_resource(r.clone()).unwrap();
    assert!(rm.register_resource(r).is_err());
}

#[test]
fn resource_manager_grant_access() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        10,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    assert!(rm
        .grant_access(&"r1".to_string(), "agent1".to_string())
        .is_ok());
    assert!(rm.has_access(&"r1".to_string(), &"agent1".to_string()));
}

#[test]
fn resource_manager_grant_access_nonexistent_resource() {
    let mut rm = ResourceManager::new();
    assert!(rm
        .grant_access(&"missing".to_string(), "agent1".to_string())
        .is_err());
}

#[test]
fn resource_manager_grant_access_duplicate() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        10,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    rm.grant_access(&"r1".to_string(), "agent1".to_string())
        .unwrap();
    assert!(rm
        .grant_access(&"r1".to_string(), "agent1".to_string())
        .is_err());
}

#[test]
fn resource_manager_revoke_access() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        10,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    rm.grant_access(&"r1".to_string(), "agent1".to_string())
        .unwrap();
    assert!(rm
        .revoke_access(&"r1".to_string(), &"agent1".to_string())
        .is_ok());
    assert!(!rm.has_access(&"r1".to_string(), &"agent1".to_string()));
}

#[test]
fn resource_manager_allocate_resource() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    rm.grant_access(&"r1".to_string(), "a1".to_string())
        .unwrap();
    assert!(rm
        .allocate_resource(&"r1".to_string(), &"a1".to_string(), 50)
        .is_ok());
}

#[test]
fn resource_manager_allocate_no_access() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    assert!(rm
        .allocate_resource(&"r1".to_string(), &"a1".to_string(), 10)
        .is_err());
}

#[test]
fn resource_manager_release_resource() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    rm.grant_access(&"r1".to_string(), "a1".to_string())
        .unwrap();
    rm.allocate_resource(&"r1".to_string(), &"a1".to_string(), 50)
        .unwrap();
    assert!(rm
        .release_resource(&"r1".to_string(), &"a1".to_string(), 50)
        .is_ok());
}

#[test]
fn resource_manager_validate_resources() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        10,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    assert!(rm.validate_resources(&["r1".to_string()]).is_ok());
    assert!(rm.validate_resources(&["r2".to_string()]).is_err());
}

#[test]
fn resource_manager_get_agent_usage() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    rm.grant_access(&"r1".to_string(), "a1".to_string())
        .unwrap();
    rm.allocate_resource(&"r1".to_string(), &"a1".to_string(), 10)
        .unwrap();
    let usage = rm.get_agent_usage(&"a1".to_string());
    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].amount, 10);
}

#[test]
fn resource_manager_get_total_usage() {
    let mut rm = ResourceManager::new();
    rm.register_resource(Resource::new(
        "r1".to_string(),
        "X".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    ))
    .unwrap();
    rm.grant_access(&"r1".to_string(), "a1".to_string())
        .unwrap();
    rm.allocate_resource(&"r1".to_string(), &"a1".to_string(), 10)
        .unwrap();
    rm.allocate_resource(&"r1".to_string(), &"a1".to_string(), 20)
        .unwrap();
    assert_eq!(rm.get_total_usage(&"r1".to_string(), &"a1".to_string()), 30);
}

// ===============================================================================
// RoleManager
// ===============================================================================

#[test]
fn role_manager_new() {
    let rm = RoleManager::new();
    let roles = rm.list_predefined_roles();
    assert_eq!(roles.len(), 4);
}

#[test]
fn role_manager_default() {
    let rm = RoleManager::default();
    let roles = rm.list_predefined_roles();
    assert_eq!(roles.len(), 4);
}

#[test]
fn role_manager_parse_predefined() {
    let rm = RoleManager::new();
    assert_eq!(rm.parse_role("Prosecutor"), AgentRole::Prosecutor);
    assert_eq!(rm.parse_role("Judge"), AgentRole::Judge);
    assert_eq!(rm.parse_role("Executor"), AgentRole::Executor);
    assert_eq!(rm.parse_role("Member"), AgentRole::Member);
}

#[test]
fn role_manager_parse_custom() {
    let rm = RoleManager::new();
    assert_eq!(
        rm.parse_role("DataValidator"),
        AgentRole::Custom("DataValidator".to_string())
    );
}

#[test]
fn role_manager_parse_empty_defaults_to_member() {
    let rm = RoleManager::new();
    assert_eq!(rm.parse_role(""), AgentRole::Member);
}

#[test]
fn role_manager_validate_role() {
    let rm = RoleManager::new();
    assert_eq!(
        rm.validate_role(AgentRole::Prosecutor),
        AgentRole::Prosecutor
    );
    assert_eq!(
        rm.validate_role(AgentRole::Custom("X".to_string())),
        AgentRole::Custom("X".to_string())
    );
}

#[test]
fn role_manager_get_default_permissions_prosecutor() {
    let rm = RoleManager::new();
    let perms = rm.get_default_permissions(&AgentRole::Prosecutor);
    assert!(perms.contains(&"propose_action".to_string()));
    assert!(perms.contains(&"read_constitution".to_string()));
}

#[test]
fn role_manager_get_default_permissions_judge() {
    let rm = RoleManager::new();
    let perms = rm.get_default_permissions(&AgentRole::Judge);
    assert!(perms.contains(&"evaluate_compliance".to_string()));
    assert!(perms.contains(&"make_decision".to_string()));
}

#[test]
fn role_manager_get_default_permissions_executor() {
    let rm = RoleManager::new();
    let perms = rm.get_default_permissions(&AgentRole::Executor);
    assert!(perms.contains(&"execute_decision".to_string()));
}

#[test]
fn role_manager_get_default_permissions_member() {
    let rm = RoleManager::new();
    let perms = rm.get_default_permissions(&AgentRole::Member);
    assert!(perms.contains(&"read_constitution".to_string()));
    assert!(perms.contains(&"read_laws".to_string()));
}

#[test]
fn role_manager_custom_role_no_default_permissions() {
    let rm = RoleManager::new();
    let perms = rm.get_default_permissions(&AgentRole::Custom("X".to_string()));
    assert!(perms.is_empty());
}

#[test]
fn role_manager_set_custom_role_permissions() {
    let mut rm = RoleManager::new();
    rm.set_custom_role_permissions(
        "Auditor".to_string(),
        vec!["audit".to_string(), "read".to_string()],
    );
    let perms = rm.get_custom_role_permissions("Auditor");
    assert_eq!(perms.len(), 2);
}

#[test]
fn role_manager_has_permission() {
    let rm = RoleManager::new();
    assert!(rm.has_permission(&AgentRole::Prosecutor, "propose_action"));
    assert!(!rm.has_permission(&AgentRole::Member, "propose_action"));
}

#[test]
fn role_manager_role_name() {
    let rm = RoleManager::new();
    assert_eq!(rm.role_name(&AgentRole::Prosecutor), "Prosecutor");
    assert_eq!(rm.role_name(&AgentRole::Custom("X".to_string())), "X");
}

#[test]
fn role_manager_is_predefined() {
    let rm = RoleManager::new();
    assert!(rm.is_predefined_role(&AgentRole::Prosecutor));
    assert!(rm.is_predefined_role(&AgentRole::Judge));
    assert!(!rm.is_predefined_role(&AgentRole::Custom("X".to_string())));
}

#[test]
fn role_manager_list_custom_roles() {
    let mut rm = RoleManager::new();
    assert!(rm.list_custom_roles().is_empty());
    rm.set_custom_role_permissions("Auditor".to_string(), vec![]);
    let custom = rm.list_custom_roles();
    assert!(custom.contains(&"Auditor".to_string()));
}

// ===============================================================================
// Swarm
// ===============================================================================

#[test]
fn swarm_new() {
    let s = Swarm::new(
        "s1".to_string(),
        "Test Swarm".to_string(),
        vec!["a1".to_string(), "a2".to_string()],
        CoordinationStrategy::Parallel,
    );
    assert_eq!(s.agents.len(), 2);
    assert_eq!(s.get_strategy(), CoordinationStrategy::Parallel);
    assert!(s.shared_state.is_empty());
}

#[test]
fn swarm_add_agent() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec![],
        CoordinationStrategy::Sequential,
    );
    assert!(s.add_agent("a1".to_string()).is_ok());
    assert_eq!(s.agent_count(), 1);
}

#[test]
fn swarm_add_agent_duplicate() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec!["a1".to_string()],
        CoordinationStrategy::Parallel,
    );
    assert!(s.add_agent("a1".to_string()).is_err());
}

#[test]
fn swarm_remove_agent() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec!["a1".to_string(), "a2".to_string()],
        CoordinationStrategy::Parallel,
    );
    assert!(s.remove_agent("a1").is_ok());
    assert_eq!(s.agent_count(), 1);
}

#[test]
fn swarm_remove_agent_not_found() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec!["a1".to_string()],
        CoordinationStrategy::Parallel,
    );
    assert!(s.remove_agent("ghost").is_err());
}

#[test]
fn swarm_has_agent() {
    let s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec!["a1".to_string()],
        CoordinationStrategy::Collaborative,
    );
    assert!(s.has_agent("a1"));
    assert!(!s.has_agent("a2"));
}

#[test]
fn swarm_get_set_strategy() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec![],
        CoordinationStrategy::Parallel,
    );
    assert_eq!(s.get_strategy(), CoordinationStrategy::Parallel);
    s.set_strategy(CoordinationStrategy::Competitive);
    assert_eq!(s.get_strategy(), CoordinationStrategy::Competitive);
}

#[test]
fn swarm_shared_state_set_get() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec![],
        CoordinationStrategy::Parallel,
    );
    s.set_shared_state("key".to_string(), json!(42));
    assert_eq!(s.get_shared_state("key"), Some(&json!(42)));
}

#[test]
fn swarm_shared_state_missing() {
    let s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec![],
        CoordinationStrategy::Parallel,
    );
    assert_eq!(s.get_shared_state("missing"), None);
}

#[test]
fn swarm_remove_shared_state() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec![],
        CoordinationStrategy::Parallel,
    );
    s.set_shared_state("key".to_string(), json!("val"));
    assert!(s.remove_shared_state("key").is_ok());
    assert!(s.get_shared_state("key").is_none());
}

#[test]
fn swarm_remove_shared_state_not_found() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec![],
        CoordinationStrategy::Parallel,
    );
    assert!(s.remove_shared_state("missing").is_err());
}

#[test]
fn swarm_clear_shared_state() {
    let mut s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec![],
        CoordinationStrategy::Parallel,
    );
    s.set_shared_state("k1".to_string(), json!(1));
    s.set_shared_state("k2".to_string(), json!(2));
    s.clear_shared_state();
    assert!(s.shared_state.is_empty());
}

#[test]
fn swarm_validate_agents_ok() {
    let s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec!["a1".to_string(), "a2".to_string()],
        CoordinationStrategy::Parallel,
    );
    let mut valid = HashSet::new();
    valid.insert("a1".to_string());
    valid.insert("a2".to_string());
    assert!(s.validate_agents(&valid).is_ok());
}

#[test]
fn swarm_validate_agents_missing() {
    let s = Swarm::new(
        "s1".to_string(),
        "S".to_string(),
        vec!["a1".to_string(), "a2".to_string()],
        CoordinationStrategy::Parallel,
    );
    let mut valid = HashSet::new();
    valid.insert("a1".to_string());
    assert!(s.validate_agents(&valid).is_err());
}

// ===============================================================================
// SwarmError
// ===============================================================================

#[test]
fn swarm_error_agent_not_found_display() {
    let e = SwarmError::AgentNotFound("ghost".to_string());
    assert!(e.to_string().contains("ghost"));
}

#[test]
fn swarm_error_duplicate_agent_display() {
    let e = SwarmError::DuplicateAgent("dup".to_string());
    assert!(e.to_string().contains("dup"));
}

#[test]
fn swarm_error_state_key_not_found_display() {
    let e = SwarmError::StateKeyNotFound("missing_key".to_string());
    assert!(e.to_string().contains("missing_key"));
}

// ===============================================================================
// CoordinationStrategy
// ===============================================================================

#[test]
fn coordination_strategy_eq() {
    assert_eq!(
        CoordinationStrategy::Parallel,
        CoordinationStrategy::Parallel
    );
    assert_eq!(
        CoordinationStrategy::Sequential,
        CoordinationStrategy::Sequential
    );
    assert_eq!(
        CoordinationStrategy::Competitive,
        CoordinationStrategy::Competitive
    );
    assert_eq!(
        CoordinationStrategy::Collaborative,
        CoordinationStrategy::Collaborative
    );
    assert_ne!(
        CoordinationStrategy::Parallel,
        CoordinationStrategy::Sequential
    );
}

// ===============================================================================
// CommunicationStyle
// ===============================================================================

#[test]
fn communication_style_eq() {
    assert_eq!(CommunicationStyle::Formal, CommunicationStyle::Formal);
    assert_eq!(CommunicationStyle::Informal, CommunicationStyle::Informal);
    assert_eq!(CommunicationStyle::Technical, CommunicationStyle::Technical);
    assert_eq!(
        CommunicationStyle::Collaborative,
        CommunicationStyle::Collaborative
    );
    assert_ne!(CommunicationStyle::Formal, CommunicationStyle::Informal);
}

// ===============================================================================
// AllocationPolicy
// ===============================================================================

#[test]
fn allocation_policy_eq() {
    assert_eq!(
        AllocationPolicy::FirstComeFirstServed,
        AllocationPolicy::FirstComeFirstServed
    );
    assert_eq!(
        AllocationPolicy::PriorityBased,
        AllocationPolicy::PriorityBased
    );
    assert_eq!(AllocationPolicy::FairShare, AllocationPolicy::FairShare);
    assert_ne!(
        AllocationPolicy::FirstComeFirstServed,
        AllocationPolicy::FairShare
    );
}
