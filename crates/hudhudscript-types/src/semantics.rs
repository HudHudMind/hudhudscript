//! Value semantics vs. reference semantics in HudHudScript.
//!
//! # Issue #104 — Bjarne Stroustrup Review: Multi-Paradigm Integration
//!
//! Stroustrup designed C++ as a deliberately *multi-paradigm* language:
//! procedural, object-oriented, generic, and functional styles coexist and
//! complement each other.  A key design axis is the distinction between
//! **value semantics** and **reference semantics**.
//!
//! HudHudScript must make this distinction explicit so that:
//!
//! 1. Agents that share state do not accidentally mutate each other's copies.
//! 2. Pipeline stages that transform data leave the source unchanged.
//! 3. The type checker can catch incorrect aliasing patterns at compile time.
//!
//! ## Value Semantics
//!
//! A type has *value semantics* when copying it produces an independent copy
//! with its own identity.  Mutations to the copy do not affect the original.
//!
//! - **All primitive types** (`Number`, `Boolean`, `String`) are value types.
//! - **Structs / records** created with `let x = { ... }` are value types by
//!   default (deep-copied on assignment).
//! - Agent *task inputs* and *task outputs* always have value semantics: each
//!   task receives its own copy of the arguments.
//!
//! ```hhs
//! let a = { count: 1 };
//! let b = a;          // b is an independent copy
//! b.count = 2;        // a.count is still 1
//! ```
//!
//! ## Reference Semantics
//!
//! A type has *reference semantics* when assignment copies the *reference*
//! (handle) rather than the data.  Both the original and the copy now observe
//! the same underlying state.
//!
//! - **Agent shared state** (`AgentState`) is managed as a shared reference
//!   inside the runtime (`Arc<RwLock<AgentState>>`).
//! - **Provider connections** are reference-counted handles (`Arc<dyn Provider>`).
//! - Large data blobs passed between orchestration layers use reference
//!   semantics internally to avoid copying megabytes of JSON.
//!
//! ```hhs
//! // (conceptual HudHudScript syntax)
//! let state = agent.shared_state();  // reference handle
//! state.count += 1;                  // observed by all holders of the handle
//! ```
//!
//! ## Implementation status (v0.4.9, Issues #330, #331)
//!
//! - DONE(#330): The parser/AST now distinguishes `let` (Owned), `ref` (Borrowed),
//!   and `ref mut` (MutBorrowed) via `VarDecl::ownership` / `OwnershipMode`.
//! - DONE(#330): The type checker's `SymbolTable` now stores `SymbolInfo` with
//!   `mutable: bool` and `ownership: Ownership` alongside each symbol. The
//!   checker rejects assignment to immutable (`const`) bindings at type-check time.
//! - DONE(#331): `Value::Array` and `Value::Object` in the interpreter now use
//!   copy-on-write (`CowArray` / `CowObject`) backed by `Arc`, so assignment
//!   shares the data and mutation clones only when the reference count > 1.
//! - TODO(#104): Agent task boundaries should enforce value-semantic deep-copy
//!   at task call sites. Currently `StateValue` is `Clone` but is not deep-copied.
//!   A `TaskCallConvention::ValueCopy` marker should be added.

use crate::Type;
use std::fmt;

// ---------------------------------------------------------------------------
// Ownership model
// ---------------------------------------------------------------------------

/// The ownership and aliasing mode of a value or variable binding.
///
/// This is the type-level representation of the value/reference distinction.
/// It is analogous to C++'s combination of value types, references (`T&`),
/// and shared smart pointers (`std::shared_ptr<T>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// The binding owns its data exclusively.  Assignment copies the data.
    ///
    /// Analogous to a C++ value type or `std::unique_ptr`.
    /// All primitive types and most record types have this ownership.
    OwnedValue,

    /// The binding holds a shared, immutable reference.  Multiple bindings may
    /// alias the same data; none may mutate it.
    ///
    /// Analogous to `const T&` or `std::shared_ptr<const T>` in C++.
    SharedRef,

    /// The binding holds a shared, mutable reference.  Multiple bindings may
    /// alias the same data; any may mutate it (with appropriate locking).
    ///
    /// Analogous to `T&` (with external synchronisation) or
    /// `std::shared_ptr<T>` in C++.
    ///
    /// # Safety
    /// The type checker should warn when a `SharedMutRef` escapes its
    /// originating scope without an explicit lock annotation.
    SharedMutRef,
}

impl fmt::Display for Ownership {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ownership::OwnedValue => write!(f, "value"),
            Ownership::SharedRef => write!(f, "&T"),
            Ownership::SharedMutRef => write!(f, "&mut T"),
        }
    }
}

/// A type annotated with its ownership mode.
///
/// The type checker will track this alongside each symbol in the
/// [`SymbolTable`](crate::SymbolTable).
///
/// # TODO(#104)
/// Currently the symbol table stores only `Type`.  This struct is the target
/// shape; the checker should be migrated to use `OwnedType` instead of `Type`.
#[derive(Debug, Clone, PartialEq)]
pub struct OwnedType {
    /// The underlying type.
    pub ty: Type,
    /// The ownership mode.
    pub ownership: Ownership,
}

impl OwnedType {
    /// Construct a value-semantic binding.
    pub fn value(ty: Type) -> Self {
        Self {
            ty,
            ownership: Ownership::OwnedValue,
        }
    }

    /// Construct a shared-immutable-reference binding.
    pub fn shared_ref(ty: Type) -> Self {
        Self {
            ty,
            ownership: Ownership::SharedRef,
        }
    }

    /// Construct a shared-mutable-reference binding.
    pub fn shared_mut_ref(ty: Type) -> Self {
        Self {
            ty,
            ownership: Ownership::SharedMutRef,
        }
    }

    /// Returns `true` if this binding is mutable.
    pub fn is_mutable(&self) -> bool {
        matches!(self.ownership, Ownership::SharedMutRef)
    }

    /// Returns `true` if this binding follows value (copy) semantics.
    pub fn is_value_semantic(&self) -> bool {
        matches!(self.ownership, Ownership::OwnedValue)
    }

    /// Returns `true` if this binding follows reference semantics.
    pub fn is_reference_semantic(&self) -> bool {
        matches!(
            self.ownership,
            Ownership::SharedRef | Ownership::SharedMutRef
        )
    }
}

impl fmt::Display for OwnedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ownership {
            Ownership::OwnedValue => write!(f, "{}", self.ty),
            Ownership::SharedRef => write!(f, "&{}", self.ty),
            Ownership::SharedMutRef => write!(f, "&mut {}", self.ty),
        }
    }
}

// ---------------------------------------------------------------------------
// Default ownership rules
// ---------------------------------------------------------------------------

/// Return the *default* ownership mode for a given type.
///
/// These rules mirror Stroustrup's recommendations for C++:
/// - Primitive types are cheap to copy → value semantics.
/// - Collection types are potentially large → shared-ref by default when
///   passed across agent boundaries (TODO(#104): enforce at call sites).
/// - Function types are reference-semantic (closures capture their environment).
pub fn default_ownership(ty: &Type) -> Ownership {
    match ty {
        // Primitives: cheap copy → value semantics.
        Type::String | Type::Number | Type::Boolean | Type::Null => Ownership::OwnedValue,

        // Collections: potentially large → shared reference by default.
        // DONE(#331): collections now use copy-on-write (Arc-backed) internally,
        // so shared-ref is cheap and mutation auto-clones when refcount > 1.
        Type::Array(_) | Type::Object(_) => Ownership::SharedRef,

        // Functions and closures are reference-semantic.
        Type::Function { .. } => Ownership::SharedRef,

        // Async results: reference-semantic (the future is a handle).
        Type::Promise(_) => Ownership::SharedRef,

        // Tool/Resource/Server handles: always shared references.
        Type::Tool { .. } | Type::Resource { .. } | Type::Server => Ownership::SharedRef,

        // Union / Generic: default to value; the concrete type determines the real mode.
        Type::Union(_) | Type::Generic(_) | Type::Any => Ownership::OwnedValue,

        // Class definitions and instances: reference-semantic (like objects).
        Type::Class { .. } | Type::Instance(_) => Ownership::SharedRef,

        // Parameterized types: follow base type semantics (collections are shared-ref).
        Type::Parameterized { .. } => Ownership::SharedRef,
    }
}
