//! `impl CompileTarget for Compiler` split across sub-files.

use super::*;

mod class_impl;
mod emit_impl;
mod func_impl;
mod mcp_impl;

impl CompileTarget for Compiler {
    fn ct_compile_decl(&mut self, decl: &Decl) -> CompileResult<()> {
        self.compile_decl(decl)
    }
    fn ct_compile_match_pattern(
        &mut self,
        pattern: &hudhudscript_ast::MatchPattern,
    ) -> CompileResult<Vec<usize>> {
        let r = crate::compiler::regalloc::temp_reg();
        self.last_match_reg = r;
        self.bytecode.push_move(r, 255 );
        self.compile_match_pattern_bytecode(pattern, r)
    }
    fn ct_compile_destructure_pattern(
        &mut self,
        pattern: &hudhudscript_ast::Pattern,
        is_const: bool,
    ) -> CompileResult<()> {
        self.compile_destructure_pattern(pattern, is_const)
    }
    fn ct_register_trait(&mut self, name: &str, methods: &[hudhudscript_ast::TraitMethodSig]) {
        let method_names: Vec<String> = methods.iter().map(|m| m.name.clone()).collect();
        self.known_traits.insert(name.to_string(), method_names);
    }

    fn ct_compile_class(&mut self, class_decl: &hudhudscript_ast::ClassDecl) -> CompileResult<()> {
        self.compile_class_impl(class_decl)
    }
    fn ct_emit(&mut self, instr: Instruction) {
        self.emit(instr)
    }
    fn ct_emit_const(&mut self, val: Value16) -> u32 {
        self.emit_const(val)
    }
    fn ct_compile_arrow(
        &mut self,
        params: &[String],
        arrow_body: &hudhudscript_ast::ArrowFunctionBody,
        is_async: bool,
    ) -> CompileResult<()> {
        self.compile_arrow(params, arrow_body, is_async)
    }
    fn ct_compile_mcp_server(
        &mut self,
        mcp_decl: &hudhudscript_ast::McpServerDecl,
    ) -> CompileResult<()> {
        self.compile_mcp_server(mcp_decl)
    }
    fn ct_emit_num_const(&mut self, val: f64) -> u32 {
        self.emit_num_const(val)
    }
    fn ct_emit_int_const(&mut self, val: i64) -> u32 {
        self.emit_int_const(val)
    }
    /// B8: global snapshots (set before function body compilation)
    fn ct_int_constants(&self) -> &[i64] { &self.global_int_constants }
    fn ct_numeric_constants(&self) -> &[u64] { &self.global_numeric_constants }
    fn ct_intern(&mut self, name: &str) -> u32 {
        self.intern(name)
    }
    fn ct_mark_stmt_pos(&mut self, span: &Span) {
        self.mark_stmt_pos(span)
    }
    fn ct_current_ip(&self) -> usize {
        self.current_ip()
    }
    fn ct_patch(&mut self, ip: usize, instr: Instruction) {
        self.patch(ip, instr)
    }
    fn ct_push_break_target(&mut self, target: crate::compiler::target::BreakTarget) {
        self.break_targets.push(target);
    }
    fn ct_pop_break_target(&mut self) -> crate::compiler::target::BreakTarget {
        self.break_targets.pop().expect("pop without push")
    }
    fn ct_emit_break(&mut self) {
        if let Some(mut target) = self.break_targets.pop() {
            match &mut target {
                crate::compiler::target::BreakTarget::Loop => {
                    self.ct_emit(Instruction::Break);
                }
                crate::compiler::target::BreakTarget::Switch { jumps } => {
                    let ip = self.ct_current_ip();
                    self.ct_emit(Instruction::Jump(0));
                    jumps.push(ip);
                }
            }
            self.break_targets.push(target);
        } else {
            self.ct_emit(Instruction::Break);
        }
    }

    fn ct_add_loop_payload(&mut self, start: u32, end: u32) -> u32 {
        self.add_loop_payload(start, end)
    }
    fn ct_patch_loop_payload_end(&mut self, idx: u32, end: u32) {
        self.patch_loop_payload_end(idx, end)
    }
    fn ct_patch_loop_payload_start(&mut self, idx: u32, start: u32) {
        self.patch_loop_payload_start(idx, start)
    }
    fn ct_add_char_dispatch_table(&mut self, table: Vec<i16>) -> u16 {
        self.add_char_dispatch_table(table)
    }
    fn ct_replace_char_dispatch_table(&mut self, idx: u16, table: Vec<i16>) {
        self.patch_char_dispatch_table(idx, table)
    }
    fn ct_add_enum_decl_payload(&mut self, payload: hudhudscript_bytecode::EnumDeclPayload) -> u32 {
        self.add_enum_decl_payload(payload)
    }
    fn ct_add_class_decl_payload(&mut self, payload: hudhudscript_bytecode::ClassDeclPayload) -> u32 {
        self.add_class_decl_payload(payload)
    }
    fn ct_add_trait_check_payload(&mut self, payload: hudhudscript_bytecode::TraitCheckPayload) -> u32 {
        self.add_trait_check_payload(payload)
    }
    fn ct_add_load_module_payload(&mut self, payload: hudhudscript_bytecode::LoadModulePayload) -> u32 {
        let idx = self.bytecode.load_module_payloads.len() as u32;
        self.bytecode.load_module_payloads.push(payload);
        idx
    }

    fn ct_module_base_dir(&self) -> Option<&std::path::Path> {
        self.module_base_dir.as_deref()
    }
    fn ct_add_define_function_payload(&mut self, payload: hudhudscript_bytecode::DefineFunctionPayload) -> u32 {
        self.add_define_function_payload(payload)
    }
    fn ct_add_class_static_decl_payload(&mut self, payload: hudhudscript_bytecode::ClassStaticDeclPayload) -> u32 {
        self.add_class_static_decl_payload(payload)
    }
    fn ct_add_destruct_object_payload(&mut self, payload: hudhudscript_bytecode::DestructObjectPayload) -> u32 {
        self.add_destruct_object_payload(payload)
    }
    fn ct_add_call_payload(&mut self, sym: SymId, arg_count: u8) -> u32 {
        self.add_call_payload(sym, arg_count)
    }
    fn ct_add_call_payload_with_builtin(
        &mut self,
        sym: SymId,
        arg_count: u8,
        builtin_idx: u32,
    ) -> u32 {
        self.bytecode.add_call_payload_with_builtin(sym, arg_count, builtin_idx)
    }
    fn ct_add_two_sym_payload(&mut self, first: u32, second: u32) -> u32 {
        self.add_two_sym_payload(first, second)
    }
    fn ct_add_opt_sym_payload(&mut self, sym: Option<SymId>) -> u32 {
        self.add_opt_sym_payload(sym)
    }
    fn ct_add_super_instr_payload(&mut self, call_idx: u32, slot: u32, imm: i16, offset: i32) -> u32 {
        self.bytecode.add_super_instr_payload(call_idx, slot, imm, offset)
    }
    fn ct_add_cmp_jump_payload(&mut self, src1: u8, src2: u8, target: u32) -> u32 {
        self.bytecode.add_cmp_jump_payload(src1, src2, target)
    }
    fn ct_declare_local(&mut self, name: &str, is_const: bool) -> CompileResult<()> {
        // ISSUE-2e-1: top_level_names must contain only symbols declared in the
        // outermost main-chunk scope.  Function bodies reset scope_depth to 0
        // (in_top_level=false), and block-scoped locals live at depth >0, so we
        // require both flags to be true.
        if self.in_top_level && self.scope_depth == 0 {
            self.top_level_names.insert(name.to_string());
        }
        self.declare_local(name, is_const)
    }
    fn ct_is_top_level(&self) -> bool {
        self.in_top_level
    }
    fn ct_is_shared_top_level(&self, name: &str) -> bool {
        self.shared_top_level_names.contains(name)
    }
    fn ct_begin_scope(&mut self) {
        self.begin_scope()
    }
    fn ct_end_scope(&mut self) {
        self.end_scope()
    }
    fn ct_compile_function_decl(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Stmt],
        is_async: bool,
        is_generator: bool,
        span: Span,
    ) -> CompileResult<()> {
        self.compile_function_decl(name, params, body, is_async, is_generator, span)
    }

    fn ct_local_type(&self, name: &str) -> crate::compiler::expr::ExprType {
        let local_ty = self.locals
            .iter()
            .rev()
            .find(|l| l.name == name)
            .map(|l| l.known_type)
            .unwrap_or(crate::compiler::expr::ExprType::Unknown);
        // P4b: if local type is Unknown, try call-site parameter types
        if local_ty == crate::compiler::expr::ExprType::Unknown {
            if let Some(ref fn_name) = self.current_function_name {
                if let Some(types) = self.call_site_param_types.get(fn_name) {
                    for (pname, pty) in types {
                        if pname == name {
                            return *pty;
                        }
                    }
                }
            }
        }
        local_ty
    }

    fn ct_set_local_type(&mut self, name: &str, ty: crate::compiler::expr::ExprType) {
        if let Some(local) = self.locals.iter_mut().rev().find(|l| l.name == name) {
            local.known_type = ty;
        }
    }

    fn ct_local_reg(&self, name: &str) -> Option<u8> {
        self.locals
            .iter()
            .rev()
            .find(|l| l.name == name)
            .and_then(|l| l.reg)
    }
    fn ct_is_const_local(&self, name: &str) -> bool {
        self.locals
            .iter()
            .rev()
            .find(|l| l.name == name)
            .map(|l| l.is_const)
            .unwrap_or(false)
    }

    fn ct_is_captured_local(&self, name: &str) -> bool {
        self.locals
            .iter()
            .rev()
            .find(|l| l.name == name)
            .map(|l| l.is_captured)
            .unwrap_or(false)
    }

    fn ct_next_local_reg(&self) -> u8 {
        self.next_local_reg
    }

    fn ct_current_fn_name(&self) -> Option<&str> {
        self.fn_ctx.as_ref().and_then(|c| c.fn_name.as_deref())
    }

    fn ct_current_has_rest(&self) -> bool {
        self.fn_ctx.as_ref().map_or(false, |c| c.has_rest)
    }

    fn ct_check_await(&self, span: &Span) -> CompileResult<()> {
        if let Some(ctx) = &self.fn_ctx {
            if !ctx.is_async {
                let pos = SourcePosition { line: span.start.line, column: span.start.column };
                return Err(compile_codes::generic_at(
                    "await can only be used inside async functions".to_string(), pos));
            }
        }
        Ok(())
    }

    fn ct_track_reference(&mut self, name: &str) {
        if let Some(ctx) = &mut self.fn_ctx {
            ctx.referenced.push(name.to_string());
            // ISSUE-2e-1: only references inside a function/closure body can make
            // a top-level symbol "shared".  Top-level code referencing its own
            // symbols must not mark them shared.
            if self.top_level_names.contains(name) {
                self.referenced_top_level.insert(name.to_string());
            }
        }
    }

    fn ct_match_reg(&self) -> u8 {
        self.last_match_reg
    }
    fn ct_set_match_reg(&mut self, reg: u8) {
        self.last_match_reg = reg;
    }
    fn ct_patch_jump_offset(&mut self, ip: usize, offset: i16) {
        match &mut self.bytecode.instructions[ip] {
            Instruction::JumpIfFalse { offset: ref mut off, .. } => *off = offset,
            Instruction::JumpIfTrue { offset: ref mut off, .. } => *off = offset,
            Instruction::Jump(ref mut off) => *off = offset as i32,
            _ => {}
        }
    }
    // ── G12: f-loop bağlamı ──────────────────────────────────────────────
    fn ct_floop_push(
        &mut self,
        slots: Vec<(String, u8)>,
        consts: Vec<(u64, u8)>,
        temp_base: u8,
    ) {
        self.floop_stack.push(crate::compiler::FloopCtx {
            slots: slots.into_iter().collect(),
            consts: consts.into_iter().collect(),
            temp_next: temp_base,
            temp_base,
        });
    }
    fn ct_floop_pop(&mut self) {
        self.floop_stack.pop();
    }
    fn ct_floop_slot(&self, name: &str) -> Option<u8> {
        self.floop_stack.last()?.slots.get(name).copied()
    }
    fn ct_floop_const_slot(&self, bits: u64) -> Option<u8> {
        self.floop_stack.last()?.consts.get(&bits).copied()
    }
    fn ct_floop_temp(&mut self) -> Option<u8> {
        let ctx = self.floop_stack.last_mut()?;
        if ctx.temp_next >= 64 {
            return None;
        }
        let t = ctx.temp_next;
        ctx.temp_next += 1;
        Some(t)
    }
    fn ct_floop_temp_pop(&mut self) {
        if let Some(ctx) = self.floop_stack.last_mut() {
            if ctx.temp_next > ctx.temp_base {
                ctx.temp_next -= 1;
            }
        }
    }
    fn ct_floop_captured(&self, name: &str) -> bool {
        let local_captured = self.locals
            .iter()
            .rev()
            .find(|l| l.name == name)
            .map(|l| l.is_captured)
            .unwrap_or(false);
        local_captured
            || self
                .fn_ctx
                .as_ref()
                .map(|c| c.nested_captured.contains(name))
                .unwrap_or(false)
    }
    fn ct_is_known_generator(&self, name: &str) -> bool {
        self.known_generators.contains(name)
    }
    fn ct_math_global_written(&self) -> bool {
        // P5d: check if Math was reassigned in this compilation unit
        self.math_reassigned
    }
    fn ct_set_math_reassigned(&mut self) {
        self.math_reassigned = true;
    }
    fn ct_get_function_chunk(&self, name: &str) -> Option<Arc<FunctionChunk>> {
        // P3a: use inline registry (populated at declaration time, independent of RefCell)
        self.inline_function_chunks.get(name).cloned()
    }
    fn ct_record_call_site_types(&mut self, fn_name: &str, args: &[(String, crate::compiler::expr::ExprType)]) {
        // P4b: merge with existing types.
        // Rule: Unknown always wins. Conflicting known types → Unknown.
        // Only when ALL calls have the same known type do we keep it.
        // Any later Unknown or conflicting call degrades to Unknown.
        if let Some(existing) = self.call_site_param_types.get_mut(fn_name) {
            for (i, (pname, pty)) in args.iter().enumerate() {
                if let Some((_, existing_ty)) = existing.get_mut(i) {
                    if *pty == crate::compiler::expr::ExprType::Unknown
                        || *existing_ty == crate::compiler::expr::ExprType::Unknown
                    {
                        *existing_ty = crate::compiler::expr::ExprType::Unknown;
                    } else if *pty != *existing_ty {
                        *existing_ty = crate::compiler::expr::ExprType::Unknown;
                    }
                }
            }
        } else {
            // First call: store as-is (even if Unknown — will be overridden by later calls)
            self.call_site_param_types.insert(fn_name.to_string(), args.to_vec());
        }
    }
    fn ct_get_fn_param_names(&self, fn_name: &str) -> Option<Vec<String>> {
        self.fn_param_names.get(fn_name).cloned()
    }
}
