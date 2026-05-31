use super::*;
impl Compiler {
    /// If the expression is an arrow function, compile it as a FunctionChunk
    /// and return a Function value. Otherwise, convert to a const value.
    pub(super) fn compile_session_hook(
        &mut self,
        decl_name: &str,
        hook_name: &str,
        hook_expr: &Expr,
    ) -> CompileResult<Value16> {
        match hook_expr {
            Expr::ArrowFunction {
                params,
                body,
                is_async,
                ..
            } => {
                use hudhudscript_ast::ArrowFunctionBody;
                let chunk_name = format!("{}::session::{}", decl_name, hook_name);
                let param_names: Vec<String> = params.clone();
                let stmts = match body {
                    ArrowFunctionBody::Block(stmts) => stmts.clone(),
                    ArrowFunctionBody::Expression(expr) => {
                        vec![hudhudscript_ast::Stmt::Return {
                            value: Some(*expr.clone()),
                            span: hudhudscript_ast::Span::default(),
                        }]
                    }
                };
                let chunk =
                    self.compile_function_body_async(param_names.clone(), &stmts, *is_async)?;
                self.bytecode
                    .functions
                    .borrow_mut()
                    .insert(chunk_name.clone(), Arc::new(chunk));
                Ok(Value16::function(FunctionData {
                    name: chunk_name.clone(),
                    params: param_names,
                    chunk_name,
                    captures: Default::default(),
                }))
            }
            _ => Ok(self.expr_to_const_value(hook_expr)),
        }
    }

    /// Compile a function body (a list of statements) into a FunctionChunk.
    pub(super) fn compile_function_body(
        &mut self,
        params: Vec<String>,
        body: &[Stmt],
    ) -> CompileResult<FunctionChunk> {
        self.compile_function_body_async(params, body, false)
    }

    /// Compile a function body with async flag.
    pub(super) fn compile_function_body_async(
        &mut self,
        params: Vec<String>,
        body: &[Stmt],
        is_async: bool,
    ) -> CompileResult<FunctionChunk> {
        self.compile_function_body_named_async(params, None, body, is_async)
    }

    /// CROSS-1 (TCO): Compile a function body tagged with the
    /// function's name so self-tail-call detection can emit
    /// `TailCall` in the shared `Stmt::Return` path.
    ///
    /// Uses Compiler directly — saves/restores bytecode state instead of
    /// creating a separate FunctionCompiler (eliminates code d'état).
    pub(super) fn compile_function_body_named_async(
        &mut self,
        params: Vec<String>,
        fn_name: Option<String>,
        body: &[Stmt],
        is_async: bool,
    ) -> CompileResult<FunctionChunk> {
        // ── Save outer compiler state ──────────────────────────────────
        let saved_bytecode = std::mem::replace(&mut self.bytecode, Bytecode::default());
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_slot_names = std::mem::take(&mut self.local_slot_names);
        let saved_scope_depth = self.scope_depth;
        self.scope_depth = 0;
        let saved_declared_fns = std::mem::replace(&mut self.declared_fns, vec![HashMap::new()]);
        let saved_in_top_level = self.in_top_level;
        self.in_top_level = false;
        let saved_max_register = self.current_max_register;
        let saved_next_local_reg = self.next_local_reg;
        self.current_max_register = 0;
        crate::compiler::regalloc::reset_temp_reg();
        crate::compiler::regalloc::reset_base();

        // ── Set function context for trait method overrides ───────────
        let mut saved_fn_ctx = self.fn_ctx.take();
        self.fn_ctx = Some(FuncCtx {
            params: params.clone(),
            fn_name: fn_name.clone(),
            has_rest: params.last().map(|p| p.starts_with("...")).unwrap_or(false),
            referenced: Vec::new(),
            is_async,
        });

        // ── Set up function-scope locals from parameters ───────────────
        let has_rest_param = params.last().map(|p| p.starts_with("...")).unwrap_or(false);
        let mut local_slot_names = Vec::new();
        let mut locals = Vec::new();
        for (i, p) in params.iter().enumerate() {
            let slot_name = p
                .strip_prefix("...")
                .map(|s| s.to_string())
                .unwrap_or_else(|| p.clone());
            local_slot_names.push(slot_name.clone());
            locals.push(Local {
                name: slot_name,
                depth: 0,
                is_captured: false,
                slot: Some(i as u32),
                reg: Some(i as u8),
                is_const: false,
                known_type: crate::compiler::expr::ExprType::Unknown,
            });
        }
        self.locals = locals;
        self.local_slot_names = local_slot_names;
        self.next_local_reg = params.len() as u8;

        // ── Compile function body ──────────────────────────────────────
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // ── Implicit return for void functions ─────────────────────────
        if !matches!(self.bytecode.instructions.last(), Some(Instruction::Return { .. })) {
            let null_idx = self.bytecode.constants.len() as u32;
            self.bytecode.constants.push(Value16::null());
            let tr = crate::compiler::regalloc::temp_reg();
            self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: null_idx as u16 });
            self.bytecode.push_instr(Instruction::Return { src: tr });
        }

        // ── Run optimizer passes on function-local pools ───────────────
        let mut sp = std::mem::take(&mut self.bytecode.source_positions);
        while sp.len() < self.bytecode.instructions.len() {
            sp.push(None);
        }
        sp.truncate(self.bytecode.instructions.len());
        // Fuse_slot_immediate skipped for function bodies — relative jumps
        // in loops break when instructions are removed. Top-level only.
        crate::optimizer::fuse_super_instructions_with_positions(
            &mut self.bytecode.instructions,
            &mut self.bytecode.loop_payloads,
            &self.bytecode.call_payloads,
            &mut self.bytecode.super_instr_payloads,
            &mut sp,
        );

        // ── Extract function-local data ────────────────────────────────
        let func_instructions = std::mem::take(&mut self.bytecode.instructions);
        let func_constants = std::mem::take(&mut self.bytecode.constants);
        let func_numeric = std::mem::take(&mut self.bytecode.numeric_constants);
        let func_int = std::mem::take(&mut self.bytecode.int_constants);
        let func_loop = std::mem::take(&mut self.bytecode.loop_payloads);
        let mut func_enum = std::mem::take(&mut self.bytecode.enum_decl_payloads);
        let mut func_class = std::mem::take(&mut self.bytecode.class_decl_payloads);
        let mut func_traitck = std::mem::take(&mut self.bytecode.trait_check_payloads);
        let mut func_loadmod = std::mem::take(&mut self.bytecode.load_module_payloads);
        let mut func_deffn = std::mem::take(&mut self.bytecode.define_function_payloads);
        let mut func_class_static = std::mem::take(&mut self.bytecode.class_static_decl_payloads);
        let mut func_destruct = std::mem::take(&mut self.bytecode.destruct_object_payloads);
        let mut func_call_payloads = std::mem::take(&mut self.bytecode.call_payloads);
        let mut func_two_sym = std::mem::take(&mut self.bytecode.two_sym_payloads);
        let mut func_opt_sym = std::mem::take(&mut self.bytecode.opt_sym_payloads);
        let mut func_super = std::mem::take(&mut self.bytecode.super_instr_payloads);
        let mut func_nested = std::mem::take(&mut *self.bytecode.functions.borrow_mut());

        // ── Restore outer bytecode ─────────────────────────────────────
        let mut outer_bytecode = saved_bytecode;
        let mut func_instructions = func_instructions;

        // Merge numeric constants: remap indices, add to parent pool
        if !func_numeric.is_empty() {
            let mut index_map: Vec<u32> = Vec::with_capacity(func_numeric.len());
            for bits in &func_numeric {
                let new_idx = outer_bytecode.add_numeric_constant_bits(*bits);
                index_map.push(new_idx);
            }
            for instr in &mut func_instructions {
                if let Instruction::LoadNumConst { const_idx, .. } = instr {
                    *const_idx = index_map[*const_idx as usize] as u16;
                }
            }
            for chunk in func_nested.values_mut() {
                if let Some(c) = Arc::get_mut(chunk) {
                    for instr in &mut c.instructions {
                        if let Instruction::LoadNumConst { const_idx, .. } = instr {
                            if let Some(&new_idx) = index_map.get(*const_idx as usize) {
                                *const_idx = new_idx as u16;
                            }
                        }
                    }
                }
            }
        }

        // Merge int constants
        if !func_int.is_empty() {
            let mut index_map: Vec<u32> = Vec::with_capacity(func_int.len());
            for v in &func_int {
                let new_idx = outer_bytecode.add_int_constant(*v);
                index_map.push(new_idx);
            }
            for instr in &mut func_instructions {
                if let Instruction::LoadIntConst { const_idx, .. } = instr {
                    *const_idx = index_map[*const_idx as usize] as u16;
                }
            }
            for chunk in func_nested.values_mut() {
                if let Some(c) = Arc::get_mut(chunk) {
                    for instr in &mut c.instructions {
                        if let Instruction::LoadIntConst { const_idx, .. } = instr {
                            if let Some(&new_idx) = index_map.get(*const_idx as usize) {
                                *const_idx = new_idx as u16;
                            }
                        }
                    }
                }
            }
        }

        // Merge loop payloads
        if !func_loop.is_empty() {
            let base = outer_bytecode.loop_payloads.len() as u32;
            outer_bytecode.loop_payloads.extend(func_loop.iter().copied());
            for instr in &mut func_instructions {
                if let Instruction::LoopBegin(idx) = instr {
                    *idx += base;
                }
            }
        }

        // Merge all declaration payload pools
        macro_rules! merge_payload_pool {
            ($outer:expr, $func:expr, $field:ident, $($pat:pat => $idx:ident),+ $(,)?) => {
                if !$func.is_empty() {
                    let base = $outer.$field.len() as u32;
                    $outer.$field.extend($func.drain(..));
                    for instr in &mut func_instructions {
                        match instr {
                            $($pat => { *$idx += base; })+
                            _ => {}
                        }
                    }
                }
            };
        }

        merge_payload_pool!(outer_bytecode, func_enum, enum_decl_payloads,
            Instruction::EnumDecl(idx) => idx);
        merge_payload_pool!(outer_bytecode, func_class, class_decl_payloads,
            Instruction::ClassDecl(idx) => idx);
        merge_payload_pool!(outer_bytecode, func_traitck, trait_check_payloads,
            Instruction::TraitCheck(idx) => idx);
        merge_payload_pool!(outer_bytecode, func_loadmod, load_module_payloads,
            Instruction::LoadModule(idx) => idx);
        merge_payload_pool!(outer_bytecode, func_deffn, define_function_payloads,
            Instruction::DefineFunction(idx) => idx);
        merge_payload_pool!(outer_bytecode, func_class_static, class_static_decl_payloads,
            Instruction::ClassStaticDecl(idx) => idx);
        merge_payload_pool!(outer_bytecode, func_destruct, destruct_object_payloads,
            Instruction::DestructObject(idx) => idx);

        // Merge call payloads
        if !func_call_payloads.is_empty() {
            let base = outer_bytecode.call_payloads.len() as u16;
            for sp in func_super.iter_mut() {
                sp.call_idx += base as u32;
            }
            outer_bytecode.call_payloads.extend(func_call_payloads.drain(..));
            for instr in &mut func_instructions {
                if let Instruction::Call { payload_idx: idx, .. }
                    | Instruction::NewInstance { payload_idx: idx, .. }
                    | Instruction::MakeGenerator { payload_idx: idx, .. }
                    | Instruction::MethodCall { payload_idx: idx, .. }
                    | Instruction::SuperCall { payload_idx: idx, .. } = instr
                {
                    *idx += base;
                }
            }
            for chunk in func_nested.values_mut() {
                if let Some(c) = Arc::get_mut(chunk) {
                    for instr in &mut c.instructions {
                        if let Instruction::Call { payload_idx: idx, .. }
                            | Instruction::NewInstance { payload_idx: idx, .. }
                            | Instruction::MakeGenerator { payload_idx: idx, .. }
                            | Instruction::MethodCall { payload_idx: idx, .. }
                            | Instruction::SuperCall { payload_idx: idx, .. } = instr
                        {
                            *idx += base;
                        }
                    }
                }
            }
        }

        // Merge super instruction payloads
        if !func_super.is_empty() {
            let base = outer_bytecode.super_instr_payloads.len() as u32;
            outer_bytecode.super_instr_payloads.extend(func_super.drain(..));
            for instr in &mut func_instructions {
                match instr {
                    Instruction::IntSubCall1(idx) => *idx += base,
                    Instruction::IntAddCall1(idx) => *idx += base,
                    Instruction::IntLeJumpIfFalse(idx) => *idx += base,
                    Instruction::IntLtJumpIfFalse(idx) => *idx += base,
                    Instruction::IntSubLocalI { payload_idx, .. } => *payload_idx += base as u16,
                    Instruction::IntAddLocalI { payload_idx, .. } => *payload_idx += base as u16,
                    _ => {}
                }
            }
        }

        // Merge two-sym payloads (MatchVariant, GetStatic: u32; DeclStore: u16 — same table)
        if !func_two_sym.is_empty() {
            let base = outer_bytecode.two_sym_payloads.len();
            outer_bytecode.two_sym_payloads.extend(func_two_sym.drain(..));
            for instr in &mut func_instructions {
                match instr {
                    Instruction::MatchVariant(idx) => *idx += base as u32,
                    Instruction::GetStatic(idx) => *idx += base as u32,
                    Instruction::DeclStore { payload_idx, .. } => *payload_idx += base as u16,
                    _ => {}
                }
            }
        }

        // Merge opt-sym payloads (base BEFORE extend)
        if !func_opt_sym.is_empty() {
            let base = outer_bytecode.opt_sym_payloads.len() as u16;
            outer_bytecode.opt_sym_payloads.extend(func_opt_sym.drain(..));
            for instr in &mut func_instructions {
                match instr {
                    Instruction::Remember { store_idx, .. } => *store_idx += base,
                    Instruction::Recall { store_idx, .. } => *store_idx += base,
                    Instruction::Forget { store_idx, .. } => *store_idx += base,
                    _ => {}
                }
            }
        }

        // ── Register nested chunks in outer bytecode ───────────────────
        for (chunk_name, chunk) in func_nested {
            outer_bytecode.functions.borrow_mut().insert(chunk_name, chunk);
        }

        // ── Resolve captures ───────────────────────────────────────────
        let named_params: Vec<String> = params.iter().map(|p| {
            p.strip_prefix("...").map(|s| s.to_string()).unwrap_or_else(|| p.clone())
        }).collect();
        let referenced = self.fn_ctx.as_ref().map(|c| c.referenced.clone()).unwrap_or_default();
        let mut captures = Vec::new();
        let mut seen = HashSet::new();
        for name in &referenced {
            if !named_params.contains(name)
                && !self.top_level_names.contains(name)
                && !seen.contains(name)
            {
                seen.insert(name.clone());
                captures.push(name.clone());
            }
        }

        // ── Capture local slot info BEFORE restoring outer state ──────
        let func_local_names = std::mem::take(&mut self.local_slot_names);
        let func_local_count = func_local_names.len() as u32;

        // ── Propagate captures to parent fn_ctx for nested closures ──
        if let Some(outer_fn_ctx) = &mut saved_fn_ctx {
            for cap in &captures {
                outer_fn_ctx.referenced.push(cap.clone());
            }
        }

        // ── Restore outer compiler state ───────────────────────────────
        self.fn_ctx = saved_fn_ctx;
        self.bytecode = outer_bytecode;
        self.locals = saved_locals;
        self.local_slot_names = saved_slot_names;
        self.scope_depth = saved_scope_depth;
        self.declared_fns = saved_declared_fns;
        self.in_top_level = saved_in_top_level;
        let max_register = self.current_max_register;
        self.current_max_register = saved_max_register;
        self.next_local_reg = saved_next_local_reg;

        // ── Build param_slots: param index → local slot index ─────────
        let param_slots: Vec<u16> = named_params.iter().map(|name| {
            func_local_names.iter().position(|n| n == name).unwrap_or(0) as u16
        }).collect();

        // ── Normalize source_positions ─────────────────────────────────
        while sp.len() < func_instructions.len() {
            sp.push(None);
        }
        sp.truncate(func_instructions.len());

        Ok(FunctionChunk {
            params: params.clone(),
            instructions: func_instructions,
            constants: func_constants,
            captures,
            is_async,
            is_generator: false,
            local_count: func_local_count,
            local_names: func_local_names,
            capture_cells: vec![],
            max_register,
            sym_to_slot: std::sync::OnceLock::new(),
            source_positions: sp,
            param_slots: param_slots.into_boxed_slice(),
        })
    }
}
