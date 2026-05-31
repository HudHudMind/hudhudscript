//! HudHudScript Abstract Syntax Tree
//!
//! This crate defines the AST node structures for HudHudScript.
//!
//! Agent, task, tool, and resource declarations are unified under `Stmt::Decl(Decl::...)`.
//! Both the pest-based parser and the legacy parser emit `Decl` variants exclusively.

pub mod annotated;
mod decl;
mod expr;
mod span;
mod stmt;
pub mod visitor;

pub use decl::{
    AccessModifier, AgentDecl, AuthConfig, AuthType, ClassDecl, ClassMember, GenericParam,
    ImportKind, InlineToolDecl, McpServerDecl, McpToolDef, OwnershipMode, Param, Pattern,
    ResourceDecl, ServerConfig, TaskDecl, ToolBinding, ToolDecl, TraitMethodSig, TransportType,
    TypeAnnotation, VarDecl,
};
pub use expr::{ArrowFunctionBody, BinaryOp, Expr, Literal, TemplateStringPart, UnaryOp};
pub use span::{Position, Span};
pub use stmt::{
    ActionDecl, AgentActionDecl, CatchClause, ConditionDecl, CouncilMemberDecl, CultureDecl, Decl, Decorator,
    DeployProviderDecl, DeployTargetDecl, EnumVariant, LawDecl, MatchArm, MatchPattern, Stmt,
    SubjectAbilityDef, ComposeMode, ComposeRule, FieldCorrespondence, SwitchCase, UiComponentDecl, UiNode, UiScreenDecl,
};
