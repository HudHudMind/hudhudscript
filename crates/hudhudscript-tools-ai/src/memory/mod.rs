//! Long-Term Memory and RAG Tooling (Issue #118)
//!
//! Provides agent-level memory tools that allow an agent to store and retrieve
//! context across conversations.  The implementation uses an in-memory vector
//! store with simple keyword-based retrieval as a baseline; a production
//! deployment can swap in a real embedding + vector DB backend by implementing
//! the `MemoryBackend` trait.

pub mod backend;
pub mod entry;
pub mod error;
pub mod store;

pub use backend::*;
pub use entry::*;
pub use error::*;
pub use store::*;

pub(crate) fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
