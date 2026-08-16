//! Declaration parsing (governance, MCP, etc.)
//!
//! This module re-exports declaration parsers from specialized sub-modules.
//! Each declaration type has its own module following SRP (Single Responsibility Principle).

// Re-export all declaration parsers from sub-modules
pub use super::declarations::{
    parse_ability_decl, parse_action_decl, parse_agent_decl, parse_community_decl,
    parse_compose_decl, parse_constitution_decl, parse_council_decl, parse_decorated_subject_decl,
    parse_deploy_decl, parse_effect_decl, parse_governance_decl, parse_import_stmt, parse_law_decl,
    parse_mcp_decl, parse_protocol_decl, parse_provider_decl, parse_relation_decl,
    parse_resource_decl, parse_role_decl, parse_rule_decl, parse_subject_decl, parse_swarm_decl,
    parse_ui_decl,
};
