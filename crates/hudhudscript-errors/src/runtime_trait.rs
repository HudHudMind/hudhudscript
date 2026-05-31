//! Runtime trait — common interface for Interpreter and VM.
//!
//! Both execution engines share these capabilities but implement them
//! differently. This trait enables runtime-agnostic code.

/// Common interface for HudHudScript execution engines.
///
/// Interpreter and VM both implement this trait, enabling:
/// - Runtime-agnostic testing
/// - Pluggable execution backends
/// - Shared tooling (debugger, profiler)
pub trait Runtime {
    /// The value type produced by this runtime.
    type Value;

    /// Execute a program from source code.
    fn execute_source(&mut self, source: &str) -> Result<(), crate::Error>;

    /// Get a variable's value by name.
    fn get_variable(&self, name: &str) -> Option<Self::Value>;

    /// Check if the runtime is in a healthy state.
    fn is_healthy(&self) -> bool {
        true
    }
}
