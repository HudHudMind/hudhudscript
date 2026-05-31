//! Unsupported-framework bridge — returns honest errors instead of fake success.
//!
//! v0.4.47.9: previously this returned `Ok(())` for init/render/update/shutdown
//! and `Ok(None)` for poll_event, silently masking the absence of a real
//! framework binding. Now it returns explicit `BridgeError::Unsupported` errors
//! so callers know the framework isn't available.
//!
//! This bridge is used as a fallback when the requested framework (e.g.
//! "qt", "gtk") doesn't have a real implementation. The CLI/runtime should
//! check `is_supported()` before instantiating, or handle the error gracefully.

use hudhudscript_ui_core::{App, BridgeError, UIBridge, UIEvent, UIUpdate};

pub struct StubBridge {
    pub name: String,
    pub running: bool,
}

impl StubBridge {
    pub fn new(framework: String) -> Self {
        Self {
            name: format!("unsupported({})", framework),
            running: false,
        }
    }

    fn unsupported<T>(&self, op: &str) -> Result<T, BridgeError> {
        Err(BridgeError::Unsupported(format!(
            "{}: cannot {} — no real implementation for this framework. \
             Supported frameworks: tauri, wasm. \
             Use a different framework or implement {} for {}.",
            self.name, op, op, self.name
        )))
    }
}

impl UIBridge for StubBridge {
    fn init(&mut self) -> Result<(), BridgeError> {
        self.unsupported("init")
    }

    fn render(&mut self, _app: &App) -> Result<(), BridgeError> {
        self.unsupported("render")
    }

    fn update(&mut self, _update: &UIUpdate) -> Result<(), BridgeError> {
        self.unsupported("update")
    }

    fn poll_event(&mut self) -> Result<Option<UIEvent>, BridgeError> {
        // poll_event returning Ok(None) was the worst lie — it pretended events
        // were being checked and finding nothing. Now it returns an explicit
        // error so the caller knows polling is impossible.
        Err(BridgeError::Unsupported(format!(
            "{}: cannot poll events — no real implementation",
            self.name
        )))
    }

    fn shutdown(&mut self) -> Result<(), BridgeError> {
        // Shutdown is the only operation that legitimately succeeds without
        // doing anything: shutting down a non-running bridge is a no-op.
        self.running = false;
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
