//! Community Management
//!
//! This module provides functionality for managing communities of agents,
//! including member management, council associations, and culture definitions.

use crate::types::{
    AgentId, CommunicationStyle, Community, CommunityCulture, CommunityId, CouncilId, ResourceId,
};
use std::collections::HashSet;

/// Errors that can occur during community management.
///
/// (v0.4.47.9 — Issue #846: replaces previous Result<T, String>)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunityError {
    DuplicateMember(String),
    MemberNotFound(String),
    DuplicateCouncil(String),
    CouncilNotFound(String),
    DuplicateResource(String),
    ResourceNotFound(String),
    InvalidConfiguration(String),
}

impl std::fmt::Display for CommunityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            CommunityError::DuplicateMember(s) => write!(f, "Member '{}' already in community", s),
            CommunityError::MemberNotFound(s) => write!(f, "Member '{}' not found in community", s),
            CommunityError::DuplicateCouncil(s) => {
                write!(f, "Council '{}' already associated with this community", s)
            }
            CommunityError::CouncilNotFound(s) => {
                write!(f, "Council '{}' not associated with this community", s)
            }
            CommunityError::DuplicateResource(s) => {
                write!(f, "Resource '{}' already in community", s)
            }
            CommunityError::ResourceNotFound(s) => {
                write!(f, "Resource '{}' not found in community", s)
            }
            CommunityError::InvalidConfiguration(s) => write!(f, "Invalid configuration: {}", s),
        }
    }
}

impl std::error::Error for CommunityError {}

impl Community {
    /// Create a new community with the specified parameters
    ///
    /// # Arguments
    /// * `id` - Unique community identifier
    /// * `name` - Human-readable community name
    /// * `members` - List of agent IDs (must be unique)
    /// * `councils` - List of associated council IDs
    /// * `shared_resources` - List of shared resource IDs
    /// * `culture` - Community culture definition
    ///
    /// # Returns
    /// * `Ok(Community)` if all members are unique
    /// * `Err(String)` if duplicate agent IDs are found
    ///
    /// # Examples
    /// ```
    /// use hudhudscript_governance::{Community, CommunityCulture, CommunicationStyle};
    ///
    /// let culture = CommunityCulture {
    ///     values: vec!["collaboration".to_string(), "transparency".to_string()],
    ///     norms: vec!["respect".to_string(), "honesty".to_string()],
    ///     communication_style: CommunicationStyle::Collaborative,
    /// };
    ///
    /// let community = Community::new(
    ///     "comm.1".to_string(),
    ///     "Test Community".to_string(),
    ///     vec!["agent1".to_string(), "agent2".to_string()],
    ///     vec!["council.1".to_string()],
    ///     vec![],
    ///     culture,
    /// ).unwrap();
    ///
    /// assert_eq!(community.members.len(), 2);
    /// ```
    pub fn new(
        id: CommunityId,
        name: String,
        members: Vec<AgentId>,
        councils: Vec<CouncilId>,
        shared_resources: Vec<ResourceId>,
        culture: CommunityCulture,
    ) -> Result<Self, CommunityError> {
        // Ensure unique agent IDs within community
        let unique_members: HashSet<_> = members.iter().collect();
        if unique_members.len() != members.len() {
            return Err(CommunityError::InvalidConfiguration(
                "Duplicate agent IDs found in community members".to_string(),
            ));
        }

        Ok(Self {
            id,
            name,
            members,
            councils,
            shared_resources,
            culture,
        })
    }

    /// Add a member to the community
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to add
    ///
    /// # Returns
    /// * `Ok(())` if member was added successfully
    /// * `Err(String)` if member already exists
    pub fn add_member(&mut self, agent_id: AgentId) -> Result<(), CommunityError> {
        if self.members.contains(&agent_id) {
            return Err(CommunityError::DuplicateMember(agent_id.to_string()));
        }
        self.members.push(agent_id);
        Ok(())
    }

    /// Remove a member from the community
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to remove
    ///
    /// # Returns
    /// * `Ok(())` if member was removed successfully
    /// * `Err(String)` if member was not found
    pub fn remove_member(&mut self, agent_id: &AgentId) -> Result<(), CommunityError> {
        let initial_len = self.members.len();
        self.members.retain(|id| id != agent_id);
        if self.members.len() == initial_len {
            return Err(CommunityError::MemberNotFound(agent_id.to_string()));
        }
        Ok(())
    }

    /// Check if an agent is a member of the community
    ///
    /// # Arguments
    /// * `agent_id` - Agent ID to check
    ///
    /// # Returns
    /// * `true` if agent is a member, `false` otherwise
    pub fn is_member(&self, agent_id: &AgentId) -> bool {
        self.members.contains(agent_id)
    }

    /// Associate a council with the community
    ///
    /// # Arguments
    /// * `council_id` - Council ID to associate
    ///
    /// # Returns
    /// * `Ok(())` if council was associated successfully
    /// * `Err(String)` if council is already associated
    pub fn add_council(&mut self, council_id: CouncilId) -> Result<(), CommunityError> {
        if self.councils.contains(&council_id) {
            return Err(CommunityError::DuplicateCouncil(council_id.to_string()));
        }
        self.councils.push(council_id);
        Ok(())
    }

    /// Remove a council association from the community
    ///
    /// # Arguments
    /// * `council_id` - Council ID to remove
    ///
    /// # Returns
    /// * `Ok(())` if council was removed successfully
    /// * `Err(String)` if council was not found
    pub fn remove_council(&mut self, council_id: &CouncilId) -> Result<(), CommunityError> {
        let initial_len = self.councils.len();
        self.councils.retain(|id| id != council_id);
        if self.councils.len() == initial_len {
            return Err(CommunityError::CouncilNotFound(council_id.to_string()));
        }
        Ok(())
    }

    /// Update the community culture
    ///
    /// # Arguments
    /// * `culture` - New culture definition
    pub fn update_culture(&mut self, culture: CommunityCulture) {
        self.culture = culture;
    }

    /// Get the communication style of the community
    ///
    /// # Returns
    /// * The current communication style
    pub fn get_communication_style(&self) -> CommunicationStyle {
        self.culture.communication_style
    }
}

impl CommunityCulture {
    /// Create a new community culture
    ///
    /// # Arguments
    /// * `values` - List of community values
    /// * `norms` - List of community norms
    /// * `communication_style` - Communication style
    ///
    /// # Returns
    /// * New `CommunityCulture` instance
    pub fn new(
        values: Vec<String>,
        norms: Vec<String>,
        communication_style: CommunicationStyle,
    ) -> Self {
        Self {
            values,
            norms,
            communication_style,
        }
    }

    /// Add a value to the community culture
    ///
    /// # Arguments
    /// * `value` - Value to add
    pub fn add_value(&mut self, value: String) {
        if !self.values.contains(&value) {
            self.values.push(value);
        }
    }

    /// Add a norm to the community culture
    ///
    /// # Arguments
    /// * `norm` - Norm to add
    pub fn add_norm(&mut self, norm: String) {
        if !self.norms.contains(&norm) {
            self.norms.push(norm);
        }
    }

    /// Set the communication style
    ///
    /// # Arguments
    /// * `style` - New communication style
    pub fn set_communication_style(&mut self, style: CommunicationStyle) {
        self.communication_style = style;
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl CommunityError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            CommunityError::CouncilNotFound(..) => {
                hudhudscript_errors::ErrorCode::CommunityCouncilNotFound
            }
            CommunityError::DuplicateCouncil(..) => {
                hudhudscript_errors::ErrorCode::CommunityDuplicateCouncil
            }
            CommunityError::DuplicateMember(..) => {
                hudhudscript_errors::ErrorCode::CommunityDuplicateMember
            }
            CommunityError::DuplicateResource(..) => {
                hudhudscript_errors::ErrorCode::CommunityDuplicateResource
            }
            CommunityError::InvalidConfiguration(..) => {
                hudhudscript_errors::ErrorCode::CommunityInvalidConfiguration
            }
            CommunityError::MemberNotFound(..) => {
                hudhudscript_errors::ErrorCode::CommunityMemberNotFound
            }
            CommunityError::ResourceNotFound(..) => {
                hudhudscript_errors::ErrorCode::CommunityResourceNotFound
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<CommunityError> for hudhudscript_errors::Error {
    fn from(e: CommunityError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
