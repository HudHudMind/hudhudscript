use crate::stmt::{Decl, Stmt};
use crate::Span;

impl Decl {
    /// Get the span of this declaration
    pub fn span(&self) -> Span {
        match self {
            Decl::Agent { span, .. } => *span,
            Decl::AgentAction { span, .. } => *span,
            Decl::Ability { span, .. } => *span,
            Decl::Action { span, .. } => *span,
            Decl::Tool { span, .. } => *span,
            Decl::Resource { span, .. } => *span,
            Decl::Import { span, .. } => *span,
            Decl::Constitution { span, .. } => *span,
            Decl::Law { span, .. } => *span,
            Decl::Council { span, .. } => *span,
            Decl::Rule { span, .. } => *span,
            Decl::Swarm { span, .. } => *span,
            Decl::Community { span, .. } => *span,
            Decl::Provider { span, .. } => *span,
            Decl::Protocol { span, .. } => *span,
            Decl::Governance { span, .. } => *span,
            Decl::Role { span, .. } => *span,
            Decl::Compose { span, .. } => *span,
            Decl::Store { span, .. } => *span,
            Decl::Strategy { span, .. } => *span,
            Decl::Subject { span, .. } => *span,
            Decl::Relation { span, .. } => *span,
            Decl::Effect { span, .. } => *span,
            Decl::Entity { span, .. } => *span,
            Decl::StateMachine { span, .. } => *span,
            Decl::Event { span, .. } => *span,
            Decl::Contract { span, .. } => *span,
            Decl::Treaty { span, .. } => *span,
            Decl::Music { span, .. } => *span,
            Decl::UiApp { span, .. } => *span,
            Decl::Deploy { span, .. } => *span,
        }
    }
}

impl Stmt {
    /// Get the span of this statement
    pub fn span(&self) -> Span {
        match self {
            Stmt::Decl(decl) => decl.span(),
            Stmt::McpServer(decl) => decl.span,
            Stmt::VarDecl(decl) => decl.span,
            Stmt::Let { span, .. } => *span,
            Stmt::Const { span, .. } => *span,
            Stmt::Assignment { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::While { span, .. } => *span,
            Stmt::For { span, .. } => *span,
            Stmt::ForCStyle { span, .. } => *span,
            Stmt::ForRange { span, .. } => *span,
            Stmt::Block { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
            Stmt::Break { span } => *span,
            Stmt::Continue { span } => *span,
            Stmt::Switch { span, .. } => *span,
            Stmt::Try { span, .. } => *span,
            Stmt::Throw { span, .. } => *span,
            Stmt::Expr(expr) => expr.span(),
            Stmt::Import { span, .. } => *span,
            Stmt::Export { span, .. } => *span,
            Stmt::Function { span, .. } => *span,
            Stmt::Trait { span, .. } => *span,
            Stmt::Destructure { span, .. } => *span,
            Stmt::Class(decl) => decl.span,
            Stmt::Match { span, .. } => *span,
            Stmt::EnumDecl { span, .. } => *span,
            Stmt::Spawn { span, .. } => *span,
            Stmt::Despawn { span, .. } => *span,
            Stmt::Send { span, .. } => *span,
            Stmt::Receive { span, .. } => *span,
            Stmt::Require { span, .. } => *span,
            Stmt::Perform { span, .. } => *span,
            Stmt::Remember { span, .. } => *span,
            Stmt::Recall { span, .. } => *span,
            Stmt::Forget { span, .. } => *span,
        }
    }
}
