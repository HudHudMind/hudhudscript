use super::*;
impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            scope_depth: 0,
            in_top_level: true,
            top_level_names: HashSet::new(),
            locals: Vec::new(),
            known_classes: HashSet::new(),
            known_generators: HashSet::new(),
            math_reassigned: false,
            inline_function_chunks: FxHashMap::default(),
            call_site_param_types: HashMap::new(),
            fn_param_names: HashMap::new(),
            current_function_name: None,
            known_traits: HashMap::new(),
            known_roles: HashMap::new(),
            declared_fns: vec![HashMap::new()],
            referenced_top_level: HashSet::new(),
            shared_top_level_names: HashSet::new(),
            pending_source_pos: None,
            local_slot_names: Vec::new(),
            fn_ctx: None,
            last_match_reg: 0,
            current_max_register: 0,
            next_local_reg: 0,
            loop_step_names: HashMap::new(),
            gate_registry: HashMap::new(),
            step_registry: HashMap::new(),
            attach_step_queue: HashMap::new(),
            attach_loop_queue: HashMap::new(),
        }
    }

    /// Record a local variable declaration at the current scope depth.
    /// PERF-B1: Assign a slot index so the VM can use slot-based fast path.
    /// Skip special compound names (__index_assign:*) — these must use the
    /// slow path for array/object index-assignment handling.
    pub(super) fn declare_local(&mut self, name: &str, is_const: bool) -> CompileResult<()> {
        if name.starts_with("__index_assign:") || name.starts_with("__") {
            // Special VM-internal names: always slow path.
            self.locals.push(Local {
                name: name.to_string(),
                depth: self.scope_depth,
                is_captured: false,
                slot: None,
                reg: None,
                is_const,
                known_type: crate::compiler::expr::ExprType::Unknown,
            });
        } else {
            let slot = self.local_slot_names.len() as u32;
            self.local_slot_names.push(name.to_string());
            let reg = self.next_local_reg;
            self.next_local_reg += 1;
            self.locals.push(Local {
                name: name.to_string(),
                depth: self.scope_depth,
                is_captured: false,
                slot: Some(slot),
                reg: Some(reg),
                is_const,
                known_type: crate::compiler::expr::ExprType::Unknown,
            });
        }
        Ok(())
    }

    /// Begin a new scope.
    pub(super) fn begin_scope(&mut self) {
        self.scope_depth += 1;
        self.declared_fns.push(HashMap::new());
    }

    /// End the current scope, removing locals declared in it.
    pub(super) fn end_scope(&mut self) {
        // FIX: Decrement scope_depth BEFORE retain so we keep locals from outer scopes
        self.scope_depth -= 1;
        self.locals.retain(|l| l.depth <= self.scope_depth);
        self.declared_fns.pop();
    }

    /// Compile AST to bytecode
    /// Compile from an annotated program (with type information from TypeChecker).
    /// The type info can be used for future optimizations.
    pub fn compile_annotated(
        &mut self,
        program: &hudhudscript_ast::annotated::AnnotatedProgram,
    ) -> CompileResult<Bytecode> {
        // For now, delegate to the existing compile method using the AST.
        // Future: use program.type_info for type-specific bytecode optimizations.
        // Future: use program.symbols for better error messages.
        self.compile(&program.stmts)
    }

    pub fn compile(&mut self, statements: &[Stmt]) -> CompileResult<Bytecode> {
        // ISSUE-2e-optimize: pre-scan AST to classify top-level names.
        self.precompute_shared_top_level(statements);

        // P4b: pure pre-pass — collect function signatures and call-site types.
        // No bytecode emitted. Enables type propagation during normal compilation.
        self.p4b_prepass_collect(statements);

        // Normal source-order compilation (all types available from pre-pass)
        for stmt in statements {
            self.compile_stmt(stmt)?;
        }
        // PERF-B1: Emit top-level local variable slot names so the VM
        // can populate sym_to_slot in execute() and use the O(1) slot
        // fast path instead of the slow HashMap scope-chain lookup.
        let mut bytecode = self.bytecode.clone();
        bytecode.main_local_names = std::mem::take(&mut self.local_slot_names);
        bytecode.main_local_count = bytecode.main_local_names.len() as u32;
        // ISSUE-2e-1: build shared/main-only bitmap.  Uses both the pre-pass
        // result and runtime tracking (fallback).
        bytecode.main_local_shared = bytecode
            .main_local_names
            .iter()
            .map(|name| {
                self.shared_top_level_names.contains(name)
                    || self.referenced_top_level.contains(name)
            })
            .collect();
        // Use the positions-aware optimizer so source_positions stays
        // parallel with instructions after any folds/drains — the VM's
        // DAP `on_statement` hook indexes by ip.
        let mut sp = std::mem::take(&mut bytecode.source_positions);
        let mut lp = std::mem::take(&mut bytecode.loop_payloads);
        let cp = std::mem::take(&mut bytecode.call_payloads);
        let ic = std::mem::take(&mut bytecode.int_constants);
        let mut sip = std::mem::take(&mut bytecode.super_instr_payloads);
        let funcs_ref = bytecode.functions.borrow();
        // Build a temporary HashMap for the optimizer (compile-time only)
        let funcs_map: std::collections::HashMap<String, std::sync::Arc<hudhudscript_bytecode::FunctionChunk>> = bytecode
            .function_names.borrow().iter()
            .map(|(name, &idx)| (name.clone(), funcs_ref[idx].clone()))
            .collect();
        crate::optimizer::optimize_with_positions(
            &mut bytecode.instructions,
            &mut bytecode.constants,
            &mut bytecode.numeric_constants,
            &ic,
            &mut lp,
            &mut sip,
            &mut sp,
            crate::optimizer::OptimizationLevel::Basic,
            Some(&funcs_map),
            &cp,
        );
        drop(funcs_ref);
        bytecode.int_constants = ic;
        bytecode.source_positions = sp;
        bytecode.loop_payloads = lp;
        bytecode.call_payloads = cp;
        bytecode.super_instr_payloads = sip;
        bytecode.pad_source_positions();

        // WI-1.2: Detect whether this bytecode needs an async runtime.
        // Check for async instructions: Await, Spawn, MakePromise, MakeGenerator.
        let needs_async = bytecode.instructions.iter().any(|instr| {
            matches!(instr,
                Instruction::Await { .. }
                | Instruction::Spawn { .. }
                | Instruction::MakeGenerator { .. }
            )
        });
        bytecode.needs_async = needs_async;
        bytecode.resolve_call_payload_function_indices();

        Ok(bytecode)
    }

    // ── ISSUE-2e-optimize pre-pass ─────────────────────────────────────

    pub(super) fn compile_stmt(&mut self, stmt: &Stmt) -> CompileResult<()> {
        compile_stmt_shared(self, stmt)
    }
}
