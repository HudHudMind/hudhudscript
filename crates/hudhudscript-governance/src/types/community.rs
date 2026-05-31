//! Community types

use serde::{Deserialize, Serialize};

use super::{AgentId, CommunityId, CouncilId, ResourceId};

/// Community: Social group of agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Community {
    pub id: CommunityId,
    pub name: String,
    pub members: Vec<AgentId>,
    pub councils: Vec<CouncilId>,
    pub shared_resources: Vec<ResourceId>,
    pub culture: CommunityCulture,
}

/// Community culture definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommunityCulture {
    pub values: Vec<String>,
    pub norms: Vec<String>,
    pub communication_style: CommunicationStyle,
}

/// Communication style for communities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommunicationStyle {
    Formal,
    Informal,
    Technical,
    Collaborative,
}
