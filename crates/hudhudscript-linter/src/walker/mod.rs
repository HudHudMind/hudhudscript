//! AST walker — uses the `AstVisitor` trait to recursively visit every node
//! and invoke lint rules.

use crate::config::LintConfig;
use crate::diagnostic::LintDiagnostic;
use crate::rules;
use hudhudscript_ast::visitor::{self};
use hudhudscript_ast::*;
use hudhudscript_errors::{ScopeManager, SymbolInfo};
use std::collections::HashSet;

/// Accumulated state while walking the AST.
pub struct LintContext<'cfg> {
    pub config: &'cfg LintConfig,
    diagnostics: Vec<LintDiagnostic>,
    /// Variable declarations: (name, span). Collected for the unused-variable rule.
    pub var_declarations: Vec<(String, Span)>,
    /// Variable references (identifiers used in expression position).
    pub var_references: HashSet<String>,
    /// Scope stack for shadowing detection. Each entry is the set of names
    /// declared in that scope level.
    scope_stack: Vec<HashSet<String>>,
    /// Current nesting depth for the deep-nesting rule.
    depth: usize,
    /// Names declared with `const` — used by const-reassign rule.
    const_names: HashSet<String>,
    /// Names declared with `let` — (name, span) for prefer-const rule.
    let_declarations: Vec<(String, Span)>,
    /// Names that appear as assignment targets — used by prefer-const rule.
    assigned_names: HashSet<String>,
}

impl<'cfg> LintContext<'cfg> {
    pub fn new(config: &'cfg LintConfig) -> Self {
        Self {
            config,
            diagnostics: Vec::new(),
            var_declarations: Vec::new(),
            var_references: HashSet::new(),
            scope_stack: vec![HashSet::new()],
            depth: 0,
            const_names: HashSet::new(),
            let_declarations: Vec::new(),
            assigned_names: HashSet::new(),
        }
    }

    /// Declare a variable in the current scope, running shadowing + naming checks.
    pub fn declare_variable(&mut self, name: &str, span: Span) {
        // Shadowing check (before inserting into current scope)
        rules::shadowing::check(
            name,
            span,
            &self.scope_stack,
            self.config,
            &mut self.diagnostics,
        );

        // Naming convention for variables
        rules::naming_convention::check_variable_name(
            name,
            span,
            self.config,
            &mut self.diagnostics,
        );

        // Record for unused-variable analysis
        rules::unused_variables::record_decl(self, name.to_string(), span);

        // Insert into current scope
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name.to_string());
        }
    }

    /// Run the unused-variable post-walk check.
    pub fn check_unused_variables(&mut self) {
        if !self.config.is_enabled("unused-variable") {
            return;
        }
        for (name, span) in &self.var_declarations {
            if name.starts_with('_') {
                continue;
            }
            if !self.var_references.contains(name) {
                self.diagnostics.push(LintDiagnostic::new(
                    "unused-variable",
                    format!("variable `{name}` is declared but never used"),
                    self.config
                        .severity("unused-variable", crate::Severity::Warning),
                    *span,
                ));
            }
        }
    }

    /// Run the prefer-const post-walk check.
    pub fn check_prefer_const(&mut self) {
        for (name, span) in &self.let_declarations {
            if name.starts_with('_') {
                continue;
            }
            if !self.assigned_names.contains(name) {
                rules::prefer_const::report(name, *span, self.config, &mut self.diagnostics);
            }
        }
    }

    pub fn into_diagnostics(self) -> Vec<LintDiagnostic> {
        self.diagnostics
    }
}

// ── ScopeManager trait implementation ──────────────────────────────────────

impl<'cfg> ScopeManager for LintContext<'cfg> {
    fn push_scope(&mut self) {
        self.scope_stack.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn declare(&mut self, name: &str, _mutable: bool) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn resolve(&self, name: &str) -> Option<&SymbolInfo> {
        // The linter's scope stack uses HashSet<String> rather than full
        // SymbolInfo records, so rich resolution is not yet available.
        let _ = name;
        None
    }

    fn is_declared_in_current(&self, name: &str) -> bool {
        self.scope_stack
            .last()
            .map(|s| s.contains(name))
            .unwrap_or(false)
    }

    fn depth(&self) -> usize {
        self.scope_stack.len()
    }
}

// ── Public entry point ─────────────────────────────────────────────────────

pub fn walk_stmts(ctx: &mut LintContext, stmts: &[Stmt], _depth: usize) {
    visitor::walk_stmts(ctx, stmts);
}

// ── AstVisitor implementation ──────────────────────────────────────────────

mod visitor_impl;
