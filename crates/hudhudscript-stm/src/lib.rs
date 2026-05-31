//! Shared Software Transactional Memory (STM) — Kural 7 single-source impl.
//!
//! Provides lightweight STM primitives used by BOTH the interpreter and the VM:
//!
//! - `TVar<V>` — a transactional variable holding a value of type `V`.
//! - `Transaction<V>` — a read/write log collected during an `atomically` block.
//! - `atomically(f)` — retry-loop that commits only when there are no conflicts.
//! - `TVarRegistry<V>` — string-id → `Arc<TVar<V>>` so scripts can reference TVars.
//!
//! ## Design
//! This is a classic optimistic-concurrency STM:
//! 1. Read phase: reads are served from a per-transaction log (or the global
//!    version if not yet seen). Each `TVar` carries a monotonic version counter.
//! 2. Commit phase: the transaction validates that every read `TVar`'s version
//!    has not advanced since it was read. If clean, writes are applied
//!    atomically under a lock; otherwise the transaction retries.
//!
//! The implementation is lock-free for reads within a transaction and uses a
//! single global commit mutex only at commit time.
//!
//! ## STM Sync-Only Constraint
//!
//! **`await` is forbidden inside `atomically()` blocks.** Enforcement of this
//! constraint (both compile-time via the type checker and runtime via an
//! `in_stm_context` flag) is the responsibility of the caller (interpreter/VM);
//! this crate only supplies the transactional machinery.
//!
//! ## Generic design (Kural 7)
//! Because the interpreter and VM use different `Value` enums
//! (`hudhudscript_interpreter_core::Value` vs `hudhudscript_bytecode::Value`),
//! the algorithm is generic over a `V: Clone + Send + Sync + 'static` type.
//! Both runtimes instantiate `TVar<V>` with their own `Value` so the retry
//! loop, version counter, and conflict detection are shared — not duplicated.

pub mod atomically;
pub mod error;
pub mod registry;
pub mod transaction;
pub mod tvar;

pub use atomically::{atomically, atomically_with_config, StmConfig};
pub use error::{err_max_retries_exceeded, err_timeout, err_tvar_not_found};
pub use registry::TVarRegistry;
pub use transaction::Transaction;
pub use tvar::TVar;
