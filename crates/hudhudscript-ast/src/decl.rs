//! Declaration AST nodes

use crate::{Expr, Span};
use serde::{Deserialize, Serialize};

/// Function/Task parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub span: Span,
}

/// Type annotation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeAnnotation {
    String,
    Number,
    Boolean,
    Null,
    Any,
    Tool,
    Resource,
    Server,
    Generic(String),
    Array(Box<TypeAnnotation>),
    Union(Vec<TypeAnnotation>),
    /// Parameterized type: Stack<T>, Map<K, V> — Issue #658
    Parameterized {
        base: Box<TypeAnnotation>,
        args: Vec<TypeAnnotation>,
    },
}

/// Generic type parameter with optional constraint — Issue #658
/// e.g. `T`, `T: Comparable`, `T: Serializable`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericParam {
    pub name: String,
    pub constraint: Option<String>,
    pub span: Span,
}

/// Trait method signature (no body) — Issue #659
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethodSig {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub span: Span,
}

/// Agent declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<crate::Stmt>,
    pub span: Span,
}

/// Task declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub is_async: bool,
    pub body: Vec<crate::Stmt>,
    pub span: Span,
}

/// Tool declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDecl {
    pub name: String,
    pub server: String,
    pub tool_name: String,
    pub span: Span,
}

/// Tool binding — binds a tool to a specific agent
///
/// Created when a tool is declared inside an agent body or explicitly
/// associated with an agent via `bind tool <name> to agent <agent>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolBinding {
    /// The agent name this tool is bound to
    pub agent_name: String,
    /// The tool name being bound
    pub tool_name: String,
    /// Optional alias the agent uses to refer to the tool
    pub alias: Option<String>,
    pub span: Span,
}

/// Inline tool definition (standalone, not from an MCP server)
///
/// `tool my_tool(param: String) -> String { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InlineToolDecl {
    pub name: String,
    pub description: Option<String>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<crate::Stmt>,
    /// Agents this tool is bound to (populated by the semantic pass)
    pub bound_agents: Vec<String>,
    pub span: Span,
}

/// Resource declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceDecl {
    pub name: String,
    pub server: String,
    pub resource_uri: String,
    pub span: Span,
}

/// MCP Server configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub transport: TransportType,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub auth: Option<AuthConfig>,
}

/// Transport type for MCP
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportType {
    Stdio,
    SSE,
}

/// Authentication configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Authentication type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthType {
    Bearer,
    Basic,
    ApiKey,
}

/// MCP tool definition inside an MCP server block — Issue #437
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolDef {
    pub name: String,
    pub params: Vec<(String, String)>, // (param_name, type_name)
    pub body: Vec<crate::Stmt>,
    pub span: Span,
}

/// MCP Server declaration — Issue #437: enhanced with tool definitions and fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerDecl {
    pub name: String,
    /// NOT authoritative. The parser leaves this at its default for every
    /// declaration (`transport: Stdio`, everything else `None`), so nothing
    /// downstream may read it: synthesizing the emitted object from it is what
    /// silently overrode a script's own `transport: "sse"` and made every
    /// server stdio. `fields` below is the single source of truth; the VM
    /// applies the defaults. Kept because it is part of the published AST
    /// shape.
    pub config: ServerConfig,
    /// The key-value fields the author wrote in the declaration body
    /// (`transport`, `command`, `args`, `url`, `auth`, …) — the single
    /// authoritative source for codegen and formatting.
    #[serde(default)]
    pub fields: Vec<(String, crate::Expr)>,
    /// Tool definitions nested inside this MCP server block
    #[serde(default)]
    pub tools: Vec<McpToolDef>,
    pub span: Span,
}

/// Ownership mode for variable declarations.
///
/// Controls whether a binding owns its data (value semantics) or holds a
/// reference (reference semantics).  This is the AST-level representation;
/// the type checker promotes it to `hudhudscript_types::Ownership`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OwnershipMode {
    /// `let x = ...` — the binding owns its data exclusively (default).
    #[default]
    Owned,
    /// `ref x = ...` / `&x` — shared immutable reference.
    Borrowed,
    /// `ref mut x = ...` / `&mut x` — shared mutable reference.
    MutBorrowed,
}

/// Variable declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarDecl {
    pub name: String,
    pub is_const: bool,
    pub type_annotation: Option<TypeAnnotation>,
    pub initializer: Option<Expr>,
    /// Ownership mode for this binding (Issue #330).
    /// Defaults to `Owned` for backward compatibility.
    #[serde(default)]
    pub ownership: OwnershipMode,
    pub span: Span,
}

/// Destructuring pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Simple identifier: var x = 1
    Identifier(String),
    /// Identifier with default value: let { port = 8080 } = config — Issue #668
    IdentifierDefault(String, Expr),
    /// Array destructuring: var [a, b, c] = arr
    Array {
        elements: Vec<Pattern>,
        rest: Option<Box<Pattern>>, // Rest element: ...rest
    },
    /// Object destructuring: var {name, age} = obj
    Object {
        properties: Vec<(String, Pattern)>, // key -> pattern
        rest: Option<Box<Pattern>>,         // Rest properties: ...rest
    },
}

/// Import kind
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportKind {
    /// Named imports: import { foo, bar } from "module"
    Named(Vec<String>),
    /// Default import: import foo from "module"
    Default(String),
    /// Wildcard import: import * as foo from "module"
    Wildcard(String),
}

/// Access modifier for class members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AccessModifier {
    Public,
    #[default]
    Private,
    Protected,
}

/// Class member (field or method)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClassMember {
    /// Field: [access] [static] var name = value
    Field {
        access: AccessModifier,
        is_static: bool,
        name: String,
        initializer: Option<Expr>,
        span: Span,
    },
    /// Method: [access] [static] function name(params) { body }
    Method {
        access: AccessModifier,
        is_static: bool,
        name: String,
        params: Vec<Param>,
        body: Vec<crate::Stmt>,
        span: Span,
    },
    /// Constructor: constructor(params) { body }
    Constructor {
        params: Vec<Param>,
        body: Vec<crate::Stmt>,
        span: Span,
    },
}

/// Class declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDecl {
    pub name: String,
    pub parent: Option<String>,
    /// Whether this class is declared with `abstract` keyword
    #[serde(default)]
    pub is_abstract: bool,
    /// Generic type parameters: class Stack<T> { ... } — Issue #658
    #[serde(default)]
    pub type_params: Vec<GenericParam>,
    /// Trait/interface implementations: class Dog implements Animal, Pet { ... } — Issue #659
    #[serde(default)]
    pub implements: Vec<String>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}
