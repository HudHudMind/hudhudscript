//! Human-in-the-Loop (HitL) Approval State Machine (Issue #120, #632)
//!
//! Tools marked as requiring human approval transition through:
//!   `Pending` → `Approved` | `Denied` → `Executed` | `Skipped`
//!
//! The state machine is intentionally synchronous so it can be driven by any
//! async executor or an interactive CLI callback.

pub mod confirmation;
pub mod gate;
pub mod prompt;
pub mod registry;
pub mod types;

pub use confirmation::*;
pub use gate::*;
pub use prompt::*;
pub use registry::*;
pub use types::*;
