//! String interner — maps strings to numeric IDs for O(1) comparison.
//!
//! Audit v3 Finding 2.1 (PERF-16): `parking_lot::RwLock` replaces the
//! `std::sync::Mutex` so concurrent `resolve` calls (the VM hot path
//! — every `LoadVar` / `StoreVar` / `Call` name lookup post-compile)
//! never serialise against each other.  `intern` still needs an
//! exclusive write lock but it's compile-time only.
//!
//! PERF-51 (E1): string storage is `Vec<Arc<str>>` so `resolve_arc(id)`
//! returns a cheap reference-counted clone (atomic increment only, no
//! heap alloc). The free `resolve(id) -> String` wrapper is preserved
//! for backward compatibility but now costs one atomic + one `to_string`.

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

static GLOBAL_INTERNER: RwLock<Option<Interner>> = RwLock::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

pub struct Interner {
    to_id: FxHashMap<Arc<str>, SymbolId>,
    to_str: Vec<Arc<str>>,
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

impl Interner {
    pub fn new() -> Self {
        Self {
            to_id: FxHashMap::default(),
            to_str: Vec::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> SymbolId {
        if let Some(&id) = self.to_id.get(s) {
            return id;
        }
        let id = SymbolId(self.to_str.len() as u32);
        // Single Arc<str> allocation shared between both storages —
        // key lookup stays O(1), clone is a cheap atomic increment.
        let shared: Arc<str> = Arc::from(s);
        self.to_str.push(Arc::clone(&shared));
        self.to_id.insert(shared, id);
        id
    }

    /// Borrow the interned string. Lifetime is bound to `&self`, so the
    /// `&str` is only usable while the (read-)lock on the interner is held.
    pub fn resolve(&self, id: SymbolId) -> &str {
        self.to_str.get(id.0 as usize).map(|s| &**s).unwrap_or("<???>")
    }

    /// Cheap `Arc<str>` clone — atomic increment only, no allocation.
    /// Preferred over `resolve`+`to_string` for hot paths that need to
    /// outlive the interner lock.
    pub fn resolve_arc(&self, id: SymbolId) -> Arc<str> {
        self.to_str.get(id.0 as usize).map(Arc::clone).unwrap_or_else(|| Arc::from("<???>"))
    }
}

/// Global interner access — takes a write lock, so call from
/// compile-time only whenever possible.
pub fn intern(s: &str) -> SymbolId {
    // Fast path: the string might already be interned. Take a read
    // lock first, and only upgrade to a write lock when we need to
    // add a new entry. This keeps compile-time parallelism intact.
    {
        let guard = GLOBAL_INTERNER.read();
        if let Some(interner) = guard.as_ref() {
            if let Some(&id) = interner.to_id.get(s) {
                return id;
            }
        }
    }
    let mut guard = GLOBAL_INTERNER.write();
    let interner = guard.get_or_insert_with(Interner::new);
    interner.intern(s)
}

/// Read-only lookup — returns the interned id if the string is already
/// known, without ever taking a write lock. VM hot path should prefer
/// this over `intern` at runtime.
#[inline(always)]
pub fn try_resolve_id(s: &str) -> Option<u32> {
    let guard = GLOBAL_INTERNER.read();
    guard.as_ref().and_then(|i| i.to_id.get(s).map(|id| id.0))
}

/// Call `f` with the resolved `&str` while holding the read lock.
/// Avoids the String allocation that `resolve` costs. Used by the VM's
/// hot path (`LoadVar` / `StoreVar` / `Call` name lookup).
///
/// This is the **single source of truth** for interner lookup; `resolve`
/// is a thin owning-String wrapper around it.
pub fn resolve_with<R>(id: SymbolId, f: impl FnOnce(&str) -> R) -> R {
    let guard = GLOBAL_INTERNER.read();
    match guard.as_ref() {
        Some(i) => f(i.resolve(id)),
        None => f(""),
    }
}

/// Number of interned symbols. Safe to call from any thread;
/// returns 0 if the interner hasn't been initialised yet.
pub fn len() -> usize {
    let guard = GLOBAL_INTERNER.read();
    guard.as_ref().map(|i| i.to_str.len()).unwrap_or(0)
}

/// Resolve an id to an owned `String`. Takes a read lock; callers that
/// only need a short-lived `&str` should prefer `resolve_with` to avoid
/// the clone. DRY: delegates to `resolve_with`, no duplicate locking.
#[inline(always)]
pub fn resolve(id: SymbolId) -> String {
    resolve_with(id, str::to_owned)
}

/// PERF-51 (E1): cheap `Arc<str>` clone — atomic increment, no alloc.
/// Takes a read lock only for the duration of the `Arc::clone`. Callers
/// that need the interned string to outlive the lock should use this
/// instead of `resolve` (which also allocates a fresh `String`).
///
/// Returns an empty `Arc<str>` if the global interner hasn't been
/// initialised — matches the `resolve_with` behaviour for consistency.
pub fn resolve_arc(id: SymbolId) -> Arc<str> {
    let guard = GLOBAL_INTERNER.read();
    match guard.as_ref() {
        Some(i) => i.resolve_arc(id),
        None => Arc::from(""),
    }
}

/// COMPILE0001: Snapshot the current interner state as `Vec<String>` indexed by SymbolId.
/// Used by `Bytecode::to_bytes()` to serialize a self-contained symbol table.
/// `snapshot()[i]` is the string for SymbolId(i).
pub fn snapshot() -> Vec<String> {
    let guard = GLOBAL_INTERNER.read();
    guard
        .as_ref()
        .map(|i| i.to_str.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

/// COMPILE0002: Restore the interner from a snapshot.
/// After this call, `resolve(SymbolId(i))` returns `snapshot[i]` for all `i < snapshot.len()`.
/// Required for `.hudb` execution: bytecode-local symbol IDs must resolve
/// to the same strings they had at compile time.
pub fn restore(snapshot: Vec<String>) -> Result<(), String> {
    let mut guard = GLOBAL_INTERNER.write();
    if let Some(ref interner) = *guard {
        if interner.to_str.len() > 0 {
            return Err(format!(
                "interner::restore: already populated ({} symbols)",
                interner.to_str.len()
            ));
        }
    }
    let mut interner = Interner::new();
    for s in &snapshot {
        interner.intern(s);
    }
    *guard = Some(interner);
    Ok(())
}

impl Interner {
    pub fn len(&self) -> usize {
        self.to_str.len()
    }
}
