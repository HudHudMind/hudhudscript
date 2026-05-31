use super::{CouncilDecision, CouncilMember};

/// Session lifecycle hooks
#[allow(clippy::type_complexity)]
pub struct SessionHooks {
    pub on_start: Option<Box<dyn Fn(&str, &[CouncilMember]) + Send + Sync>>,
    pub on_vote: Option<Box<dyn Fn(&str, Option<bool>) + Send + Sync>>,
    pub on_complete: Option<Box<dyn Fn(&str, &CouncilDecision) + Send + Sync>>,
}
