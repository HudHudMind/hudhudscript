//! HudHudScript Debugger
//!
//! This crate provides debugging and profiling support for HudHudScript.

pub mod breakpoint;
pub mod dap;
pub mod debugger;
pub mod profiler;

pub use breakpoint::{Breakpoint, BreakpointId, BreakpointKind};
pub use dap::{
    DapEvent, DapMessage, DapRequest, DapResponse, DapServer, Source, Variable, THREAD_ID,
    THREAD_NAME,
};
pub use debugger::{
    CallFrame, DebugState, Debugger, PauseReason, ScopeVariable, StatementAction, StepMode,
    WatchExpression,
};
pub use profiler::{ProfileReport, ProfileSample, Profiler};
