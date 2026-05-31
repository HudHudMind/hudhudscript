//! Tauri desktop adapter (#552)
//!
//! Renders widget tree as HTML for a Tauri WebView window.
//! Uses the same HTML generation as the web bridge but targets a local window.

use hudhudscript_ui_core::*;

pub struct TauriBridge {
    pub name: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub running: bool,
}

impl Default for TauriBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl TauriBridge {
    pub fn new() -> Self {
        Self {
            name: "tauri(desktop)".to_string(),
            title: "HudHudScript App".to_string(),
            width: 1200,
            height: 800,
            running: false,
        }
    }
}

impl UIBridge for TauriBridge {
    fn init(&mut self) -> Result<(), BridgeError> {
        // v0.4.47.9: Returns honest error. The Tauri framework crate is not
        // linked at the workspace level (would add ~50MB of WebKit deps), so
        // the bridge cannot create real windows. To enable Tauri:
        //   1. Add `tauri = "2"` to Cargo.toml
        //   2. Replace this with `tauri::Builder::default().run(...)`
        Err(BridgeError::Unsupported(format!(
            "{}: real Tauri integration requires the `tauri` crate (not linked). \
             Add tauri to Cargo.toml and implement TauriBridge::init() with Builder::default().",
            self.name
        )))
    }

    fn render(&mut self, _app: &App) -> Result<(), BridgeError> {
        Err(BridgeError::Unsupported(format!(
            "{}: cannot render — Tauri runtime not initialized (init() returns Unsupported)",
            self.name
        )))
    }

    fn update(&mut self, _update: &UIUpdate) -> Result<(), BridgeError> {
        Err(BridgeError::Unsupported(format!(
            "{}: cannot update — Tauri IPC not available",
            self.name
        )))
    }

    fn poll_event(&mut self) -> Result<Option<UIEvent>, BridgeError> {
        Err(BridgeError::Unsupported(format!(
            "{}: cannot poll events — Tauri event listener not registered",
            self.name
        )))
    }

    fn shutdown(&mut self) -> Result<(), BridgeError> {
        // Shutdown is the only no-op success — closing a non-running bridge.
        self.running = false;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
