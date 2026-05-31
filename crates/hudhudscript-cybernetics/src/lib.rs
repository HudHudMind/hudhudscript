//! Cybernetic Orchestration Meta-Language (COML) — formal semantics.
//!
//! # Issue #108 — PL Theory: Formalize the Cybernetic Orchestration Meta-Language Paradigm
//!
//! ## Background: Cybernetics and Control Theory
//!
//! Norbert Wiener's *cybernetics* (1948) studies goal-directed systems that
//! use feedback to regulate their own behaviour.  The canonical cybernetic
//! structure is the **control loop**:
//!
//! ```text
//! ┌────────────────────────────────────────────────────┐
//! │                  CONTROL LOOP                      │
//! │                                                    │
//! │  ┌──────────┐   u(t)   ┌──────────┐  y(t)        │
//! │  │Controller├──────────►  Plant   ├──────────►    │
//! │  └────▲─────┘          └──────────┘            output
//! │       │ e(t) = r(t) - y(t)                        │
//! │       │                                            │
//! │  r(t) ◄──────────────────────────────────────────┘
//! │  (reference / goal)                                │
//! └────────────────────────────────────────────────────┘
//! ```
//!
//! - **r(t)** — reference signal (the goal / desired output).
//! - **y(t)** — actual output (what the system is currently doing).
//! - **e(t)** — error signal (`r − y`).
//! - **u(t)** — control signal (action taken by the controller to reduce `e`).
//!
//! ## COML: Applying Cybernetics to Agent Orchestration
//!
//! HudHudScript's orchestration engine is a *cybernetic meta-language*: its
//! primitive constructs map directly onto cybernetic concepts.
//!
//! | Cybernetic concept | COML construct              |
//! |--------------------|-----------------------------|
//! | Plant              | `Agent` / `Network`         |
//! | Controller         | `OrchestrationEngine`       |
//! | Reference signal   | `Goal`                      |
//! | Sensor / Observer  | `Observer<S>`               |
//! | Error signal       | `ControlError<S>`           |
//! | Actuator           | `Actuator<A>`               |
//! | Control loop       | `ControlLoop<S,A>`          |
//! | Feedback policy    | `FeedbackPolicy`            |
//!
//! ## Formal Denotational Semantics
//!
//! Let `S` be the type of system state and `A` the type of actions.
//!
//! ```text
//! ⟦ControlLoop⟧ : S → Stream(A)
//!
//! ⟦observe⟧    : S      → Observable(S)
//! ⟦goal⟧       : ()     → Goal(S)
//! ⟦error⟧      : Goal(S) × Observable(S) → ControlError(S)
//! ⟦policy⟧     : ControlError(S) → A
//! ⟦actuate⟧    : A      → Effect(S)
//!
//! ⟦loop⟧(s₀) = fix(λ s. let obs  = ⟦observe⟧(s)
//!                         let err  = ⟦error⟧(goal, obs)
//!                         let act  = ⟦policy⟧(err)
//!                         let s'   = ⟦actuate⟧(act)(s)
//!                         in s')
//! ```
//!
//! The fixed-point `fix` captures the iterative nature of the feedback loop.
//! In the discrete (digital) setting this becomes a `tick()` function called
//! repeatedly by the orchestration engine scheduler.
//!
//! ## Type-Level Representation
//!
//! The types in this module give the runtime a first-class, inspectable
//! representation of control loops.  This enables:
//! - **Static analysis**: detect loops that can never converge.
//! - **Monitoring**: expose loop state via the observability subsystem.
//! - **Composition**: nest loops (inner/outer loop patterns).
//! - **Testing**: drive loops with mock observers and actuators.

pub mod actuation;
pub mod control_error;
pub mod errors;
pub mod goal;
pub mod r#loop;
pub mod observable;
pub mod policies;
pub mod traits;

// Backward compatibility re-exports
pub use actuation::ActuationResult;
pub use control_error::ControlError;
pub use errors::CyberneticsError;
pub use goal::Goal;
pub use observable::Observable;
pub use policies::BangBangPolicy;
pub use r#loop::{ControlLoop, LoopStats};
pub use traits::{Actuator, FeedbackPolicy, Observer};
