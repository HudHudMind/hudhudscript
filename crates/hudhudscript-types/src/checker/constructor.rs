use super::*;
use hudhudscript_ast::Span;

impl TypeChecker {
    /// Create a new type checker (non-strict mode by default)
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            in_atomically_block: false,
            current_class: None,
            strict: false,
            generic_constraints: HashMap::new(),
            redeclare_policy: RedeclarePolicy::Error,
        }
    }

    /// BOLEM-A: Set redeclare policy for shadowing behavior.
    pub fn set_redeclare_policy(&mut self, policy: RedeclarePolicy) {
        self.redeclare_policy = policy;
    }

    /// BOLEM-A: Handle duplicate variable according to policy.
    pub(super) fn handle_duplicate(&mut self, name: &str, span: Span) -> Result<(), TypeError> {
        let diag = type_codes::duplicate_variable(name.to_string(), span);
        match self.redeclare_policy {
            RedeclarePolicy::Allow => Ok(()),
            RedeclarePolicy::Warn => { self.warnings.push(diag); Ok(()) }
            RedeclarePolicy::Error => Err(diag),
        }
    }

    /// Create a new type checker in strict mode.
    ///
    /// Issue #866 TYPE-001: In strict mode, type annotations are enforced:
    /// - `let x: number = "hello"` produces a type error
    /// - Function parameter type annotations are checked at call sites
    /// - Return type annotations are verified
    pub fn new_strict() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            in_atomically_block: false,
            current_class: None,
            strict: true,
            generic_constraints: HashMap::new(),
            redeclare_policy: RedeclarePolicy::Error,
        }
    }

    /// Whether strict mode is enabled.
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// Enable or disable strict mode.
    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Issue #1009: Register generic type parameters in the symbol table and
    /// record their constraints for enforcement during type unification.
    ///
    /// Each `GenericParam` with a constraint (e.g. `T: Number`) is recorded so
    /// that when `T` is later resolved to a concrete type, the checker can verify
    /// the concrete type satisfies the constraint.
    pub(super) fn register_generic_params(&mut self, type_params: &[GenericParam]) {
        for gp in type_params {
            // Register the generic as a type variable in the symbol table
            let _ = self
                .symbol_table
                .define(gp.name.clone(), Type::Generic(gp.name.clone()));
            // If a constraint is specified, record it
            if let Some(ref constraint) = gp.constraint {
                self.generic_constraints
                    .insert(gp.name.clone(), constraint.clone());
            }
        }
    }

    /// Issue #1009: Remove generic constraints that were added for a scope.
    pub(super) fn unregister_generic_params(&mut self, type_params: &[GenericParam]) {
        for gp in type_params {
            self.generic_constraints.remove(&gp.name);
        }
    }

    /// Issue #1009: Resolve a constraint name to a `Type`.
    ///
    /// Uses the shared `resolve_constraint_name` for built-in types (Kural 7),
    /// then falls back to symbol table lookup for class/trait names.
    #[allow(dead_code)] // Infrastructure for call-site constraint checking
    pub(super) fn resolve_constraint_type(&self, constraint: &str) -> Option<Type> {
        // Try shared built-in resolution first
        if let Some(ty) = crate::inference::resolve_constraint_name(constraint) {
            return Some(ty);
        }
        // Fall back to symbol table for class/trait names
        self.symbol_table.lookup(constraint).cloned()
    }

    /// Issue #1009: Validate that a concrete type satisfies a generic constraint.
    ///
    /// Returns `true` if the type is compatible with the constraint, `false` otherwise.
    #[allow(dead_code)] // Infrastructure for call-site constraint checking
    pub(super) fn type_satisfies_constraint(&self, ty: &Type, constraint_name: &str) -> bool {
        // Any type always satisfies any constraint (gradual typing escape hatch)
        if matches!(ty, Type::Any) {
            return true;
        }
        // Generic types (unresolved) pass — they will be checked when resolved
        if matches!(ty, Type::Generic(_)) {
            return true;
        }

        if let Some(constraint_type) = self.resolve_constraint_type(constraint_name) {
            match &constraint_type {
                Type::Any => true,
                Type::Class { name, .. } => {
                    // Instance of the class or the class itself satisfies
                    matches!(ty, Type::Instance(n) if n == name)
                        || matches!(ty, Type::Class { name: n, .. } if n == name)
                        || ty.is_compatible_with(&constraint_type)
                }
                _ => ty.is_compatible_with(&constraint_type),
            }
        } else {
            // Unknown constraint — treat as satisfied (avoid false positives)
            true
        }
    }

    /// Get collected errors
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    /// Get collected warnings (e.g. non-exhaustive match) — Issue #1018.
    pub fn warnings(&self) -> &[TypeError] {
        &self.warnings
    }

    /// Check an entire program (list of statements).
    ///
    /// Returns `Ok(())` if all statements pass type-checking, or the first
    /// error encountered.  Additionally, any non-fatal errors are accumulated
    /// in `self.errors` so callers can inspect them after a full run.
    pub fn check_program(&mut self, statements: &[Stmt]) -> Result<(), TypeError> {
        for stmt in statements {
            if let Err(e) = self.check_stmt(stmt) {
                self.errors.push(e.clone());
                return Err(e);
            }
        }
        Ok(())
    }

    /// Run type checking and produce an `AnnotatedProgram` carrying symbol
    /// information and diagnostics alongside the original AST.
    ///
    /// This is the primary entry-point for the TypeChecker → Compiler pipeline
    /// (Issue #919).  The existing `check_program` method is left unchanged.
    pub fn check_and_annotate(
        &mut self,
        stmts: Vec<Stmt>,
    ) -> Result<AnnotatedProgram, Vec<TypeError>> {
        // Run the normal type-checking pass.
        if let Err(e) = self.check_program(&stmts) {
            // `check_program` also pushes to `self.errors`, but the early-return
            // error may be the only one — make sure the vec is populated.
            if self.errors.is_empty() {
                self.errors.push(e);
            }
            return Err(self.errors.clone());
        }

        // Build the annotated program from the now-checked AST.
        let mut program = AnnotatedProgram::from_ast(stmts);

        // Transfer symbol information from the SymbolTable.
        for (name, info) in self.symbol_table.all_symbols() {
            program.symbols.insert(
                name.clone(),
                SymbolType {
                    name: name.clone(),
                    type_name: info.ty.to_string(),
                    is_mutable: info.mutable,
                    is_exported: false,
                },
            );
        }

        // Transfer any accumulated diagnostics.
        // Issue #1010: In strict mode, type mismatches are errors, not warnings.
        let level = if self.strict {
            DiagnosticLevel::Error
        } else {
            DiagnosticLevel::Warning
        };
        for err in self.errors() {
            program.diagnostics.push(AnnotationDiagnostic {
                level,
                message: err.to_string(),
                span: None,
            });
        }

        // Issue #1018: Transfer accumulated warnings (e.g. non-exhaustive match).
        for warn in &self.warnings {
            program.diagnostics.push(AnnotationDiagnostic {
                level: DiagnosticLevel::Warning,
                message: warn.to_string(),
                span: None,
            });
        }

        Ok(program)
    }
}
