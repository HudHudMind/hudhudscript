//! Statement AST nodes
//!
//! All agent, tool, and resource declarations use the unified `Decl` enum
//! via `Stmt::Decl(Decl::Agent { .. })`, `Stmt::Decl(Decl::Action { .. })`, etc.
//! Both the pest-based parser and the legacy statement parser emit these canonical forms.

use crate::{Expr, ImportKind, McpServerDecl, Span, VarDecl};
use serde::{Deserialize, Serialize};

pub mod decl;
pub use decl::SubjectAbilityDef;
mod impls;

pub use decl::*;

/// Switch case clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchCase {
    pub value: Expr,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Pattern for match arms (ADT pattern matching)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchPattern {
    /// Wildcard: _
    Wildcard,
    /// Literal: 42, "hello", true
    Literal(crate::Literal),
    /// Simple identifier binding: x
    Identifier(String),
    /// Enum variant: Shape::Circle or Shape::Circle(r)
    EnumVariant {
        enum_name: String,
        variant: String,
        binding: Option<String>,
    },
    /// OR pattern: pattern1 | pattern2 | pattern3 — Issue #748
    Or(Vec<MatchPattern>),
}

/// A single arm in a match statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    /// Optional guard expression: `pattern if expr => { ... }` — Issue #748
    #[serde(default)]
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Enum variant declaration: Circle(radius) or Point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<String>,
    pub span: Span,
}

/// Catch clause for try-catch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchClause {
    pub param: String,
    pub body: Box<Stmt>,
    pub span: Span,
}

/// Decorator annotation: @ai, @payment, @cloud, @hudhud, @custom(params)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decorator {
    pub name: String,
    pub params: Vec<(String, Expr)>,
    pub span: Span,
}

/// Declaration node (for agent, task, tool, resource, import, governance)
/// Both the pest-based parser and the legacy parser emit `Decl` variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    /// Declaration (agent, task, tool, resource, import, governance, etc.).
    Decl(Decl),

    /// MCP server declaration: mcp server myServer { ... }
    McpServer(McpServerDecl),

    /// Variable declaration: let x = 42; or const x = 42;
    ///
    /// This is the richer form that includes `is_const` and `type_annotation`.
    /// Produced by the legacy statement parser. Prefer this over `Let`/`Const` for new code.
    VarDecl(VarDecl),

    /// Let statement: let x = 42;
    ///
    /// Simpler form produced by the pest-based parser. Does not carry a type annotation.
    /// Prefer `VarDecl` when type annotation support is needed.
    Let {
        name: String,
        value: Expr,
        span: Span,
    },

    /// Const statement: const x = 42;
    ///
    /// Simpler form produced by the pest-based parser. Does not carry a type annotation.
    /// Prefer `VarDecl` (with `is_const: true`) when type annotation support is needed.
    Const {
        name: String,
        value: Expr,
        span: Span,
    },

    /// Assignment: x = 42;
    Assignment {
        target: Expr,
        value: Expr,
        span: Span,
    },

    /// If statement: if (cond) { ... } else { ... }
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        span: Span,
    },

    /// While loop: while (cond) { ... }
    While {
        condition: Expr,
        body: Box<Stmt>,
        span: Span,
    },

    /// For loop: for (x in arr) { ... }
    For {
        variable: String,
        iterable: Expr,
        body: Box<Stmt>,
        span: Span,
    },

    /// C-style for loop: for (var i = 0; i < 10; i = i + 1) { ... }
    ForCStyle {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        update: Option<Box<Stmt>>,
        body: Box<Stmt>,
        span: Span,
    },

    /// Range-based for loop: for(0, 100) or for(0, 100, 2) or for(100, 0, -1)
    /// Turkish: döngü(0, 100) / döngü(0, 100, 1) / döngü(100, 0, -1)
    ForRange {
        start: Expr,
        stop: Expr,
        step: Option<Expr>,
        body: Box<Stmt>,
        span: Span,
    },

    /// Block: { ... }
    Block { statements: Vec<Stmt>, span: Span },

    /// Return statement: return expr;
    Return { value: Option<Expr>, span: Span },

    /// Break statement: break;
    Break { span: Span },

    /// Continue statement: continue;
    Continue { span: Span },

    /// Switch statement: switch (expr) { case 1: ... default: ... }
    Switch {
        value: Expr,
        cases: Vec<SwitchCase>,
        default: Option<Vec<Stmt>>,
        span: Span,
    },

    /// Try-catch-finally statement
    Try {
        try_block: Box<Stmt>,
        catch_clause: Option<CatchClause>,
        finally_block: Option<Box<Stmt>>,
        span: Span,
    },

    /// Throw statement: throw expr;
    Throw { value: Expr, span: Span },

    /// Expression statement: foo();
    Expr(Expr),

    /// ES-module style import: import { foo } from "module";
    ///
    /// Carries an `ImportKind` (Named/Default/Wildcard). Produced by the pest-based parser.
    /// Distinct from `Decl(Decl::Import { .. })`, which represents HudHudScript `use` imports.
    Import {
        path: String,
        imports: ImportKind,
        span: Span,
    },

    /// Export statement: export let x = 42; or re-export: export { foo } from 'module';
    Export {
        item: Box<Stmt>,
        /// When present, this is a re-export from another module (e.g. `export { x } from 'mod'`).
        source: Option<String>,
        span: Span,
    },

    /// Function declaration: function foo(x, y) { ... }
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        is_async: bool,
        /// Generator function: function* foo() { yield value; } — Issue #667
        #[serde(default)]
        is_generator: bool,
        /// Generic type parameters: function map<T>(arr: Array<T>): Array<T> — Issue #658
        #[serde(default)]
        type_params: Vec<crate::GenericParam>,
        span: Span,
    },

    /// Trait/Interface declaration — Issue #659
    /// trait Serializable { function serialize(): String; function deserialize(data: String); }
    Trait {
        name: String,
        /// Generic type parameters: trait Comparable<T> { ... }
        #[serde(default)]
        type_params: Vec<crate::GenericParam>,
        /// Method signatures (no bodies)
        methods: Vec<crate::TraitMethodSig>,
        span: Span,
    },

    /// Destructuring variable declaration — Issue #668
    /// let { name, version } = config; or let [first, ...rest] = items;
    Destructure {
        pattern: crate::Pattern,
        value: Expr,
        is_const: bool,
        span: Span,
    },

    /// Class declaration: class Car <- Vehicle { ... }
    Class(crate::ClassDecl),

    /// Match statement: match x { Pattern => { ... } }
    Match {
        value: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// Enum declaration: enum Shape { Circle(radius), Rectangle(w, h), Point }
    EnumDecl {
        name: String,
        variants: Vec<EnumVariant>,
        span: Span,
    },

    // ── SOP statements ──────────────────────────────────────────────────
    /// Spawn a subject instance: spawn Player("args")
    Spawn {
        subject_name: String,
        args: Vec<Expr>,
        span: Span,
    },
    /// Despawn a subject instance: despawn hero
    Despawn {
        name: String,
        span: Span,
    },

    /// Send a message to a subject: send message to target
    Send {
        message: Box<Expr>,
        target: Box<Expr>,
        span: Span,
    },

    /// Receive a message: receive msg from source
    Receive {
        variable: String,
        source: Box<Expr>,
        span: Span,
    },

    /// Require a condition: require health > 0
    Require { condition: Box<Expr>, span: Span },

    /// Perform an action: perform attack
    Perform { action: Box<Expr>, span: Span },

    // ── RAG statements ─────────────────────────────────────────────────
    /// Remember statement: remember "text" in store
    Remember {
        content: Box<Expr>,
        store_name: Option<String>,
        span: Span,
    },

    /// Recall statement: recall "query" from store
    Recall {
        query: Box<Expr>,
        store_name: Option<String>,
        span: Span,
    },

    /// Forget statement: forget "id" from store
    Forget {
        target: Box<Expr>,
        store_name: Option<String>,
        span: Span,
    },
}
