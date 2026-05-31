//! ID type aliases

/// Constitution ID (e.g., "cons.1", "cons.2")
pub type ConstitutionId = String;

/// Law ID (e.g., "cons1.law1", "cons2.law3")
pub type LawId = String;

/// Rule ID (e.g., "rule.1", "rule.2")
pub type RuleId = String;

/// Council ID (e.g., "council.1")
pub type CouncilId = String;

/// Swarm ID (e.g., "swarm.1")
pub type SwarmId = String;

/// Community ID (e.g., "community.1")
pub type CommunityId = String;

/// Agent ID (e.g., "agent.1")
pub type AgentId = String;

/// Resource ID (e.g., "resource.1")
pub type ResourceId = String;

/// Permission string for role-based access (e.g., "read", "write", "execute").
///
/// This is distinct from `access_control::Permission` which is an enum for
/// governance-level permissions. This alias is used for fine-grained action
/// permissions attached to agent roles inside councils.
pub type PermissionStr = String;
