//! Shared exec/process builtin — used by both VM and interpreter (Kural 7).
//!
//! Provides: exec.run, exec.output, exec.stream, exec.lines, exec.spawn,
//!           exec.timeout, exec.kill

pub mod dispatch;
pub mod kill;
pub mod run;
pub mod stream;
pub mod timeout;
pub mod utils;

pub use dispatch::dispatch;
