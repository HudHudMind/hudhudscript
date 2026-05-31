//! Go-to-definition provider (Issue #297)
//!
//! Tracks symbol definitions during AST traversal and returns the definition
//! location for the identifier at the cursor position.
//!
//! Uses `AstVisitor` from `hudhudscript_ast::visitor` for AST traversal
//! (migrated in #894 Phase 2).

use hudhudscript_ast::visitor::{walk_stmts, AstVisitor, VisitControl};
use hudhudscript_ast::{Decl, Stmt};
use hudhudscript_parser::parse;
use tower_lsp::lsp_types::*;

/// A definition entry: name -> location in the file.
#[derive(Debug, Clone)]
pub struct DefinitionEntry {
    pub name: String,
    pub range: Range,
}

/// Convert an AST span to an LSP Range (AST uses 1-indexed; LSP uses 0-indexed).
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

/// Find the definition of the symbol at the given position.
///
/// Returns a `GotoDefinitionResponse` with the location within the same file,
/// or `None` if no definition was found.
pub fn goto_definition(
    uri: &Url,
    source: &str,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    let word = word_at_position(source, position)?;
    let defs = collect_definitions(source);

    let entry = defs.iter().find(|e| e.name == word)?;

    Some(GotoDefinitionResponse::Scalar(Location {
        uri: uri.clone(),
        range: entry.range,
    }))
}

/// Extract the word at a given position (same as in hover.rs).
pub fn word_at_position(source: &str, pos: Position) -> Option<String> {
    let line_idx = pos.line as usize;
    let col = pos.character as usize;

    let line = source.lines().nth(line_idx)?;

    if col >= line.len() {
        return None;
    }

    let start = line[..col]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    let end = line[col..]
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| col + i)
        .unwrap_or(line.len());

    if start >= end {
        return None;
    }

    Some(line[start..end].to_string())
}

// ── AstVisitor-based definition collector ──────────────────────────────────

/// Visitor that collects named definition entries from the AST.
struct DefinitionCollector {
    defs: Vec<DefinitionEntry>,
}

impl AstVisitor for DefinitionCollector {
    fn visit_stmt(&mut self, stmt: &Stmt) -> VisitControl {
        match stmt {
            Stmt::Function { name, span, .. } => {
                self.defs.push(DefinitionEntry {
                    name: name.clone(),
                    range: span_to_range(*span),
                });
            }

            Stmt::Let { name, span, .. } | Stmt::Const { name, span, .. } => {
                self.defs.push(DefinitionEntry {
                    name: name.clone(),
                    range: span_to_range(*span),
                });
            }

            Stmt::VarDecl(decl) => {
                self.defs.push(DefinitionEntry {
                    name: decl.name.clone(),
                    range: span_to_range(decl.span),
                });
            }

            Stmt::For { variable, span, .. } => {
                self.defs.push(DefinitionEntry {
                    name: variable.clone(),
                    range: span_to_range(*span),
                });
            }

            Stmt::Class(decl) => {
                self.defs.push(DefinitionEntry {
                    name: decl.name.clone(),
                    range: span_to_range(decl.span),
                });
            }

            Stmt::EnumDecl {
                name,
                span,
                variants,
                ..
            } => {
                self.defs.push(DefinitionEntry {
                    name: name.clone(),
                    range: span_to_range(*span),
                });
                for v in variants {
                    self.defs.push(DefinitionEntry {
                        name: v.name.clone(),
                        range: span_to_range(v.span),
                    });
                }
            }

            _ => {}
        }
        // Continue — the walker handles recursion into blocks, if/else, etc.
        VisitControl::Continue
    }

    fn visit_decl(&mut self, decl: &Decl) -> VisitControl {
        let (name, span) = match decl {
            Decl::Agent { name, span, .. } => (name.clone(), *span),
            Decl::AgentAction { name, span, .. } => (name.clone(), *span),
            Decl::Ability { name, span, .. } => (name.clone(), *span),
            Decl::Action { name, span, .. } => (name.clone(), *span),
            Decl::Tool { name, span, .. } => (name.clone(), *span),
            Decl::Resource { name, span, .. } => (name.clone(), *span),
            Decl::Subject { name, span, .. } => (name.clone(), *span),
            Decl::Role { name, span, .. } => (name.clone(), *span),
            Decl::Constitution { name, span, .. } => (name.clone(), *span),
            Decl::Law { name, span, .. } => (name.clone(), *span),
            Decl::Council { name, span, .. } => (name.clone(), *span),
            Decl::Rule { name, span, .. } => (name.clone(), *span),
            Decl::Swarm { name, span, .. } => (name.clone(), *span),
            Decl::Community { name, span, .. } => (name.clone(), *span),
            Decl::Provider { name, span, .. } => (name.clone(), *span),
            Decl::Protocol { name, span, .. } => (name.clone(), *span),
            Decl::Governance { name, span, .. } => (name.clone(), *span),
            Decl::Store { name, span, .. } => (name.clone(), *span),
            Decl::Strategy { name, span, .. } => (name.clone(), *span),
            Decl::Import { module, span, .. } => (module.clone(), *span),
            Decl::Relation {
                subject_a, span, ..
            } => (subject_a.clone(), *span),
            Decl::Effect {
                event_name, span, ..
            } => (event_name.clone(), *span),
            Decl::Compose { base_subject, span, .. } => (base_subject.clone(), *span),
            Decl::Entity { name, span, .. } => (name.clone(), *span),
            Decl::StateMachine { name, span, .. } => (name.clone(), *span),
            Decl::Event { name, span, .. } => (name.clone(), *span),
            Decl::Contract { name, span, .. } => (name.clone(), *span),
            Decl::Treaty { name, span, .. } => (name.clone(), *span),
            Decl::Music { name, span, .. } => (name.clone(), *span),
            Decl::UiApp { name, span, .. } => (name.clone(), *span),
            Decl::Deploy { name, span, .. } => (name.clone(), *span),
        };

        self.defs.push(DefinitionEntry {
            name,
            range: span_to_range(span),
        });
        VisitControl::Continue
    }
}

/// Walk the AST and collect all named definition locations.
pub fn collect_definitions(source: &str) -> Vec<DefinitionEntry> {
    let ast = match parse(source) {
        Ok(ast) => ast,
        Err(_) => return Vec::new(),
    };

    let mut collector = DefinitionCollector { defs: Vec::new() };
    walk_stmts(&mut collector, &ast);
    collector.defs
}
