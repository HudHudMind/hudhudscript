//! AST-based hover symbol finder.

use hudhudscript_ast::visitor::{walk_stmts, AstVisitor, VisitControl};
use hudhudscript_ast::{Decl, Stmt};
use hudhudscript_parser::parse;

use crate::hover::HoverInfo;

/// Visitor that searches for a named symbol and extracts hover info.
/// Stops traversal as soon as a match is found.
struct HoverFinder<'a> {
    target: &'a str,
    result: Option<HoverInfo>,
}

impl<'a> AstVisitor for HoverFinder<'a> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> VisitControl {
        if let Some(info) = match_stmt(self.target, stmt) {
            self.result = Some(info);
            return VisitControl::Stop;
        }
        VisitControl::Continue
    }

    fn visit_decl(&mut self, decl: &Decl) -> VisitControl {
        if let Some(info) = match_decl(self.target, decl) {
            self.result = Some(info);
            return VisitControl::Stop;
        }
        VisitControl::Continue
    }
}

/// Search the parsed AST for a declaration matching `name`.
pub fn symbol_info_for(name: &str, source: &str) -> Option<HoverInfo> {
    let ast = match crate::server::helpers::isolate(|| parse(source))? {
        Ok(ast) => ast,
        Err(_) => return None,
    };

    let mut finder = HoverFinder {
        target: name,
        result: None,
    };
    walk_stmts(&mut finder, &ast);
    finder.result
}

/// Match a statement against a symbol name.
pub fn match_stmt(name: &str, stmt: &Stmt) -> Option<HoverInfo> {
    match stmt {
        Stmt::Function {
            name: fn_name,
            params,
            is_async,
            ..
        } if fn_name == name => {
            let async_prefix = if *is_async { "async " } else { "" };
            Some(HoverInfo {
                signature: format!(
                    "{}function {}({})",
                    async_prefix,
                    fn_name,
                    params.join(", ")
                ),
                docs: Some("User-defined function".to_string()),
            })
        }

        Stmt::Let { name: var_name, .. } if var_name == name => Some(HoverInfo {
            signature: format!("let {}", var_name),
            docs: Some("Mutable variable binding".to_string()),
        }),

        Stmt::Const { name: var_name, .. } if var_name == name => Some(HoverInfo {
            signature: format!("const {}", var_name),
            docs: Some("Immutable constant binding".to_string()),
        }),

        Stmt::VarDecl(decl) if decl.name == name => {
            let kw = if decl.is_const { "const" } else { "let" };
            let type_str = decl
                .type_annotation
                .as_ref()
                .map(|t| format!(": {:?}", t))
                .unwrap_or_default();
            Some(HoverInfo {
                signature: format!("{} {}{}", kw, decl.name, type_str),
                docs: Some(
                    if decl.is_const {
                        "Immutable constant binding"
                    } else {
                        "Mutable variable binding"
                    }
                    .to_string(),
                ),
            })
        }

        Stmt::Class(decl) if decl.name == name => {
            let parent_str = decl
                .parent
                .as_ref()
                .map(|p| format!(" <- {}", p))
                .unwrap_or_default();
            Some(HoverInfo {
                signature: format!("class {}{}", decl.name, parent_str),
                docs: Some("Class declaration".to_string()),
            })
        }

        Stmt::EnumDecl {
            name: enum_name,
            variants,
            ..
        } if enum_name == name => {
            let variant_names: Vec<&str> = variants.iter().map(|v| v.name.as_str()).collect();
            Some(HoverInfo {
                signature: format!("enum {}", enum_name),
                docs: Some(format!("Variants: {}", variant_names.join(", "))),
            })
        }

        // Stmt::Decl is handled by the visitor's `visit_decl` method —
        // the walker recurses into Decl children automatically.
        _ => None,
    }
}

/// Match a Decl variant against a name.
pub fn match_decl(name: &str, decl: &Decl) -> Option<HoverInfo> {
    match decl {
        Decl::Agent {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("agent {}", decl_name),
            docs: Some("Agent declaration".to_string()),
        }),
        Decl::Action {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("action {}", decl_name),
            docs: Some("Action declaration".to_string()),
        }),
        Decl::Tool {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("tool {}", decl_name),
            docs: Some("Tool declaration".to_string()),
        }),
        Decl::Resource {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("resource {}", decl_name),
            docs: Some("Resource declaration".to_string()),
        }),
        Decl::Subject {
            name: decl_name,
            roles,
            capabilities,
            ..
        } if decl_name == name => {
            let mut doc = "Subject declaration (SOP)".to_string();
            if !roles.is_empty() {
                doc.push_str(&format!("\n\nRoles: {}", roles.join(", ")));
            }
            if !capabilities.is_empty() {
                doc.push_str(&format!("\n\nCapabilities: {}", capabilities.join(", ")));
            }
            Some(HoverInfo {
                signature: format!("subject {}", decl_name),
                docs: Some(doc),
            })
        }
        Decl::Role {
            name: decl_name,
            capabilities,
            ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("role {}", decl_name),
            docs: Some(format!(
                "Role declaration\n\nCapabilities: {}",
                if capabilities.is_empty() {
                    "(none)".to_string()
                } else {
                    capabilities.join(", ")
                }
            )),
        }),
        Decl::Compose { base_subject, .. } if base_subject == name => Some(HoverInfo {
            signature: format!("compose {}", base_subject),
            docs: Some("Composition rules (Harrison & Ossher SOP)".to_string()),
        }),
        Decl::Constitution {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("constitution {}", decl_name),
            docs: Some("Constitution (governance framework)".to_string()),
        }),
        Decl::Provider {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("provider {}", decl_name),
            docs: Some("LLM provider declaration".to_string()),
        }),
        Decl::Protocol {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("protocol {}", decl_name),
            docs: Some("Protocol declaration".to_string()),
        }),
        Decl::Governance {
            name: decl_name,
            base_type,
            ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("governance {}: {}", decl_name, base_type),
            docs: Some("Governance model declaration".to_string()),
        }),
        Decl::Store {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("store {}", decl_name),
            docs: Some("RAG vector store".to_string()),
        }),
        Decl::Strategy {
            name: decl_name, ..
        } if decl_name == name => Some(HoverInfo {
            signature: format!("strategy {}", decl_name),
            docs: Some("Strategy declaration".to_string()),
        }),
        _ => None,
    }
}
