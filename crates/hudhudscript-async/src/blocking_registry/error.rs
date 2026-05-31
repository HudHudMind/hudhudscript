use super::PromiseId;

/// Errors the registry can return to the caller of `await_blocking`.
#[derive(Debug)]
pub enum RegistryError {
    /// No receiver or cached result is associated with this id.
    ///
    /// For real async work this should never happen — callers are
    /// expected to register before surfacing the id. It typically
    /// indicates a test harness building a `Value::Promise(AsyncPending)`
    /// by hand with no accompanying spawn.
    Unregistered(PromiseId),
    /// The sender was dropped before the task sent a result.
    ///
    /// Usually caused by a panic on the spawned thread.
    SenderDropped(PromiseId),
    /// The promise was rejected. The string is the reject reason as
    /// provided by the task.
    Rejected(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::Unregistered(id) => {
                write!(f, "Promise {} has no registered resolver", id)
            }
            RegistryError::SenderDropped(id) => {
                write!(f, "Promise {} sender was dropped before resolution", id)
            }
            RegistryError::Rejected(msg) => write!(f, "Promise rejected: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}
