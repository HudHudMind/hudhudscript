//! REPL (Read-Eval-Print Loop) support structures for interactive .hud execution
//!
//! Provides configuration, command registration, line evaluation, and history
//! management for an interactive HudHudScript session.
//!
//! # Issue #612

pub mod buffer;
pub mod command;
pub mod completer;
pub mod config;
pub mod engine;

pub use buffer::LineBuffer;
pub use command::ReplAction;
pub use completer::HudCompleter;
pub use config::ReplConfig;
pub use engine::Repl;
