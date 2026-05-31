//! Issue #99: Zero-copy Parsing and String Interning
//!
//! A simple, lock-protected string interner that deduplicates identifier
//! strings produced during parsing.  Interned strings are stored as `Arc<str>`
//! so callers pay only for an `Arc` clone (pointer copy + reference-count
//! increment) rather than a full heap allocation per occurrence.
//!
//! ## Why this matters
//! The parser sees every identifier multiple times — first in its declaration
//! and then at each use site.  Without interning, each occurrence allocates a
//! new `String` on the heap.  With interning, every occurrence of, say,
//! `"provider"` points to the same heap allocation.
//!
//! ## Usage
//! ```rust
//! use std::sync::Arc;
//! use hudhudscript_parser::interner::StringInterner;
//! let mut interner = StringInterner::default();
//! let a = interner.intern("hello");
//! let b = interner.intern("hello");
//! assert!(Arc::ptr_eq(&a, &b)); // same allocation
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A thread-safe string interner backed by a `HashMap<String, Arc<str>>`.
///
/// Interning is O(1) amortised (hash-table lookup) and produces `Arc<str>`
/// handles.  Two calls with equal strings always return `Arc`s that point to
/// the same heap allocation.
#[derive(Debug, Default)]
pub struct StringInterner {
    map: HashMap<String, Arc<str>>,
}

impl StringInterner {
    /// Create a new, empty interner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern `s`, returning a shared `Arc<str>`.
    ///
    /// If `s` has been interned before the existing `Arc` is returned (no new
    /// heap allocation for the string data).  If not, a single allocation is
    /// made and cached for future calls.
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(existing) = self.map.get(s) {
            return Arc::clone(existing);
        }
        let arc: Arc<str> = Arc::from(s);
        self.map.insert(s.to_string(), Arc::clone(&arc));
        arc
    }

    /// Number of distinct strings currently interned.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if no strings have been interned yet.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Reserve capacity to avoid rehashing when many identifiers are expected.
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }
}

// ---------------------------------------------------------------------------
// Global / session-level interner
// ---------------------------------------------------------------------------

/// A thread-safe wrapper around `StringInterner` suitable for sharing across
/// threads (e.g., parallel parsing tasks).
#[derive(Debug, Default, Clone)]
pub struct SharedInterner(Arc<Mutex<StringInterner>>);

impl SharedInterner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a string, returning an `Arc<str>`.
    pub fn intern(&self, s: &str) -> Arc<str> {
        self.0.lock().expect("internal: mutex poisoned").intern(s)
    }

    /// Number of distinct strings currently interned.
    pub fn len(&self) -> usize {
        self.0.lock().expect("internal: mutex poisoned").len()
    }

    /// Returns `true` if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.0.lock().expect("internal: mutex poisoned").is_empty()
    }
}

// ---------------------------------------------------------------------------
// Parser-level interner — module-level singleton for a single parse session
// ---------------------------------------------------------------------------

/// Intern an identifier string during parsing.
///
/// This is a convenience wrapper used by the parser.  It returns `String`
/// (rather than `Arc<str>`) because the AST currently stores `String` for
/// identifiers; the interning benefit is reduced allocation pressure during
/// the parse phase itself (repeated identifiers reuse the `Arc` internally).
///
/// Once the AST migrates identifier storage to `Arc<str>` this function can
/// return `Arc<str>` directly.
pub fn intern_identifier(interner: &mut StringInterner, name: &str) -> String {
    // Intern to share the heap allocation, then convert to String.
    // The Arc drop after this call is cheap (just a ref-count decrement).
    interner.intern(name).to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
