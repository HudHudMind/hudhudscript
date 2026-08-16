use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreakTarget {
    Loop,
    Switch { jumps: Vec<usize> },
}

pub trait CompileTarget {
    // ── Instruction emission ─────────────────────────────────────────────
    /// Append an instruction to the output stream.
    ///
    /// Implementations MUST also extend `source_positions` by one entry
    /// (defaulting to `None`) so the parallel-vector invariant holds.
    /// Statement-boundary position info is then overwritten by
    /// [`ct_mark_stmt_pos`].
    fn ct_emit(&mut self, instr: Instruction);

    /// G5.2: emit a Move instruction, skipping self-moves (dst == src).
    /// All call sites that emit Move should use this instead of raw ct_emit.
    fn emit_move(&mut self, dst: u8, src: u8) {
        if dst != src {
            self.ct_emit(Instruction::Move { dst, src });
        }
    }
    /// Add a constant to the pool and return its `u32` index (CROSS-2b).
    fn ct_emit_const(&mut self, val: Value16) -> u32;
    /// Add a numeric constant to the packed pool and return its `u32` index.
    /// Uses NaN-boxed storage (8 bytes instead of 176-byte Value).
    fn ct_emit_num_const(&mut self, val: f64) -> u32;
    /// A3b: add an integer constant to the `int_constants` pool and return
    /// its `u32` index (for `Instruction::LoadIntConst`).  Deduplicates
    /// exact `i64` matches.
    fn ct_emit_int_const(&mut self, val: i64) -> u32;
    /// B8: global int/numeric constant snapshots (for inliner remap).
    fn ct_int_constants(&self) -> &[i64] {
        &[]
    }
    fn ct_numeric_constants(&self) -> &[u64] {
        &[]
    }
    /// Intern a symbol name and return its index (Issue #1032 P1).
    /// Used for variable/property names in LoadVar, StoreVar, DeclVar, etc.
    fn ct_intern(&mut self, name: &str) -> u32;

    /// Record that the next-emitted instruction begins the statement at
    /// `span`.  Used by the DAP debugger's `on_statement` hook to map
    /// VM instruction pointers back to source file/line positions.
    ///
    /// Implementations set `source_positions[current_ip]` to
    /// `Some((line, column))` (padding earlier entries with `None` to
    /// stay in lockstep with `instructions`).
    fn ct_mark_stmt_pos(&mut self, span: &Span);

    // ── Interned instruction helpers (avoid double &mut self borrow) ────

    /// Intern a name and return a compact `SymId` for use in instructions.
    fn ct_sym(&mut self, name: &str) -> SymId {
        SymId(self.ct_intern(name))
    }

    fn ct_emit_load_var(&mut self, name: &str) {
        self.ct_emit_load_var_to(name, 255)
    }
    fn ct_emit_load_var_to(&mut self, name: &str, dst: u8) {
        if let Some(reg) = self.ct_local_reg(name) {
            self.ct_emit(Instruction::Move { dst, src: reg });
        } else {
            let sym = self.ct_intern(name);
            self.ct_emit(Instruction::LoadGlobal {
                dst,
                sym: sym as u16,
            });
        }
    }
    fn ct_emit_store_var(&mut self, name: &str) {
        self.ct_emit_store_var_from(name, 255)
    }
    fn ct_emit_store_var_from(&mut self, name: &str, src: u8) {
        if let Some(reg) = self.ct_local_reg(name) {
            self.ct_emit(Instruction::Move { dst: reg, src });
        } else {
            let sym = self.ct_intern(name);
            self.ct_emit(Instruction::StoreGlobal {
                src,
                sym: sym as u16,
            });
        }
    }

    /// Return the current instruction pointer (= instruction count so far).
    fn ct_current_ip(&self) -> usize;
    /// Back-patch an instruction at the given position.
    fn ct_patch(&mut self, ip: usize, instr: Instruction);

    /// Push a break target context (Loop or Switch).
    fn ct_push_break_target(&mut self, target: BreakTarget);
    /// Pop a break target context.
    fn ct_pop_break_target(&mut self) -> BreakTarget;
    /// Emit a break instruction or a jump depending on the nearest target.
    fn ct_emit_break(&mut self);

    /// Register a loop header payload (CROSS-2b) and return the `u32`
    /// index for `Instruction::LoopBegin(idx)`.  The compiler emits
    /// `LoopBegin` with the returned index; `ct_patch_loop_payload_end`
    /// backfills the exit IP once the loop body is compiled.
    fn ct_add_loop_payload(&mut self, start: u32, end: u32) -> u32;

    /// Back-fill the `end` field of a previously added loop payload.
    /// Used by While/C-style loops that know the exit IP only after
    /// the body has been emitted.
    fn ct_patch_loop_payload_end(&mut self, idx: u32, end: u32);

    /// C3: Back-fill the `start` field of a loop payload. Used by ForCStyle
    /// loops where `continue` must jump past the body to the update clause.
    fn ct_patch_loop_payload_start(&mut self, idx: u32, start: u32);

    /// C6: Register a 256-entry char-dispatch jump table and return its index.
    fn ct_add_char_dispatch_table(&mut self, table: Vec<i16>) -> u16;
    /// C6: Replace a previously registered char-dispatch jump table.
    fn ct_replace_char_dispatch_table(&mut self, idx: u16, table: Vec<i16>);

    // ── CROSS-2a: externalised declaration-payload registration.
    // Instead of `Instruction::ClassDecl(Box<...>)` etc., the compiler
    // registers the payload in the appropriate side-table pool and
    // emits `Instruction::ClassDecl(u32)` with the returned index.
    /// Register an `EnumDecl` payload and return its pool index.
    fn ct_add_enum_decl_payload(&mut self, payload: hudhudscript_bytecode::EnumDeclPayload) -> u32;
    /// Register a `ClassDecl` payload and return its pool index.
    ///
    /// Currently only `Compiler` emits `ClassDecl` (at the top level);
    /// FunctionCompiler's `ct_compile_class` lowers classes to a plain
    /// `LoadConst + StoreVar` string marker instead.  The trait method
    /// exists so a future expansion that emits ClassDecl inside
    /// function bodies does not need to rediscover the pool plumbing.
    #[allow(dead_code)]
    fn ct_add_class_decl_payload(
        &mut self,
        payload: hudhudscript_bytecode::ClassDeclPayload,
    ) -> u32;
    /// Register a `TraitCheck` payload and return its pool index.
    /// Top-level-only today; kept on the trait for future function-body
    /// class emission (same rationale as `ct_add_class_decl_payload`).
    #[allow(dead_code)]
    fn ct_add_trait_check_payload(
        &mut self,
        payload: hudhudscript_bytecode::TraitCheckPayload,
    ) -> u32;
    /// Register a `LoadModule` payload and return its pool index.
    fn ct_add_load_module_payload(
        &mut self,
        payload: hudhudscript_bytecode::LoadModulePayload,
    ) -> u32;
    fn ct_module_base_dir(&self) -> Option<&std::path::Path>;
    /// Register a `DefineFunction` payload and return its pool index.
    /// Currently unemitted by the compiler (the explicit `DefineFunction`
    /// VM instruction is reserved for a later closure-hoisting pass);
    /// the pool + trait method exist so the plumbing survives future use.
    #[allow(dead_code)]
    fn ct_add_define_function_payload(
        &mut self,
        payload: hudhudscript_bytecode::DefineFunctionPayload,
    ) -> u32;
    /// Register a `ClassStaticDecl` payload and return its pool index.
    /// Top-level-only today; see `ct_add_class_decl_payload`.
    #[allow(dead_code)]
    fn ct_add_class_static_decl_payload(
        &mut self,
        payload: hudhudscript_bytecode::ClassStaticDeclPayload,
    ) -> u32;
    /// Register a `DestructObject` payload and return its pool index.
    /// The shared `compile_destructure_pattern` helpers push directly to
    /// each compiler's local pool (`self.bytecode.add_destruct_object_payload`
    /// vs `self.destruct_object_payloads.push`) rather than dispatching
    /// through the trait, so this method is unused today but kept for
    /// any future call site that has only a `&mut dyn CompileTarget`.
    #[allow(dead_code)]
    fn ct_add_destruct_object_payload(
        &mut self,
        payload: hudhudscript_bytecode::DestructObjectPayload,
    ) -> u32;

    // ── CROSS-2c+d: three more side-table pools backing the final 14
    // variants externalised on the path from 12 B → 8 B
    // `Instruction`.  Used by call/method/spawn sites, two-symbol
    // instructions, and the RAG Option<SymId> variants.
    /// Register a call-family payload and return its pool index (CROSS-2c).
    fn ct_add_call_payload(&mut self, sym: SymId, arg_count: u8) -> u32;
    /// P6: register a call payload with a known builtin method ID for fast dispatch.
    /// Default implementation falls back to regular ct_add_call_payload.
    fn ct_add_call_payload_with_builtin(
        &mut self,
        sym: SymId,
        arg_count: u8,
        _builtin_idx: u32,
    ) -> u32 {
        self.ct_add_call_payload(sym, arg_count)
    }
    /// Register a two-symbol payload and return its pool index (CROSS-2d).
    /// `first` is a raw `u32` (either a symbol-table index from
    /// `ct_intern` or a `SymId.0`); `second` is stored as `u32` the same
    /// way — the shared struct stays type-agnostic.
    fn ct_add_two_sym_payload(&mut self, first: u32, second: u32) -> u32;
    /// Register an optional-symbol payload and return its pool index
    /// (CROSS-2d).
    fn ct_add_opt_sym_payload(&mut self, sym: Option<SymId>) -> u32;
    fn ct_add_super_instr_payload(
        &mut self,
        call_idx: u32,
        slot: u32,
        imm: i16,
        offset: i32,
    ) -> u32;
    /// Add a packed compare+jump payload (GÖREV 5) and return its index.
    fn ct_add_cmp_jump_payload(&mut self, src1: u8, src2: u8, target: u32) -> u32;

    // ── Expression helpers ───────────────────────────────────────────────
    /// Track an identifier reference (FunctionCompiler records captures).
    fn ct_track_reference(&mut self, _name: &str) {}
    /// Check whether `name` is a known class (for GetStatic emission).
    fn ct_is_known_class(&self, _name: &str) -> bool {
        false
    }
    /// Check whether `name` is a known generator function.
    fn ct_is_known_generator(&self, _name: &str) -> bool {
        false
    }
    /// P5c: true when the Math global has been written to (shadow detection).
    fn ct_math_global_written(&self) -> bool {
        false
    }
    /// P5d: mark that Math global has been reassigned in this compilation unit.
    fn ct_set_math_reassigned(&mut self) {}
    /// P3: get a previously-compiled function chunk for inlining analysis.
    fn ct_get_function_chunk(&self, _name: &str) -> Option<Arc<FunctionChunk>> {
        None
    }
    /// P4: record call-site argument types for a function.
    /// Called when compiling a Call expression with known argument types.
    fn ct_record_call_site_types(
        &mut self,
        _fn_name: &str,
        _args: &[(String, crate::compiler::expr::ExprType)],
    ) {
    }
    /// P4a: get pre-declared parameter names for a function.
    fn ct_get_fn_param_names(&self, _fn_name: &str) -> Option<Vec<String>> {
        None
    }
    /// Validate that `await` is allowed in the current context.
    /// Returns `Ok(())` if allowed, or an error otherwise.
    fn ct_check_await(&self, _span: &Span) -> CompileResult<()> {
        Ok(())
    }

    /// Compile an arrow-function expression. The implementations diverge
    /// significantly between Compiler and FunctionCompiler, so each provides
    /// its own version.
    fn ct_compile_arrow(
        &mut self,
        params: &[String],
        arrow_body: &hudhudscript_ast::ArrowFunctionBody,
        is_async: bool,
    ) -> CompileResult<()>;

    // ── Statement helpers ────────────────────────────────────────────────
    /// Declare a local variable by name.
    fn ct_declare_local(&mut self, name: &str, is_const: bool) -> CompileResult<()>;
    /// True when compiling the outermost scope (not inside a function body).
    fn ct_is_top_level(&self) -> bool;
    /// True when this top-level symbol is referenced from a function/closure
    /// body.  Used by the compiler to decide whether to emit StoreGlobal/
    /// DeclGlobal or stay pure register for main-only symbols.
    fn ct_is_shared_top_level(&self, name: &str) -> bool {
        false
    }
    /// Begin a new scope (emits PushScope and tracks depth).
    fn ct_begin_scope(&mut self);
    /// End the current scope (emits PopScope and cleans up locals).
    fn ct_end_scope(&mut self);
    /// Current lexical scope depth.
    fn ct_scope_depth(&self) -> usize {
        0
    }
    /// Compile a function declaration statement.
    fn ct_compile_function_decl(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Stmt],
        is_async: bool,
        is_generator: bool,
        span: Span,
    ) -> CompileResult<()>;
    /// Compile a class declaration statement.
    fn ct_compile_class(&mut self, class_decl: &hudhudscript_ast::ClassDecl) -> CompileResult<()>;
    /// Compile a Decl (domain-specific declarations).
    fn ct_compile_decl(&mut self, decl: &Decl) -> CompileResult<()>;
    /// Compile a match pattern, returning JumpIfFalse positions to back-patch.
    fn ct_compile_match_pattern(
        &mut self,
        pattern: &hudhudscript_ast::MatchPattern,
    ) -> CompileResult<Vec<usize>>;
    /// Compile a destructuring pattern. The value is on the stack.
    fn ct_compile_destructure_pattern(
        &mut self,
        pattern: &hudhudscript_ast::Pattern,
        is_const: bool,
    ) -> CompileResult<()>;
    /// Compile an MCP server declaration.
    fn ct_compile_mcp_server(
        &mut self,
        decl: &hudhudscript_ast::McpServerDecl,
    ) -> CompileResult<()>;

    /// Issue #982: Register a trait's required method names for SOP enforcement.
    /// Called when compiling `Stmt::Trait`. Default is no-op (FunctionCompiler).
    fn ct_register_trait(&mut self, _name: &str, _methods: &[hudhudscript_ast::TraitMethodSig]) {}

    /// CROSS-1 (TCO): name of the function currently being compiled, if
    /// any.  Used by the shared `Stmt::Return` path to detect
    /// self-recursive tail calls.  Default is `None` (top-level
    /// `Compiler` has no surrounding function name).
    fn ct_current_fn_name(&self) -> Option<&str> {
        None
    }

    /// CROSS-1 (TCO): whether the function currently being compiled
    /// has a rest parameter (`...args`).  When true, self-tail-call
    /// emission is disabled because in-place rest-arg rebinding needs
    /// extra plumbing (bundle remaining stack values into an Array).
    fn ct_current_has_rest(&self) -> bool {
        false
    }

    // ── ISSUE-1: local type inference ───────────────────────────────────
    /// Return the compile-time known type of a local variable, if any.
    fn ct_local_type(&self, _name: &str) -> crate::compiler::expr::ExprType {
        crate::compiler::expr::ExprType::Unknown
    }
    /// Record the compile-time known type of a local variable.
    fn ct_set_local_type(&mut self, _name: &str, _ty: crate::compiler::expr::ExprType) {}

    /// ISSUE-2/3: return the slot index of a local variable if it is
    /// slot-mapped (FunctionCompiler), otherwise None.
    fn ct_local_reg(&self, _name: &str) -> Option<u8> {
        None
    }
    /// K1-3: return true if the named local is declared `const`.
    fn ct_is_const_local(&self, _name: &str) -> bool {
        false
    }
    /// WI-4: return true if the named local is captured by a closure.
    fn ct_is_captured_local(&self, _name: &str) -> bool {
        false
    }

    /// Return the next local register index so that RegAlloc can start
    /// above local variable registers, preventing runtime aliasing.
    fn ct_next_local_reg(&self) -> u8 {
        0
    }

    /// ISSUE-1: Return the last allocated match register (for patch sites).
    fn ct_match_reg(&self) -> u8 {
        0
    }
    /// Store the match-value register for use by ct_compile_match_pattern.
    fn ct_set_match_reg(&mut self, _reg: u8) {}
    /// Patch only the offset of a JumpIfFalse at `ip`, preserving src.
    fn ct_patch_jump_offset(&mut self, ip: usize, offset: i16);

    // ── G12: f-loop (unboxed float) bağlamı ─────────────────────────────
    /// Aday tablosunu, hoist edilmiş sabit slotlarını ve geçici-slot
    /// tabanını yığına iter (while girişi).
    fn ct_floop_push(
        &mut self,
        _slots: Vec<(String, u8)>,
        _consts: Vec<(u64, u8)>,
        _temp_base: u8,
    ) {
    }
    /// En üstteki f-loop bağlamını çıkarır (while çıkışı).
    fn ct_floop_pop(&mut self) {}
    /// `name` etkin bağlamda adaysa f-slot'unu döner.
    fn ct_floop_slot(&self, _name: &str) -> Option<u8> {
        None
    }
    /// Hoist edilmiş sabitin (f64 bit deseni) f-slot'unu döner.
    fn ct_floop_const_slot(&self, _bits: u64) -> Option<u8> {
        None
    }
    /// Geçici f-slot ayır (64 sınırı aşılırsa None — analiz bunu önler).
    fn ct_floop_temp(&mut self) -> Option<u8> {
        None
    }
    /// En son ayrılan geçici slot'u geri ver.
    fn ct_floop_temp_pop(&mut self) {}
    /// G12 kaçış kontrolü: isim bir kapama tarafından yakalanmış mı
    /// (derlenmiş kapamalar dahil — Local.is_captured + nested_captured).
    fn ct_floop_captured(&self, _name: &str) -> bool {
        false
    }
}
