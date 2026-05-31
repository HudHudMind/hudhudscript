use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum TypeErrorCode {
    /// E0299 — await is not allowed inside atomically blocks
    TypeAwaitInAtomically = 299,
    /// E0300 — Variable declared more than once in the same scope
    TypeDuplicateVariable = 300,
    /// E0301 — await applied to a non-promise value
    TypeInvalidAwait = 301,
    /// E0302 — Indexing applied to a non-indexable type
    TypeInvalidIndex = 302,
    /// E0303 — Member access on a type without that field
    TypeInvalidMember = 303,
    /// E0304 — Operator not defined for operand type
    TypeInvalidOperator = 304,
    /// E0305 — Type mismatch between expected and actual
    TypeMismatch = 305,
    /// E0306 — Declared type does not match initializer
    TypeTypeMismatchInDecl = 306,
    /// E0307 — Call to an undefined function
    TypeUndefinedFunction = 307,
    /// E0308 — Reference to an undefined variable
    TypeUndefinedVariable = 308,
    /// E0309 — Wrong number of arguments in function call
    TypeWrongArgumentCount = 309,
    /// E0324 — Match on union type does not cover all variants
    TypeNonExhaustiveMatch = 324,
}
