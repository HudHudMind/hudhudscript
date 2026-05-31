//! Centralized builtin registry — single source of truth for all passes.
//!
//! LSP, TypeChecker, VM, and Interpreter should all read from this registry
//! instead of maintaining their own hardcoded lists.

/// Information about a builtin module (Math, JSON, TOML, etc.)
pub struct BuiltinModule {
    pub name: &'static str,
    pub description: &'static str,
    pub members: &'static [BuiltinMember],
}

/// A member of a builtin module (function, constant, etc.)
pub struct BuiltinMember {
    pub name: &'static str,
    pub kind: MemberKind,
    pub description: &'static str,
    pub params: &'static [(&'static str, &'static str)], // (name, type)
    pub return_type: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemberKind {
    Function,
    Constant,
}
