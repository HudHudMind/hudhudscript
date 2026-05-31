//! Context-aware completion provider (Issue #300)

use hudhudscript_ast::{Decl, Stmt};
use hudhudscript_parser::parse;

/// Completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
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
        Self {
            keywords: vec![
                "agent".to_string(),
                "task".to_string(),
                "action".to_string(),
                "tool".to_string(),
                "resource".to_string(),
                "let".to_string(),
                "const".to_string(),
                "if".to_string(),
                "else".to_string(),
                "while".to_string(),
                "for".to_string(),
                "return".to_string(),
                "async".to_string(),
                "await".to_string(),
                "function".to_string(),
                "class".to_string(),
                "import".to_string(),
                "export".to_string(),
                "subject".to_string(),
                "role".to_string(),
                "spawn".to_string(),
                "send".to_string(),
                "receive".to_string(),
                "enum".to_string(),
                "match".to_string(),
                "try".to_string(),
                "catch".to_string(),
                "throw".to_string(),
                "switch".to_string(),
                "break".to_string(),
                "continue".to_string(),
                "store".to_string(),
                "remember".to_string(),
                "recall".to_string(),
                "constitution".to_string(),
                "governance".to_string(),
                "protocol".to_string(),
                "strategy".to_string(),
                "provider".to_string(),
            ],
        }
    }

    /// Determine the cursor context from source text and cursor offset.
    pub fn determine_context(&self, text: &str, offset: usize) -> CursorContext {
        let before_cursor = if offset <= text.len() {
            &text[..offset]
        } else {
            text
        };

        // Check for dot-access: the character right before cursor is '.' or
        // the last non-whitespace token before cursor ends with '.'
        let trimmed = before_cursor.trim_end();
        if let Some(before_dot) = trimmed.strip_suffix('.') {
            // Extract the object name before the dot
            let object = before_dot
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .next_back()
                .unwrap_or("")
                .to_string();
            return CursorContext::MemberAccess { object };
        }

        // Count brace depth and determine enclosing declaration
        let mut depth = 0i32;
        // Track what declaration we are inside at each depth
        let mut context_stack: Vec<&str> = Vec::new();

        // Simple line-by-line scan
        for line in before_cursor.lines() {
            let trimmed_line = line.trim();

            // Detect declaration openings
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

            // Count remaining braces
            for ch in line.chars() {
                if ch == '{' {
                    depth += 1;
                    // If we don't know the context, mark as block
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
                // Offer all keywords
                for kw in &self.keywords {
                    items.push(CompletionItem {
                        label: kw.clone(),
                        kind: CompletionKind::Keyword,
                        detail: Some(format!("{} declaration", kw)),
                    });
                }
                // Add symbols found in the document
                items.extend(self.symbols_from_source(text));
            }
            CursorContext::AgentBody => {
                // Agent-specific fields
                let agent_fields = [
                    ("model", "AI model to use"),
                    ("provider", "LLM provider"),
                    ("instructions", "Agent instructions/system prompt"),
                    ("tools", "Tools available to the agent"),
                    ("temperature", "Sampling temperature"),
                    ("max_tokens", "Maximum token limit"),
                ];
                for (field, desc) in agent_fields {
                    items.push(CompletionItem {
                        label: field.to_string(),
                        kind: CompletionKind::Field,
                        detail: Some(desc.to_string()),
                    });
                }
                // Also offer statement keywords
                self.add_statement_keywords(&mut items);
            }
            CursorContext::SubjectBody => {
                let subject_fields = [
                    ("state", "State variable declaration"),
                    ("can", "Capability declaration"),
                    ("intent", "Intent declaration"),
                    ("has", "Role assignment"),
                    ("uses", "Provider usage"),
                    ("memory", "Memory field"),
                    ("perception", "Perception field"),
                ];
                for (field, desc) in subject_fields {
                    items.push(CompletionItem {
                        label: field.to_string(),
                        kind: CompletionKind::Field,
                        detail: Some(desc.to_string()),
                    });
                }
                self.add_statement_keywords(&mut items);
            }
            CursorContext::TaskBody | CursorContext::Block => {
                self.add_statement_keywords(&mut items);
                items.extend(self.symbols_from_source(text));
            }
            CursorContext::MemberAccess { ref object } => {
                // Look up builtin module from central registry
                if let Some(module) = hudhudscript_bytecode::registry::get_module(object) {
                    for member in module.members {
                        items.push(CompletionItem {
                            label: member.name.to_string(),
                            kind: if member.kind
                                == hudhudscript_bytecode::registry::MemberKind::Constant
                            {
                                CompletionKind::Field
                            } else {
                                CompletionKind::Function
                            },
                            detail: Some(member.description.to_string()),
                        });
                    }
                } else {
                    // Fallback: common methods for unknown objects
                    let defaults = [
                        ("length", "Length/size property"),
                        ("push", "Add element"),
                        ("pop", "Remove last element"),
                        ("map", "Map over elements"),
                        ("filter", "Filter elements"),
                        ("forEach", "Iterate elements"),
                        ("toString", "Convert to string"),
                    ];
                    for (name, desc) in defaults {
                        items.push(CompletionItem {
                            label: name.to_string(),
                            kind: CompletionKind::Function,
                            detail: Some(desc.to_string()),
                        });
                    }
                }
            }
        }

        items
    }

    /// Add common statement-level keywords.
    fn add_statement_keywords(&self, items: &mut Vec<CompletionItem>) {
        let stmt_keywords = [
            "let", "const", "if", "else", "while", "for", "return", "break", "continue", "try",
            "catch", "throw", "switch", "match", "await", "async",
        ];
        for kw in stmt_keywords {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: CompletionKind::Keyword,
                detail: None,
            });
        }
    }

    /// Extract user-defined symbols from the source for completion.
    fn symbols_from_source(&self, text: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        let ast = match parse(text) {
            Ok(ast) => ast,
            Err(_) => return items,
        };

        for stmt in &ast {
            match stmt {
                Stmt::Function { name, params, .. } => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Function,
                        detail: Some(format!("function({})", params.join(", "))),
                    });
                }
                Stmt::Let { name, .. } | Stmt::Const { name, .. } => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Variable,
                        detail: Some("variable".to_string()),
                    });
                }
                Stmt::VarDecl(decl) => {
                    items.push(CompletionItem {
                        label: decl.name.clone(),
                        kind: CompletionKind::Variable,
                        detail: Some(if decl.is_const { "const" } else { "variable" }.to_string()),
                    });
                }
                Stmt::Decl(Decl::Agent { name, .. }) => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Module,
                        detail: Some("agent".to_string()),
                    });
                }
                Stmt::Decl(Decl::Subject { name, .. }) => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Module,
                        detail: Some("subject".to_string()),
                    });
                }
                Stmt::Class(decl) => {
                    items.push(CompletionItem {
                        label: decl.name.clone(),
                        kind: CompletionKind::Module,
                        detail: Some("class".to_string()),
                    });
                }
                Stmt::EnumDecl { name, .. } => {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: CompletionKind::Module,
                        detail: Some("enum".to_string()),
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
