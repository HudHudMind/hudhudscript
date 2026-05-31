use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::atomically::atomically;
use crate::error::err_tvar_not_found;
use crate::tvar::TVar;
use hudhudscript_errors::HudHudResult;

/// Maps a string id (held as a value-level handle in scripts) to a live
/// `Arc<TVar<V>>`. Used by the interpreter and the VM to translate
/// script-level TVar handles into real transactional variables.
#[derive(Debug)]
pub struct TVarRegistry<V> {
    vars: Mutex<HashMap<String, Arc<TVar<V>>>>,
}

impl<V: Clone> Default for TVarRegistry<V> {
    fn default() -> Self {
        Self {
            vars: Mutex::new(HashMap::new()),
        }
    }
}

impl<V: Clone> TVarRegistry<V> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new TVar with `initial` and return its id.
    pub fn create(&self, initial: V) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let tvar = TVar::new(initial);
        self.vars.lock().insert(id.clone(), tvar);
        id
    }

    /// Create a new TVar with an explicit id (e.g. a human-readable name).
    /// If the id already exists, the initial value is ignored and the existing
    /// TVar is returned as-is.
    pub fn create_with_id(&self, id: impl Into<String>, initial: V) -> String {
        let id = id.into();
        let mut guard = self.vars.lock();
        guard
            .entry(id.clone())
            .or_insert_with(|| TVar::new(initial));
        id
    }

    /// Get the `Arc<TVar<V>>` for the given id.
    pub fn get(&self, id: &str) -> Option<Arc<TVar<V>>> {
        self.vars.lock().get(id).cloned()
    }

    /// Read the current committed value of a TVar (outside a transaction).
    pub fn read(&self, id: &str) -> HudHudResult<V> {
        let tvar = self.get(id).ok_or_else(|| err_tvar_not_found(id))?;
        Ok(tvar.read_committed().0)
    }

    /// Write directly to committed state — wraps a single-write transaction
    /// so version counters stay accurate.
    pub fn write_direct(&self, id: &str, value: V) -> HudHudResult<()> {
        let tvar = self.get(id).ok_or_else(|| err_tvar_not_found(id))?;
        atomically::<V, _, _>(|tx| {
            tx.write(&tvar, value.clone());
            Ok(())
        })
    }

    /// Number of TVars registered. Useful for testing.
    pub fn len(&self) -> usize {
        self.vars.lock().len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.vars.lock().is_empty()
    }
}
