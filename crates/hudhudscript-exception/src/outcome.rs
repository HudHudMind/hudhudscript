use crate::exception::Exception;

/// An outcome that may be exact or degraded (a partial / best-effort result
/// produced under stress, with an attached exception explaining what was lost).
///
/// Used by APIs that prefer to return *some* result over none — for example,
/// a query that returns stale cached data alongside the staleness exception.
#[derive(Debug, Clone)]
pub enum Outcome<T> {
    /// Exact, full-fidelity result.
    Exact(T),
    /// Degraded result with an attached exception describing the degradation.
    Degraded(T, Exception),
}

impl<T> Outcome<T> {
    pub fn value(self) -> T {
        match self {
            Outcome::Exact(v) | Outcome::Degraded(v, _) => v,
        }
    }

    pub fn is_exact(&self) -> bool {
        matches!(self, Outcome::Exact(_))
    }

    pub fn is_degraded(&self) -> bool {
        matches!(self, Outcome::Degraded(_, _))
    }

    pub fn exception(&self) -> Option<&Exception> {
        match self {
            Outcome::Exact(_) => None,
            Outcome::Degraded(_, e) => Some(e),
        }
    }
}
