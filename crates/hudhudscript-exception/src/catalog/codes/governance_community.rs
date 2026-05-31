use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum GovernanceCommunityExceptionCode {
    /// E0027 — Council not associated with this community
    CommunityCouncilNotFound = 27,
    /// E0028 — Council already attached to community
    CommunityDuplicateCouncil = 28,
    /// E0029 — Agent already a member of this community
    CommunityDuplicateMember = 29,
    /// E0030 — Resource already registered in community
    CommunityDuplicateResource = 30,
    /// E0031 — Invalid community configuration
    CommunityInvalidConfiguration = 31,
    /// E0032 — Agent not a member of this community
    CommunityMemberNotFound = 32,
    /// E0033 — Resource key not registered in community
    CommunityResourceNotFound = 33,
}
