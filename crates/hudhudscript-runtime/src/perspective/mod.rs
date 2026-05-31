//! Subject-Oriented Perspectives (SOP) for agents.
//!
//! # Issue #106 — PL Theory: Evolve Agents into Subject-Oriented Perspectives
//!
//! Traditional object-oriented and agent-based systems maintain a single,
//! authoritative view of shared state.  *Subject-Oriented Programming* (SOP),
//! pioneered by William Harrison & Harold Ossher (IBM, 1993), challenges this
//! monolith: different subjects (agents, roles, observers) may hold *different,
//! partial, possibly conflicting perspectives* on the same underlying reality,
//! and those perspectives are composed at runtime.
//!
//! ## Core Concepts
//!
//! ### Perspective
//!
//! A **Perspective** is a named, typed *view* of a shared state space.
//! It defines:
//! - Which fields of the underlying state are *visible* to the holder.
//! - Which fields are *writable* by the holder.
//! - How field values are *projected* (transformed) before being exposed to
//!   the holder.
//!
//! ### PerspectiveHolder
//!
//! A **PerspectiveHolder** binds an agent to one or more perspectives on a
//! shared `AgentState`.  Reads go through the perspective's projection
//! function; writes are filtered by the write-access list.
//!
//! ### Composition
//!
//! When two agents hold different perspectives on the same state, conflicts are
//! resolved by a **MergeStrategy** registered on the shared state.  This is
//! analogous to SOP's *subject composition* rules.
//!
//! ## Example
//!
//! ```text
//! // Shared ledger state: { balance: 1000, audit_trail: [...], secret_key: "..." }
//!
//! // Accountant perspective: sees balance + audit_trail, can write both.
//! // Trader perspective:     sees balance only, can write balance.
//! // Auditor perspective:    sees audit_trail only, read-only.
//!
//! let accountant = PerspectiveHolder::new("accountant-agent", shared_state)
//!     .with_perspective(ACCOUNTANT_PERSPECTIVE);
//! ```
//!
//! ## Relationship to PL Theory
//!
//! SOP extends the object model with a *subject dimension*:
//!
//! ```text
//! OOP:  object.method()
//! SOP:  object.method() [as seen from subject S with perspective P]
//! ```
//!
//! This maps naturally onto agent orchestration: each agent is a subject with
//! its own perspective on global state, and the orchestration engine composes
//! those perspectives when resolving inter-agent communication.

pub mod definition;
pub mod error;
pub mod holder;

pub use definition::*;
pub use error::*;
pub use holder::*;
