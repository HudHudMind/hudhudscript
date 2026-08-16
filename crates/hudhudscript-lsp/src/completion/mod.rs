//! Context-aware completion provider (Issue #300)

use std::collections::HashSet;

use hudhudscript_ast::{Decl, Stmt};
use hudhudscript_parser::parse;

mod keywords_ar;
mod keywords_en;
mod keywords_tr;
pub mod snippets;

/// Completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    /// Non-default text to insert (used for snippets).
    pub insert_text: Option<String>,
}

impl CompletionItem {
    fn keyword(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: CompletionKind::Keyword,
            detail: Some(detail.into()),
            insert_text: None,
        }
    }

    fn field(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: CompletionKind::Field,
            detail: Some(detail.into()),
            insert_text: None,
        }
    }

    fn snippet(
        label: impl Into<String>,
        detail: impl Into<String>,
        insert_text: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            kind: CompletionKind::Snippet,
            detail: Some(detail.into()),
            insert_text: Some(insert_text.into()),
        }
    }
}

/// Completion kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Function,
    Variable,
    Module,
    Snippet,
    Field,
}

/// The context surrounding the cursor position
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorContext {
    /// Top-level scope — offer keywords, declarations
    TopLevel,
    /// Inside an agent body — offer agent-specific fields
    AgentBody,
    /// Inside a task/action body — offer statements/expressions
    TaskBody,
    /// Inside a subject body — offer SOP-specific keywords
    SubjectBody,
    /// After a dot — offer member completion
    MemberAccess { object: String },
    /// Inside a block (generic) — offer statements/expressions
    Block,
}

/// Completion provider
pub struct CompletionProvider {
    keywords: Vec<String>,
}

impl CompletionProvider {
    pub fn new() -> Self {
        let mut seen = HashSet::new();
        let mut keywords = Vec::new();
        for kw in keywords_en::KEYWORDS
            .iter()
            .chain(keywords_tr::KEYWORDS)
            .chain(keywords_ar::KEYWORDS)
        {
            if seen.insert(*kw) {
                keywords.push(kw.to_string());
            }
        }
        keywords.sort();
        Self { keywords }
    }

    /// Determine the cursor context from source text and cursor offset.
    pub fn determine_context(&self, text: &str, offset: usize) -> CursorContext {
        let before_cursor = text.get(..offset).unwrap_or(text);

        // Check for dot-access.
        let trimmed = before_cursor.trim_end();
        if let Some(before_dot) = trimmed.strip_suffix('.') {
            let object = before_dot
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .next_back()
                .unwrap_or("")
                .to_string();
            return CursorContext::MemberAccess { object };
        }

        let mut depth = 0i32;
        let mut context_stack: Vec<&str> = Vec::new();

        for line in before_cursor.lines() {
            let trimmed_line = line.trim();

            if trimmed_line.starts_with("agent ") && trimmed_line.contains('{') {
                depth += 1;
                context_stack.push("agent");
                continue;
            }
            if (trimmed_line.starts_with("task ") || trimmed_line.starts_with("action "))
                && trimmed_line.contains('{')
            {
                depth += 1;
                context_stack.push("task");
                continue;
            }
            if trimmed_line.starts_with("subject ") && trimmed_line.contains('{') {
                depth += 1;
                context_stack.push("subject");
                continue;
            }

            for ch in line.chars() {
                if ch == '{' {
                    depth += 1;
                    if context_stack.len() < depth as usize {
                        context_stack.push("block");
                    }
                } else if ch == '}' {
                    depth -= 1;
                    if !context_stack.is_empty() {
                        context_stack.pop();
                    }
                }
            }
        }

        match context_stack.last().copied() {
            Some("agent") => CursorContext::AgentBody,
            Some("task") => CursorContext::TaskBody,
            Some("subject") => CursorContext::SubjectBody,
            Some("block") => CursorContext::Block,
            _ => CursorContext::TopLevel,
        }
    }

    /// Provide context-aware completions.
    pub fn complete(&self, text: &str, offset: usize) -> Vec<CompletionItem> {
        let context = self.determine_context(text, offset);
        let mut items = Vec::new();

        match context {
            CursorContext::TopLevel => {
                self.add_keywords(&mut items);
                self.add_snippets(&mut items, snippets::DECLARATION_SNIPPETS);
                self.add_snippets(&mut items, snippets::STATEMENT_SNIPPETS);
                items.extend(self.symbols_from_source(text));
            }
            CursorContext::AgentBody => {
                self.add_agent_fields(&mut items);
                self.add_keywords(&mut items);
                self.add_snippets(&mut items, snippets::STATEMENT_SNIPPETS);
            }
            CursorContext::SubjectBody => {
                self.add_subject_fields(&mut items);
                self.add_keywords(&mut items);
                self.add_snippets(&mut items, snippets::STATEMENT_SNIPPETS);
            }
            CursorContext::TaskBody | CursorContext::Block => {
                self.add_keywords(&mut items);
                self.add_snippets(&mut items, snippets::STATEMENT_SNIPPETS);
                items.extend(self.symbols_from_source(text));
            }
            CursorContext::MemberAccess { ref object } => {
                self.add_member_access(&mut items, object);
            }
        }

        items
    }

    fn add_keywords(&self, items: &mut Vec<CompletionItem>) {
        for kw in &self.keywords {
            items.push(CompletionItem::keyword(kw, format!("{} keyword", kw)));
        }
    }

    fn add_snippets(&self, items: &mut Vec<CompletionItem>, groups: &[&[snippets::Snippet]]) {
        for group in groups {
            for s in *group {
                items.push(CompletionItem::snippet(s.label, s.detail, s.insert_text));
            }
        }
    }

    fn add_agent_fields(&self, items: &mut Vec<CompletionItem>) {
        let fields = [
            ("model", "AI model to use"),
            ("provider", "LLM provider"),
            ("instructions", "Agent instructions/system prompt"),
            ("tools", "Tools available to the agent"),
            ("temperature", "Sampling temperature"),
            ("max_tokens", "Maximum token limit"),
        ];
        for (name, detail) in fields {
            items.push(CompletionItem::field(name, detail));
        }
    }

    fn add_subject_fields(&self, items: &mut Vec<CompletionItem>) {
        let fields = [
            ("state", "State variable declaration"),
            ("can", "Capability declaration"),
            ("intent", "Intent declaration"),
            ("has", "Role assignment"),
            ("uses", "Provider usage"),
            ("memory", "Memory field"),
            ("perception", "Perception field"),
        ];
        for (name, detail) in fields {
            items.push(CompletionItem::field(name, detail));
        }
    }

    fn add_member_access(&self, items: &mut Vec<CompletionItem>, object: &str) {
        if let Some(module) = hudhudscript_bytecode::registry::get_module(object) {
            for member in module.members {
                let kind = if member.kind == hudhudscript_bytecode::registry::MemberKind::Constant {
                    CompletionKind::Field
                } else {
                    CompletionKind::Function
                };
                items.push(CompletionItem {
                    label: member.name.to_string(),
                    kind,
                    detail: Some(member.description.to_string()),
                    insert_text: None,
                });
            }
        } else {
            let defaults = [
                ("length", "Length/size property"),
                ("push", "Add element"),
                ("pop", "Remove last element"),
                ("map", "Map over elements"),
                ("filter", "Filter elements"),
                ("forEach", "Iterate elements"),
                ("toString", "Convert to string"),
            ];
            for (name, detail) in defaults {
                items.push(CompletionItem::keyword(name, detail));
            }
        }
    }

    /// Extract user-defined symbols from the source for completion.
    fn symbols_from_source(&self, text: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        let ast = match crate::server::helpers::isolate(|| parse(text)) {
            Some(Ok(ast)) => ast,
            _ => return items,
        };

        for stmt in &ast {
            match stmt {
                Stmt::Function { name, params, .. } => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Function,
                        detail: Some(format!("function({})", params.join(", "))),
                        insert_text: None,
                    });
                }
                Stmt::Let { name, .. } | Stmt::Const { name, .. } => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Variable,
                        detail: Some("variable".to_string()),
                        insert_text: None,
                    });
                }
                Stmt::VarDecl(decl) => {
                    items.push(CompletionItem {
                        label: decl.name.clone(),
                        kind: CompletionKind::Variable,
                        detail: Some(if decl.is_const { "const" } else { "variable" }.to_string()),
                        insert_text: None,
                    });
                }
                Stmt::Decl(Decl::Agent { name, .. }) => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Module,
                        detail: Some("agent".to_string()),
                        insert_text: None,
                    });
                }
                Stmt::Decl(Decl::Subject { name, .. }) => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Module,
                        detail: Some("subject".to_string()),
                        insert_text: None,
                    });
                }
                Stmt::Class(decl) => {
                    items.push(CompletionItem {
                        label: decl.name.clone(),
                        kind: CompletionKind::Module,
                        detail: Some("class".to_string()),
                        insert_text: None,
                    });
                }
                Stmt::EnumDecl { name, .. } => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Module,
                        detail: Some("enum".to_string()),
                        insert_text: None,
                    });
                }
                _ => {}
            }
        }

        items
    }
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}
