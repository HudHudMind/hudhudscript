//! Mock and stub framework for HudHudScript tests.
//!
//! Provides lightweight building blocks for:
//! - **Function mocks** — record calls and supply canned return values.
//! - **Provider mocks** — simulate external data providers.
//! - **Event mocks** — capture and assert emitted events.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

// ── Function mock ────────────────────────────────────────────────────────

/// A recorded invocation of a mocked function.
#[derive(Debug, Clone)]
pub struct MockCall {
    /// Positional argument representations (stored as debug strings).
    pub args: Vec<String>,
}

/// Generic function mock that records calls and optionally returns canned values.
///
/// The mock is `Clone + Send + Sync` so it can be shared across threads.
#[derive(Clone)]
pub struct FunctionMock {
    name: String,
    calls: Arc<Mutex<Vec<MockCall>>>,
    /// Return value as a string (the scripting layer can interpret it).
    return_value: Arc<Mutex<Option<String>>>,
}

impl Debug for FunctionMock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionMock")
            .field("name", &self.name)
            .field("call_count", &self.call_count())
            .finish()
    }
}

impl FunctionMock {
    /// Create a new function mock with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            calls: Arc::new(Mutex::new(Vec::new())),
            return_value: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the canned return value.
    pub fn returns(&self, value: impl Into<String>) {
        *self.return_value.lock().unwrap() = Some(value.into());
    }

    /// Simulate a call, recording arguments and returning the canned value.
    pub fn call(&self, args: Vec<String>) -> Option<String> {
        self.calls.lock().unwrap().push(MockCall { args });
        self.return_value.lock().unwrap().clone()
    }

    /// Name of the mock.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of times the mock has been called.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Returns `true` if the mock was called at least once.
    pub fn was_called(&self) -> bool {
        self.call_count() > 0
    }

    /// Returns `true` if the mock was called exactly `n` times.
    pub fn was_called_times(&self, n: usize) -> bool {
        self.call_count() == n
    }

    /// Access a snapshot of recorded calls.
    pub fn calls(&self) -> Vec<MockCall> {
        self.calls.lock().unwrap().clone()
    }

    /// Reset all recorded calls and the return value.
    pub fn reset(&self) {
        self.calls.lock().unwrap().clear();
        *self.return_value.lock().unwrap() = None;
    }

    /// Assert that the mock was called at least once with args containing `expected`.
    pub fn assert_called_with(&self, expected: &[&str]) -> Result<(), String> {
        let calls = self.calls.lock().unwrap();
        let found = calls.iter().any(|c| {
            expected
                .iter()
                .all(|e| c.args.iter().any(|a| a.contains(e)))
        });
        if found {
            Ok(())
        } else {
            Err(format!(
                "Expected mock '{}' to have been called with args containing {:?}",
                self.name, expected
            ))
        }
    }
}

// ── Provider mock ────────────────────────────────────────────────────────

/// Simulates an external data provider (database, API, file system, etc.).
///
/// Data is stored as `String -> String` pairs. The scripting runtime can use
/// this to inject test data without touching real infrastructure.
#[derive(Debug, Clone)]
pub struct ProviderMock {
    name: String,
    data: Arc<Mutex<HashMap<String, String>>>,
    access_log: Arc<Mutex<Vec<String>>>,
}

impl ProviderMock {
    /// Create a new provider mock.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: Arc::new(Mutex::new(HashMap::new())),
            access_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Name of the provider.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Seed the provider with a key-value pair.
    pub fn set(&self, key: impl Into<String>, value: impl Into<String>) {
        self.data.lock().unwrap().insert(key.into(), value.into());
    }

    /// Seed multiple key-value pairs at once.
    pub fn set_many(&self, pairs: Vec<(String, String)>) {
        let mut data = self.data.lock().unwrap();
        for (k, v) in pairs {
            data.insert(k, v);
        }
    }

    /// Look up a key, recording the access.
    pub fn get(&self, key: &str) -> Option<String> {
        self.access_log.lock().unwrap().push(key.to_string());
        self.data.lock().unwrap().get(key).cloned()
    }

    /// Keys that have been accessed (in order).
    pub fn access_log(&self) -> Vec<String> {
        self.access_log.lock().unwrap().clone()
    }

    /// Number of accesses recorded.
    pub fn access_count(&self) -> usize {
        self.access_log.lock().unwrap().len()
    }

    /// Reset data and access log.
    pub fn reset(&self) {
        self.data.lock().unwrap().clear();
        self.access_log.lock().unwrap().clear();
    }
}

// ── Event mock ───────────────────────────────────────────────────────────

/// A captured event.
#[derive(Debug, Clone)]
pub struct MockEvent {
    pub name: String,
    pub payload: Option<String>,
}

/// Captures events emitted during a test so they can be asserted on.
#[derive(Debug, Clone)]
pub struct EventMock {
    events: Arc<Mutex<Vec<MockEvent>>>,
}

impl Default for EventMock {
    fn default() -> Self {
        Self::new()
    }
}

impl EventMock {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record an emitted event.
    pub fn emit(&self, name: impl Into<String>, payload: Option<String>) {
        self.events.lock().unwrap().push(MockEvent {
            name: name.into(),
            payload,
        });
    }

    /// Total number of captured events.
    pub fn count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    /// Number of captured events with the given name.
    pub fn count_by_name(&self, name: &str) -> usize {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.name == name)
            .count()
    }

    /// Returns `true` if an event with the given name was emitted.
    pub fn has_event(&self, name: &str) -> bool {
        self.count_by_name(name) > 0
    }

    /// Snapshot of all captured events.
    pub fn events(&self) -> Vec<MockEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Assert that an event with the given name was emitted.
    pub fn assert_emitted(&self, name: &str) -> Result<(), String> {
        if self.has_event(name) {
            Ok(())
        } else {
            Err(format!("Expected event '{}' to have been emitted", name))
        }
    }

    /// Assert that an event with the given name was emitted exactly `n` times.
    pub fn assert_emitted_times(&self, name: &str, n: usize) -> Result<(), String> {
        let actual = self.count_by_name(name);
        if actual == n {
            Ok(())
        } else {
            Err(format!(
                "Expected event '{}' to be emitted {} time(s), got {}",
                name, n, actual
            ))
        }
    }

    /// Reset all captured events.
    pub fn reset(&self) {
        self.events.lock().unwrap().clear();
    }
}
