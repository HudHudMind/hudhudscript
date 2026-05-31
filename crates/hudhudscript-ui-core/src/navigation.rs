//! Navigation and routing system (#550)
//!
//! Screen-to-screen navigation with parameter passing, history stack,
//! deep linking, and navigation guards.

use crate::PropValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A navigation route entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub screen: String,
    pub params: HashMap<String, PropValue>,
    pub path: Option<String>,
}

/// Navigation guard result
#[derive(Debug, Clone)]
pub enum GuardResult {
    /// Allow navigation to proceed
    Allow,
    /// Redirect to a different screen
    Redirect(String),
    /// Block navigation
    Block(String),
}

/// Navigation guard trait — called before screen transitions
pub trait NavigationGuard: Send {
    fn check(&self, from: &Route, to: &Route) -> GuardResult;
}

/// Router manages screen navigation and history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Router {
    /// Available screens by name
    screens: Vec<String>,
    /// Navigation history stack
    history: Vec<Route>,
    /// Current active route index in history
    current_index: usize,
    /// Deep link path mappings: path → screen name
    deep_links: HashMap<String, String>,
}

impl Router {
    pub fn new(screens: Vec<String>, entry_screen: &str) -> Self {
        let initial_route = Route {
            screen: entry_screen.to_string(),
            params: HashMap::new(),
            path: None,
        };
        Self {
            screens,
            history: vec![initial_route],
            current_index: 0,
            deep_links: HashMap::new(),
        }
    }

    /// Navigate to a screen with parameters
    pub fn navigate(
        &mut self,
        screen: String,
        params: HashMap<String, PropValue>,
    ) -> Result<&Route, NavigationError> {
        if !self.screens.contains(&screen) {
            return Err(NavigationError::ScreenNotFound(screen));
        }
        // Truncate forward history
        self.history.truncate(self.current_index + 1);
        let route = Route {
            screen,
            params,
            path: None,
        };
        self.history.push(route);
        self.current_index = self.history.len() - 1;
        Ok(&self.history[self.current_index])
    }

    /// Go back to the previous screen
    pub fn back(&mut self) -> Result<&Route, NavigationError> {
        if self.current_index == 0 {
            return Err(NavigationError::NoHistory);
        }
        self.current_index -= 1;
        Ok(&self.history[self.current_index])
    }

    /// Go forward in history
    pub fn forward(&mut self) -> Result<&Route, NavigationError> {
        if self.current_index >= self.history.len() - 1 {
            return Err(NavigationError::NoHistory);
        }
        self.current_index += 1;
        Ok(&self.history[self.current_index])
    }

    /// Get current route
    pub fn current(&self) -> &Route {
        &self.history[self.current_index]
    }

    /// Register a deep link: URL path → screen name
    pub fn add_deep_link(&mut self, path: String, screen: String) {
        self.deep_links.insert(path, screen);
    }

    /// Resolve a deep link path to a screen
    pub fn resolve_deep_link(&self, path: &str) -> Option<&String> {
        self.deep_links.get(path)
    }

    /// Get history length
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Can go back?
    pub fn can_go_back(&self) -> bool {
        self.current_index > 0
    }

    /// Can go forward?
    pub fn can_go_forward(&self) -> bool {
        self.current_index < self.history.len() - 1
    }
}

/// Navigation error
#[derive(Debug, Clone)]
pub enum NavigationError {
    ScreenNotFound(String),
    NoHistory,
    Blocked(String),
}

impl std::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NavigationError::ScreenNotFound(s) => write!(f, "Screen not found: {}", s),
            NavigationError::NoHistory => write!(f, "No history to navigate"),
            NavigationError::Blocked(msg) => write!(f, "Navigation blocked: {}", msg),
        }
    }
}

impl std::error::Error for NavigationError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl NavigationError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            NavigationError::Blocked(..) => hudhudscript_errors::ErrorCode::NavigationBlocked,
            NavigationError::NoHistory => hudhudscript_errors::ErrorCode::NavigationNoHistory,
            NavigationError::ScreenNotFound(..) => {
                hudhudscript_errors::ErrorCode::NavigationScreenNotFound
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

impl From<NavigationError> for hudhudscript_errors::Error {
    fn from(e: NavigationError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
