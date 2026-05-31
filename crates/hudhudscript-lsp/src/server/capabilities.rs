//! Server capabilities.

/// Capabilities advertised by this server
#[derive(Debug, Clone)]
pub struct HudHudServerCapabilities {
    pub completion: bool,
    pub hover: bool,
    pub goto_definition: bool,
    pub formatting: bool,
    pub references: bool,
    pub rename: bool,
}

impl Default for HudHudServerCapabilities {
    fn default() -> Self {
        Self {
            completion: true,
            hover: true,
            goto_definition: true,
            formatting: true,
            references: true,
            rename: true,
        }
    }
}
