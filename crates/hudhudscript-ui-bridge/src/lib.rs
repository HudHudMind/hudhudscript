//! HudHudScript UI Bridge — GUI framework adapters
//!
//! Each adapter implements the `UIBridge` trait from ui-core.
//! Framework selection via `--ui` flag or `deploy` config.
//!
//! Supported frameworks (planned):
//! - GTK (Linux native)
//! - Qt (cross-platform native)
//! - Tauri (Rust + WebView)
//! - Iced (Pure Rust)
//! - Web (HTML + WebSocket)
//! - Electron (Node.js + Chromium)
//! - wxWidgets (C++ cross-platform)
//! - Flutter (mobile via FFI)

pub mod fallback;
pub mod flutter_bridge;
pub mod tauri_bridge;
pub mod wasm_bridge;
pub mod web;

use hudhudscript_ui_core::{BridgeError, UIBridge};

/// Available framework backends
#[derive(Debug, Clone)]
pub enum Framework {
    Gtk,
    Qt,
    Tauri,
    Iced,
    Web,
    Electron,
    WxWidgets,
    Flutter,
    Wasm,
    Custom(String),
}

impl Framework {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gtk" => Some(Framework::Gtk),
            "qt" => Some(Framework::Qt),
            "tauri" => Some(Framework::Tauri),
            "iced" => Some(Framework::Iced),
            "web" | "html" => Some(Framework::Web),
            "electron" => Some(Framework::Electron),
            "wxwidgets" | "wx" => Some(Framework::WxWidgets),
            "flutter" => Some(Framework::Flutter),
            "wasm" | "wasm32" | "webassembly" => Some(Framework::Wasm),
            _ => Some(Framework::Custom(s.to_string())),
        }
    }
}

/// Create a bridge for the given framework
pub fn create_bridge(framework: &Framework) -> Result<Box<dyn UIBridge>, BridgeError> {
    match framework {
        Framework::Web => Ok(Box::new(web::WebBridge::new(3000))),
        Framework::Tauri => Ok(Box::new(tauri_bridge::TauriBridge::new())),
        Framework::Flutter => Ok(Box::new(flutter_bridge::FlutterBridge::new())),
        Framework::Wasm => Ok(Box::new(wasm_bridge::WasmBridge::new())),
        _ => Ok(Box::new(fallback::StubBridge::new(format!(
            "{:?}",
            framework
        )))),
    }
}
