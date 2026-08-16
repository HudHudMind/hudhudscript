//! Document symbol extraction (Issue #299)
//!
//! Walks the AST and returns all top-level declarations as LSP `DocumentSymbol`s.
//!
//! Uses `AstVisitor` from `hudhudscript_ast::visitor` for AST traversal
//! (migrated in #894 Phase 2).

use hudhudscript_ast::visitor::{walk_stmts, AstVisitor, VisitControl};
use hudhudscript_ast::{Decl, Stmt};
use tower_lsp::lsp_types::*;

/// Convert an AST span to an LSP Range (AST uses 1-indexed lines/columns; LSP uses 0-indexed).
fn span_to_range(span: hudhudscript_ast::Span) -> Range {
    Range {
        start: Position::new(
            span.start.line.saturating_sub(1) as u32,
            span.start.column.saturating_sub(1) as u32,
        ),
        end: Position::new(
            span.end.line.saturating_sub(1) as u32,
            span.end.column.saturating_sub(1) as u32,
        ),
    }
}

// ── AstVisitor-based symbol collector ──────────────────────────────────────

/// Visitor that collects document symbols from the AST.
/// We use `SkipChildren` on matched statements so that nested definitions
/// (e.g., a `let` inside a function body) are not reported as top-level symbols.
/// We still walk into `Stmt::Export` (and other wrapper nodes) to find the inner declaration.
#[allow(deprecated)]
struct SymbolCollector {
    symbols: Vec<DocumentSymbol>,
}

#[allow(deprecated)]
impl AstVisitor for SymbolCollector {
    fn visit_stmt(&mut self, stmt: &Stmt) -> VisitControl {
        if let Some(sym) = stmt_to_symbol(stmt) {
            self.symbols.push(sym);
            // Skip children — we already extracted child symbols (e.g., class members)
            // inside `stmt_to_symbol`, and we don't want to recurse into function
            // bodies and report inner definitions as top-level symbols.
            return VisitControl::SkipChildren;
        }
        // For non-symbol statements (e.g., Export wraps an inner item),
        // continue walking so the inner declaration is found.
        VisitControl::Continue
    }
}

/// Extract top-level document symbols from a list of AST statements.
#[allow(deprecated)]
pub fn extract_symbols(stmts: &[Stmt]) -> Vec<DocumentSymbol> {
    let mut collector = SymbolCollector {
        symbols: Vec::new(),
    };
    walk_stmts(&mut collector, stmts);
    collector.symbols
}

/// Convert a single statement to a DocumentSymbol, if it represents a named declaration.
#[allow(deprecated)]
fn stmt_to_symbol(stmt: &Stmt) -> Option<DocumentSymbol> {
    match stmt {
        // ── Functions ──
        Stmt::Function {
            name, params, span, ..
        } => Some(DocumentSymbol {
            name: name.clone(),
            detail: Some(format!("function({})", params.join(", "))),
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: span_to_range(*span),
            selection_range: span_to_range(*span),
            children: None,
        }),

        // ── Variables ──
        Stmt::Let { name, span, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: Some("let".to_string()),
            kind: SymbolKind::VARIABLE,
            tags: None,
            deprecated: None,
            range: span_to_range(*span),
            selection_range: span_to_range(*span),
            children: None,
        }),
        Stmt::Const { name, span, .. } => Some(DocumentSymbol {
            name: name.clone(),
            detail: Some("const".to_string()),
            kind: SymbolKind::CONSTANT,
            tags: None,
            deprecated: None,
            range: span_to_range(*span),
            selection_range: span_to_range(*span),
            children: None,
        }),
        Stmt::VarDecl(decl) => Some(DocumentSymbol {
            name: decl.name.clone(),
            detail: Some(if decl.is_const { "const" } else { "let" }.to_string()),
            kind: if decl.is_const {
                SymbolKind::CONSTANT
            } else {
                SymbolKind::VARIABLE
            },
            tags: None,
            deprecated: None,
            range: span_to_range(decl.span),
            selection_range: span_to_range(decl.span),
            children: None,
        }),

        // ── Classes ──
        Stmt::Class(decl) => {
            let children: Vec<DocumentSymbol> = decl
                .members
                .iter()
                .map(|m| match m {
                    hudhudscript_ast::ClassMember::Method {
                        name, span, params, ..
                    } => DocumentSymbol {
                        name: name.clone(),
                        detail: Some(format!(
                            "method({})",
                            params
                                .iter()
                                .map(|p| p.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                        kind: SymbolKind::METHOD,
                        tags: None,
                        deprecated: None,
                        range: span_to_range(*span),
                        selection_range: span_to_range(*span),
                        children: None,
                    },
                    hudhudscript_ast::ClassMember::Field { name, span, .. } => DocumentSymbol {
                        name: name.clone(),
                        detail: Some("field".to_string()),
                        kind: SymbolKind::FIELD,
                        tags: None,
                        deprecated: None,
                        range: span_to_range(*span),
                        selection_range: span_to_range(*span),
                        children: None,
                    },
                    hudhudscript_ast::ClassMember::Constructor { span, .. } => DocumentSymbol {
                        name: "constructor".to_string(),
                        detail: Some("constructor".to_string()),
                        kind: SymbolKind::CONSTRUCTOR,
                        tags: None,
                        deprecated: None,
                        range: span_to_range(*span),
                        selection_range: span_to_range(*span),
                        children: None,
                    },
                })
                .collect();

            Some(DocumentSymbol {
                name: decl.name.clone(),
                detail: decl.parent.as_ref().map(|p| format!("extends {}", p)),
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                range: span_to_range(decl.span),
                selection_range: span_to_range(decl.span),
                children: if children.is_empty() {
                    None
                } else {
                    Some(children)
                },
            })
        }

        // ── Enums ──
        Stmt::EnumDecl {
            name,
            variants,
            span,
        } => {
            let children: Vec<DocumentSymbol> = variants
                .iter()
                .map(|v| DocumentSymbol {
                    name: v.name.clone(),
                    detail: if v.fields.is_empty() {
                        None
                    } else {
                        Some(format!("({})", v.fields.join(", ")))
                    },
                    kind: SymbolKind::ENUM_MEMBER,
                    tags: None,
                    deprecated: None,
                    range: span_to_range(v.span),
                    selection_range: span_to_range(v.span),
                    children: None,
                })
                .collect();

            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some("enum".to_string()),
                kind: SymbolKind::ENUM,
                tags: None,
                deprecated: None,
                range: span_to_range(*span),
                selection_range: span_to_range(*span),
                children: if children.is_empty() {
                    None
                } else {
                    Some(children)
                },
            })
        }

        // ── Decl-based declarations ──
        Stmt::Decl(decl) => decl_to_symbol(decl),

        _ => None,
    }
}

/// Convert a `Decl` variant to a `DocumentSymbol`.
#[allow(deprecated)]
fn decl_to_symbol(decl: &Decl) -> Option<DocumentSymbol> {
    let (name, detail, kind, span) = match decl {
        Decl::Agent { name, span, .. } => (name.clone(), "agent", SymbolKind::CLASS, *span),
        Decl::AgentAction { name, span, .. } => {
            (name.clone(), "action", SymbolKind::FUNCTION, *span)
        }
        Decl::Ability { name, span, .. } => (name.clone(), "method", SymbolKind::METHOD, *span),
        Decl::Action { name, span, .. } => (name.clone(), "action", SymbolKind::FUNCTION, *span),
        Decl::Tool { name, span, .. } => (name.clone(), "tool", SymbolKind::FUNCTION, *span),
        Decl::Resource { name, span, .. } => (name.clone(), "resource", SymbolKind::FILE, *span),
        Decl::Subject { name, span, .. } => (name.clone(), "subject", SymbolKind::CLASS, *span),
        Decl::Role { name, span, .. } => (name.clone(), "role", SymbolKind::INTERFACE, *span),
        Decl::Constitution { name, span, .. } => {
            (name.clone(), "constitution", SymbolKind::NAMESPACE, *span)
        }
        Decl::Law { name, span, .. } => (name.clone(), "law", SymbolKind::PROPERTY, *span),
        Decl::Council { name, span, .. } => (name.clone(), "council", SymbolKind::NAMESPACE, *span),
        Decl::Rule { name, span, .. } => (name.clone(), "rule", SymbolKind::PROPERTY, *span),
        Decl::Swarm { name, span, .. } => (name.clone(), "swarm", SymbolKind::NAMESPACE, *span),
        Decl::Community { name, span, .. } => {
            (name.clone(), "community", SymbolKind::NAMESPACE, *span)
        }
        Decl::Provider { name, span, .. } => (name.clone(), "provider", SymbolKind::MODULE, *span),
        Decl::Protocol { name, span, .. } => {
            (name.clone(), "protocol", SymbolKind::INTERFACE, *span)
        }
        Decl::Governance { name, span, .. } => {
            (name.clone(), "governance", SymbolKind::NAMESPACE, *span)
        }
        Decl::Store { name, span, .. } => (name.clone(), "store", SymbolKind::STRUCT, *span),
        Decl::Strategy { name, span, .. } => {
            (name.clone(), "strategy", SymbolKind::INTERFACE, *span)
        }
        Decl::Relation {
            subject_a,
            subject_b,
            span,
            ..
        } => (
            format!("{} <-> {}", subject_a, subject_b),
            "relation",
            SymbolKind::STRUCT,
            *span,
        ),
        Decl::Effect {
            event_name, span, ..
        } => (event_name.clone(), "effect", SymbolKind::EVENT, *span),
        Decl::Compose {
            base_subject, span, ..
        } => (
            base_subject.clone(),
            "compose",
            SymbolKind::INTERFACE,
            *span,
        ),
        Decl::Import { module, span, .. } => (module.clone(), "import", SymbolKind::MODULE, *span),
        Decl::Entity { name, span, .. } => (name.clone(), "entity", SymbolKind::CLASS, *span),
        Decl::StateMachine { name, span, .. } => {
            (name.clone(), "statemachine", SymbolKind::CLASS, *span)
        }
        Decl::Event { name, span, .. } => (name.clone(), "event", SymbolKind::EVENT, *span),
        Decl::Contract { name, span, .. } => {
            (name.clone(), "contract", SymbolKind::INTERFACE, *span)
        }
        Decl::Treaty { name, span, .. } => (name.clone(), "treaty", SymbolKind::INTERFACE, *span),
        Decl::Music { name, span, .. } => (name.clone(), "music", SymbolKind::FUNCTION, *span),
        Decl::UiApp { name, span, .. } => (name.clone(), "ui", SymbolKind::MODULE, *span),
        Decl::Deploy { name, span, .. } => (name.clone(), "deploy", SymbolKind::MODULE, *span),
        _ => return None,
    };

    Some(DocumentSymbol {
        name,
        detail: Some(detail.to_string()),
        kind,
        tags: None,
        deprecated: None,
        range: span_to_range(span),
        selection_range: span_to_range(span),
        children: None,
    })
}
