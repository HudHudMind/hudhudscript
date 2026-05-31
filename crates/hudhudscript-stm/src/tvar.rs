use parking_lot::Mutex;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static TVAR_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

static COMMIT_LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();

pub(crate) fn commit_lock() -> &'static Mutex<()> {
    COMMIT_LOCK.get_or_init(|| Mutex::new(()))
}

/// A transactional variable, generic over the value type `V`.
///
/// `TVar` wraps a value together with a version counter. The version is
/// incremented on every successful commit that writes this variable.
#[derive(Debug)]
pub struct TVar<V> {
    id: u64,
    inner: Mutex<TVarInner<V>>,
}

impl<V> PartialEq for TVar<V> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<V> Eq for TVar<V> {}

impl<V> Hash for TVar<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone)]
struct TVarInner<V> {
    value: V,
    version: u64,
}

impl<V: Clone> TVar<V> {
    /// Create a new `TVar<V>` with an initial value.
    pub fn new(initial: V) -> Arc<Self> {
        let id = TVAR_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Arc::new(TVar {
            id,
            inner: Mutex::new(TVarInner {
                value: initial,
                version: 0,
            }),
        })
    }

    /// Globally-unique numeric id (for debugging / Hash).
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Read the current committed value and version (outside a transaction).
    pub fn read_committed(&self) -> (V, u64) {
        let guard = self.inner.lock();
        (guard.value.clone(), guard.version)
    }

    /// Write a new committed value under the commit lock.
    pub fn commit_write(&self, value: V) {
        let mut guard = self.inner.lock();
        guard.version += 1;
        guard.value = value;
    }
}
