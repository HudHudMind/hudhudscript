use crate::Instruction;
use serde::{Deserialize, Serialize};

/// Compact symbol identifier — wraps a `u32` index into the global string
/// interner.  Used by `Instruction` variants in place of heap-allocated
/// `String` operands (Issue #1038, P7.1).
///
/// Implements `PartialEq<&str>` so existing pattern-match guards such as
/// `name == "Color"` continue to work without modification.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SymId(pub u32);

impl std::fmt::Debug for SymId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SymId({}={})",
            self.0,
            crate::interner::resolve(crate::interner::SymbolId(self.0))
        )
    }
}

impl std::fmt::Display for SymId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&crate::interner::resolve(crate::interner::SymbolId(self.0)))
    }
}

impl PartialEq<&str> for SymId {
    fn eq(&self, other: &&str) -> bool {
        crate::interner::resolve(crate::interner::SymbolId(self.0)) == *other
    }
}

impl PartialEq<str> for SymId {
    fn eq(&self, other: &str) -> bool {
        crate::interner::resolve(crate::interner::SymbolId(self.0)) == other
    }
}

impl PartialEq<String> for SymId {
    fn eq(&self, other: &String) -> bool {
        crate::interner::resolve(crate::interner::SymbolId(self.0)) == *other
    }
}

impl From<&str> for SymId {
    fn from(s: &str) -> Self {
        SymId(crate::interner::intern(s).0)
    }
}

impl From<String> for SymId {
    fn from(s: String) -> Self {
        SymId(crate::interner::intern(&s).0)
    }
}
