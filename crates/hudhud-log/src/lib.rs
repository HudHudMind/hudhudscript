//! HudHudScript logging builtins — tracing subscriber wrapper.
//!
//! Wraps the `tracing` ecosystem: init, level-based logging, and span context.
//! All functions return `CompileResult<Value16>` for direct VM integration.

pub mod log_ops;
