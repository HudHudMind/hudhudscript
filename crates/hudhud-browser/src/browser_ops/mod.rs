//! Shared browser integration builtins — open URLs, read bookmarks/history/tabs,
//! detect installed browsers, search (Issue #643).
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

mod bookmarks;
mod helpers;
mod history;
mod search;
mod system;
mod tabs;

pub use bookmarks::{browser_bookmarks, extract_json_string};
use helpers::*;
pub use history::browser_history;
pub use search::{browser_search, url_encode};
pub use system::{browser_default_browser, browser_installed_browsers};
pub use tabs::browser_tabs;

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Open,
    Bookmarks,
    History,
    Tabs,
    DefaultBrowser,
    InstalledBrowsers,
    Search,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "bookmarks" => Ok(Self::Bookmarks),
            "history" => Ok(Self::History),
            "tabs" => Ok(Self::Tabs),
            "default_browser" => Ok(Self::DefaultBrowser),
            "installed_browsers" => Ok(Self::InstalledBrowsers),
            "search" => Ok(Self::Search),
            _ => Err(Error::new(
                ErrorCode::CompileRuntimeError,
                format!("Unknown method: {}", s),
            )),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Open => browser_open(args),
        ScriptMethodId::Bookmarks => browser_bookmarks(args),
        ScriptMethodId::History => browser_history(args),
        ScriptMethodId::Tabs => browser_tabs(args),
        ScriptMethodId::DefaultBrowser => browser_default_browser(args),
        ScriptMethodId::InstalledBrowsers => browser_installed_browsers(args),
        ScriptMethodId::Search => browser_search(args),
    }
}

pub fn browser_open(args: &[Value16]) -> HudHudResult<Value16> {
    let url = match args.first() {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), "browser.open"))?
            .to_string(),
        None => return Err(runtime_error("browser.open requires a URL argument")),
    };

    if std::env::var("HUDHUD_NO_BROWSER").is_ok() || cfg!(test) {
        return Ok(Value16::bool_(true));
    }

    let status = Command::new("xdg-open").arg(&url).status();
    match status {
        Ok(s) if s.success() => Ok(Value16::bool_(true)),
        Ok(_) => Ok(Value16::bool_(false)),
        Err(e) => Err(runtime_error(format!(
            "browser.open: failed to launch xdg-open: {}",
            e
        ))),
    }
}
