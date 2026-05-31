//! Declaration parsers
//!
//! This module contains specialized parsers for different declaration types.
//! Each sub-module handles one type of declaration, following SRP.

pub mod agent;
pub mod common;
pub mod community;
pub mod deploy;
pub mod governance;
pub mod mcp;
pub mod provider;
pub mod resource;
pub mod rule;
pub mod subject;
pub mod compose;
pub mod swarm;
pub mod task;
pub mod ui;

// Re-export main parsing functions for convenience
pub use agent::parse_agent_decl;
pub use community::parse_community_decl;
pub use deploy::parse_deploy_decl;
pub use governance::{
    parse_constitution_decl, parse_council_decl, parse_governance_decl, parse_law_decl,
    parse_protocol_decl, parse_role_decl,
};
pub use mcp::parse_mcp_decl;
pub use provider::{parse_import_stmt, parse_provider_decl};
pub use resource::parse_resource_decl;
pub use rule::parse_rule_decl;
pub use subject::{
    parse_decorated_subject_decl, parse_effect_decl, parse_ability_decl, parse_relation_decl, parse_subject_decl,
};
pub use compose::parse_compose_decl;
pub use swarm::parse_swarm_decl;
pub use task::parse_action_decl;
pub use ui::parse_ui_decl;
