//! Compile-time function contracts — preconditions, postconditions, and
//! class invariants à la Bjarne Stroustrup's C++20 *Contracts* proposal.
//!
//! # Issue #103 — Bjarne Stroustrup Review: Concepts and Constexpr for Compile-Time Contracts
//!
//! C++20 introduces *Concepts* (compile-time type constraints) and formalises
//! *Contracts* (preconditions / postconditions / invariants checked at the call
//! site).  HudHudScript adopts the same philosophy at the type-checker level:
//!
//! - **Preconditions** — constraints that must hold on function inputs before
//!   the function body executes (checked at every call site).
//! - **Postconditions** — constraints that the function guarantees on its
//!   return value (checked at return sites inside the function body and
//!   verifiable by callers via the function's [`ContractSignature`]).
//! - **Invariants** — structural constraints that must hold throughout an
//!   agent's lifetime (checked on agent construction and periodically by the
//!   runtime).
//!
//! ## Relationship to C++ Concepts
//!
//! A C++ *Concept* is essentially a named set of compile-time constraints on a
//! template parameter.  Here we map that idea onto HudHudScript's function
//! signatures:
//!
//! ```text
//! C++ Concept                   HudHudScript equivalent
//! ──────────────────────────────────────────────────────
//! requires T: Numeric           param_type == Type::Number
//! requires T: Serializable      param_type: HasToString (future)
//! requires { t.size() } -> int  precondition: type constraint on member
//! ```
//!
//! ## Usage in the Type Checker
//!
//! ```text
//! function transfer(amount: Number, account: String)
//!     requires amount > 0             // precondition
//!     ensures result.success == true  // postcondition (conceptual)
//! ```
//!
//! The type checker resolves `ContractSignature`s at the call site and reports
//! `ContractViolation` errors before any code runs.

use crate::Type;
use std::fmt;

// ---------------------------------------------------------------------------
// Type Constraints (C++ Concept equivalents)
// ---------------------------------------------------------------------------

/// A compile-time constraint on a single value's type.
///
/// This is the HudHudScript analogue of a C++ *Concept*: a named, checkable
/// predicate over a type that can be attached to function parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    /// The value must have exactly this type (no widening).
    Exact(Type),
    /// The value must be a numeric type (`Number`).
    Numeric,
    /// The value must be a string type (`String`).
    Textual,
    /// The value must be iterable (`Array<T>` for any `T`).
    Iterable,
    /// The value must be callable (`Function`).
    Callable,
    /// The value must be non-null (excludes `Null`).
    NonNull,
    /// The value must satisfy all listed constraints.
    All(Vec<TypeConstraint>),
    /// The value must satisfy at least one listed constraint.
    Any(Vec<TypeConstraint>),
}

impl TypeConstraint {
    /// Check whether `ty` satisfies this constraint.
    ///
    /// Returns `Ok(())` on success or an explanation string on failure.
    pub fn check(&self, ty: &Type) -> Result<(), String> {
        match self {
            TypeConstraint::Exact(expected) => {
                if ty.is_compatible_with(expected) {
                    Ok(())
                } else {
                    Err(format!("expected {}, found {}", expected, ty))
                }
            }
            TypeConstraint::Numeric => {
                if matches!(ty, Type::Number | Type::Any) {
                    Ok(())
                } else {
                    Err(format!("expected Numeric type, found {}", ty))
                }
            }
            TypeConstraint::Textual => {
                if matches!(ty, Type::String | Type::Any) {
                    Ok(())
                } else {
                    Err(format!("expected Textual type, found {}", ty))
                }
            }
            TypeConstraint::Iterable => {
                if matches!(ty, Type::Array(_) | Type::Any) {
                    Ok(())
                } else {
                    Err(format!("expected Iterable type (Array<T>), found {}", ty))
                }
            }
            TypeConstraint::Callable => {
                if matches!(ty, Type::Function { .. } | Type::Any) {
                    Ok(())
                } else {
                    Err(format!("expected Callable type, found {}", ty))
                }
            }
            TypeConstraint::NonNull => {
                if matches!(ty, Type::Null) {
                    Err("value must be non-null".to_string())
                } else {
                    Ok(())
                }
            }
            TypeConstraint::All(constraints) => {
                for c in constraints {
                    c.check(ty)?;
                }
                Ok(())
            }
            TypeConstraint::Any(constraints) => {
                if constraints.iter().any(|c| c.check(ty).is_ok()) {
                    Ok(())
                } else {
                    Err(format!(
                        "type {} does not satisfy any of the required constraints",
                        ty
                    ))
                }
            }
        }
    }
}

impl fmt::Display for TypeConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeConstraint::Exact(t) => write!(f, "Exact({})", t),
            TypeConstraint::Numeric => write!(f, "Numeric"),
            TypeConstraint::Textual => write!(f, "Textual"),
            TypeConstraint::Iterable => write!(f, "Iterable"),
            TypeConstraint::Callable => write!(f, "Callable"),
            TypeConstraint::NonNull => write!(f, "NonNull"),
            TypeConstraint::All(cs) => {
                let s: Vec<_> = cs.iter().map(|c| format!("{}", c)).collect();
                write!(f, "All({})", s.join(", "))
            }
            TypeConstraint::Any(cs) => {
                let s: Vec<_> = cs.iter().map(|c| format!("{}", c)).collect();
                write!(f, "Any({})", s.join(", "))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Preconditions
// ---------------------------------------------------------------------------

/// A precondition binds a type constraint to a named function parameter.
///
/// At each call site the type checker resolves the type of the actual argument
/// and verifies it satisfies the constraint before allowing the call to proceed.
///
/// # Example
/// ```
/// # use hudhudscript_types::contracts::{Precondition, TypeConstraint};
/// # use hudhudscript_types::Type;
/// let pre = Precondition::new("amount", TypeConstraint::Numeric);
/// assert!(pre.check(&Type::Number).is_ok());
/// assert!(pre.check(&Type::String).is_err());
/// ```
#[derive(Debug, Clone)]
pub struct Precondition {
    /// The parameter name this precondition applies to.
    pub param: String,
    /// The type constraint that must hold.
    pub constraint: TypeConstraint,
    /// Human-readable description of the precondition (appears in error messages).
    pub description: Option<String>,
}

impl Precondition {
    /// Create a new precondition without a description.
    pub fn new(param: impl Into<String>, constraint: TypeConstraint) -> Self {
        Self {
            param: param.into(),
            constraint,
            description: None,
        }
    }

    /// Attach a description to improve error messages.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Check that `ty` (the type of the actual argument) satisfies this
    /// precondition.
    pub fn check(&self, ty: &Type) -> Result<(), ContractViolation> {
        self.constraint
            .check(ty)
            .map_err(|reason| ContractViolation::PreconditionFailed {
                param: self.param.clone(),
                reason,
                description: self.description.clone(),
            })
    }
}

impl fmt::Display for Precondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = &self.description {
            write!(f, "requires {}:{} ({})", self.param, self.constraint, desc)
        } else {
            write!(f, "requires {}:{}", self.param, self.constraint)
        }
    }
}

// ---------------------------------------------------------------------------
// Postconditions
// ---------------------------------------------------------------------------

/// A postcondition describes a type constraint the function's return value must
/// satisfy.
///
/// The type checker uses postconditions in two complementary ways:
/// 1. **Return-site checking** — when analysing the function body, each
///    `return` expression's type is checked against the postcondition.
/// 2. **Call-site narrowing** — callers can rely on the postcondition to narrow
///    the return type (e.g., `NonNull` tells the caller the result is safe to
///    use without a null check).
#[derive(Debug, Clone)]
pub struct Postcondition {
    /// The type constraint the return value must satisfy.
    pub constraint: TypeConstraint,
    /// Human-readable description.
    pub description: Option<String>,
}

impl Postcondition {
    /// Create a new postcondition.
    pub fn new(constraint: TypeConstraint) -> Self {
        Self {
            constraint,
            description: None,
        }
    }

    /// Attach a description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Check that `ty` (the return value's type) satisfies this postcondition.
    pub fn check(&self, ty: &Type) -> Result<(), ContractViolation> {
        self.constraint
            .check(ty)
            .map_err(|reason| ContractViolation::PostconditionFailed {
                reason,
                description: self.description.clone(),
            })
    }
}

impl fmt::Display for Postcondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = &self.description {
            write!(f, "ensures result:{} ({})", self.constraint, desc)
        } else {
            write!(f, "ensures result:{}", self.constraint)
        }
    }
}

// ---------------------------------------------------------------------------
// Contract Signature
// ---------------------------------------------------------------------------

/// The full compile-time contract for a function.
///
/// A `ContractSignature` is stored alongside the function's [`Type`] in the
/// [`SymbolTable`](crate::SymbolTable) and consulted by the type checker at
/// every call site and every return statement inside the function body.
#[derive(Debug, Clone, Default)]
pub struct ContractSignature {
    /// Preconditions (one per constrained parameter, in declaration order).
    pub preconditions: Vec<Precondition>,
    /// Postconditions on the return value.
    pub postconditions: Vec<Postcondition>,
}

impl ContractSignature {
    /// Build a new, empty contract signature.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a precondition.
    pub fn with_precondition(mut self, pre: Precondition) -> Self {
        self.preconditions.push(pre);
        self
    }

    /// Add a postcondition.
    pub fn with_postcondition(mut self, post: Postcondition) -> Self {
        self.postconditions.push(post);
        self
    }

    /// Verify that `arg_types` (keyed by parameter name) satisfy all
    /// preconditions.  Returns a list of all violations.
    ///
    /// # Call-site usage
    /// The type checker calls this with the resolved types of every argument at
    /// each call site.  Any returned violations are reported as compile errors.
    pub fn check_preconditions(
        &self,
        arg_types: &std::collections::HashMap<String, Type>,
    ) -> Vec<ContractViolation> {
        self.preconditions
            .iter()
            .filter_map(|pre| {
                if let Some(ty) = arg_types.get(&pre.param) {
                    pre.check(ty).err()
                } else {
                    // Parameter not provided — a separate type error will fire.
                    None
                }
            })
            .collect()
    }

    /// Verify that `return_type` satisfies all postconditions.
    pub fn check_postconditions(&self, return_type: &Type) -> Vec<ContractViolation> {
        self.postconditions
            .iter()
            .filter_map(|post| post.check(return_type).err())
            .collect()
    }

    /// Returns `true` if there are no contracts to check.
    pub fn is_empty(&self) -> bool {
        self.preconditions.is_empty() && self.postconditions.is_empty()
    }
}

impl fmt::Display for ContractSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for pre in &self.preconditions {
            writeln!(f, "  {}", pre)?;
        }
        for post in &self.postconditions {
            writeln!(f, "  {}", post)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Contract Violations
// ---------------------------------------------------------------------------

/// An error produced when a compile-time contract is violated.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ContractViolation {
    /// A precondition on a function parameter was not satisfied at the call site.
    #[error(
        "precondition violated for parameter '{param}': {reason}{}",
        description.as_deref().map(|d| format!(" ({})", d)).unwrap_or_default()
    )]
    PreconditionFailed {
        param: String,
        reason: String,
        description: Option<String>,
    },

    /// A postcondition on the function's return value was not satisfied.
    #[error(
        "postcondition violated: {reason}{}",
        description.as_deref().map(|d| format!(" ({})", d)).unwrap_or_default()
    )]
    PostconditionFailed {
        reason: String,
        description: Option<String>,
    },
}
