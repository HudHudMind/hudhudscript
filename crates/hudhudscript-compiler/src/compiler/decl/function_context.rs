use super::*;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, Instruction, Value16};
use std::sync::Arc;

impl Compiler {
    /// Canonical function chunk compilation context.
    /// Saves outer state, sets up function-local state, calls `emit_body`,
    /// merges payloads/constants back into outer bytecode, restores state,
    /// returns FunctionChunk.
    pub(crate) fn compile_function_chunk_with<F>(
        &mut self,
        params: Vec<String>,
        fn_name: Option<String>,
        is_async: bool,
        emit_body: F,
    ) -> CompileResult<FunctionChunk>
    where
        F: FnOnce(&mut Self) -> CompileResult<()>,
    {
        // ── Save outer compiler state ──────────────────────────────────
        let saved_bytecode = std::mem::replace(&mut self.bytecode, Bytecode::default());
        // B8: save outer int/numeric constants for inliner remap
        let saved_global_int = std::mem::take(&mut self.global_int_constants);
        let saved_global_num = std::mem::take(&mut self.global_numeric_constants);
        self.global_int_constants = saved_bytecode.int_constants.clone();
        self.global_numeric_constants = saved_bytecode.numeric_constants.clone();
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_slot_names = std::mem::take(&mut self.local_slot_names);
        let saved_scope_depth = self.scope_depth;
        self.scope_depth = 0;
        let saved_declared_fns = std::mem::replace(&mut self.declared_fns, vec![HashMap::new()]);
        let saved_in_top_level = self.in_top_level;
        self.in_top_level = false;
        let saved_max_register = self.current_max_register;
        self.current_max_register = 0;
        let saved_next_local_reg = self.next_local_reg;
        crate::compiler::regalloc::reset_temp_reg();
        crate::compiler::regalloc::reset_base();

        // ── Set function context ───────────────────────────────────────
        let mut saved_fn_ctx = self.fn_ctx.take();
        self.fn_ctx = Some(FuncCtx {
            params: params.clone(),
            fn_name: fn_name.clone(),
            has_rest: params.last().map(|p| p.starts_with("...")).unwrap_or(false),
            referenced: Vec::new(),
            nested_captured: HashSet::new(),
            is_async,
        });

        // ── Set up function-scope locals from parameters ───────────────
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

        // ── Emit function body ─────────────────────────────────────────
        let emit_result = emit_body(self);
        if let Err(e) = &emit_result {
            // Error path: restore outer state before propagating
            self.fn_ctx = saved_fn_ctx;
            self.bytecode = saved_bytecode;
            self.global_int_constants = saved_global_int;
            self.global_numeric_constants = saved_global_num;
            self.locals = saved_locals;
            self.local_slot_names = saved_slot_names;
            self.scope_depth = saved_scope_depth;
            self.declared_fns = saved_declared_fns;
            self.in_top_level = saved_in_top_level;
            self.current_max_register = saved_max_register;
            self.next_local_reg = saved_next_local_reg;
            return Err(e.clone());
        }

        // ── Implicit return for void functions ─────────────────────────
        if !matches!(
            self.bytecode.instructions.last(),
            Some(Instruction::Return { .. })
        ) {
            let null_idx = self.bytecode.constants.len() as u32;
            self.bytecode.constants.push(Value16::null());
            let tr = crate::compiler::regalloc::temp_reg();
            self.bytecode.push_instr(Instruction::LoadConst {
                dst: tr,
                const_idx: null_idx as u16,
            });
            self.bytecode.push_instr(Instruction::Return { src: tr });
        }

        // ── Run optimizer passes ───────────────────────────────────────
        let mut sp = std::mem::take(&mut self.bytecode.source_positions);
        crate::compiler::decl::function_optimizer::run_function_optimizer_passes(
            &mut self.bytecode,
            &mut sp,
            // G5: params+locals bölgesi korunur (fonksiyon evleri; generator
            // resume dahil chunk-sonu sonrası canlılık).
            self.next_local_reg,
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
        let func_cmp_jump = std::mem::take(&mut self.bytecode.cmp_jump_payloads);
        let mut func_nested = std::mem::take(&mut *self.bytecode.functions.borrow_mut());
        let func_names = std::mem::take(&mut *self.bytecode.function_names.borrow_mut());

        // ── Restore outer bytecode ─────────────────────────────────────
        let mut outer_bytecode = saved_bytecode;
        let mut func_instructions = func_instructions;

        // Merge numeric constants
        // Merge int constants
        crate::compiler::decl::function_optimizer::merge_function_constant_pools(
            &mut outer_bytecode,
            func_numeric,
            func_int,
            &mut func_instructions,
            &mut func_nested,
        );

        // Merge loop payloads
        if !func_loop.is_empty() {
            let base = outer_bytecode.loop_payloads.len() as u32;
            outer_bytecode
                .loop_payloads
                .extend(func_loop.iter().copied());
            for instr in &mut func_instructions {
                if let Instruction::LoopBegin(idx) = instr {
                    *idx += base;
                }
            }
        }

        // G4: cmp+branch payload'larını dış tabloya base-kaydırmalı taşı
        // (loop_payloads merge deseninin aynısı). u16 taşarsa komut
        // payload'dan geri açılır (kapasite sınırı; runtime'da unpacked
        // IntCmpRRJumpIfFalse aynı cmp çekirdeğini koşar).
        if !func_cmp_jump.is_empty() {
            let base = outer_bytecode.cmp_jump_payloads.len();
            outer_bytecode
                .cmp_jump_payloads
                .extend(func_cmp_jump.iter().copied());
            let rewrite = |instrs: &mut [Instruction]| {
                for (i, instr) in instrs.iter_mut().enumerate() {
                    if let Instruction::IntCmpRRJumpPacked { op, payload_idx } = *instr {
                        let new_idx = base + payload_idx as usize;
                        if new_idx <= u16::MAX as usize {
                            *instr = Instruction::IntCmpRRJumpPacked {
                                op,
                                payload_idx: new_idx as u16,
                            };
                        } else {
                            let p = func_cmp_jump[payload_idx as usize];
                            let offset = (p.target as i64 - i as i64) as i16;
                            *instr = Instruction::IntCmpRRJumpIfFalse {
                                src1: p.src1,
                                src2: p.src2,
                                op,
                                offset,
                            };
                        }
                    }
                }
            };
            rewrite(&mut func_instructions);
            for chunk in func_nested.iter_mut() {
                let c = std::sync::Arc::make_mut(chunk);
                rewrite(&mut c.instructions);
            }
        }

        // Merge declaration payload pools via macro
        macro_rules! merge_pool {
            ($outer:expr, $func:expr, $field:ident, $($pat:pat => $idx:ident),+) => {
                if !$func.is_empty() {
                    let base = $outer.$field.len() as u32;
                    $outer.$field.extend($func.drain(..));
                    for instr in &mut func_instructions { match instr { $($pat => { *$idx += base; })+ _ => {} } }
                }
            };
        }
        merge_pool!(outer_bytecode, func_enum, enum_decl_payloads, Instruction::EnumDecl(idx) => idx);
        merge_pool!(outer_bytecode, func_class, class_decl_payloads, Instruction::ClassDecl(idx) => idx);
        merge_pool!(outer_bytecode, func_traitck, trait_check_payloads, Instruction::TraitCheck(idx) => idx);
        merge_pool!(outer_bytecode, func_loadmod, load_module_payloads, Instruction::LoadModule(idx) => idx);
        merge_pool!(outer_bytecode, func_deffn, define_function_payloads, Instruction::DefineFunction(idx) => idx);
        merge_pool!(outer_bytecode, func_class_static, class_static_decl_payloads, Instruction::ClassStaticDecl(idx) => idx);
        merge_pool!(outer_bytecode, func_destruct, destruct_object_payloads, Instruction::DestructObject(idx) => idx);

        // Merge call payloads
        if !func_call_payloads.is_empty() {
            let base = outer_bytecode.call_payloads.len() as u16;
            for sp in func_super.iter_mut() {
                sp.call_idx += base as u32;
            }
            outer_bytecode
                .call_payloads
                .extend(func_call_payloads.drain(..));
            for instr in &mut func_instructions {
                if let Instruction::Call {
                    payload_idx: idx, ..
                }
                | Instruction::NewInstance {
                    payload_idx: idx, ..
                }
                | Instruction::MakeGenerator {
                    payload_idx: idx, ..
                }
                | Instruction::MethodCall {
                    payload_idx: idx, ..
                }
                | Instruction::SuperCall {
                    payload_idx: idx, ..
                } = instr
                {
                    *idx += base;
                }
            }
            for chunk in func_nested.iter_mut() {
                let c = Arc::make_mut(chunk);
                for instr in &mut c.instructions {
                    if let Instruction::Call {
                        payload_idx: idx, ..
                    }
                    | Instruction::NewInstance {
                        payload_idx: idx, ..
                    }
                    | Instruction::MakeGenerator {
                        payload_idx: idx, ..
                    }
                    | Instruction::MethodCall {
                        payload_idx: idx, ..
                    }
                    | Instruction::SuperCall {
                        payload_idx: idx, ..
                    } = instr
                    {
                        *idx += base;
                    }
                }
            }
        }

        // Merge super instruction payloads
        if !func_super.is_empty() {
            let base = outer_bytecode.super_instr_payloads.len() as u32;
            outer_bytecode
                .super_instr_payloads
                .extend(func_super.drain(..));
            for instr in &mut func_instructions {
                match instr {
                    Instruction::IntSubCall1(idx) => *idx += base,
                    Instruction::IntAddCall1(idx) => *idx += base,
                    Instruction::IntLeJumpIfFalse(idx) => *idx += base,
                    Instruction::IntLtJumpIfFalse(idx) => *idx += base,
                    _ => {}
                }
            }
        }

        // Merge two-sym payloads
        if !func_two_sym.is_empty() {
            let base = outer_bytecode.two_sym_payloads.len();
            outer_bytecode
                .two_sym_payloads
                .extend(func_two_sym.drain(..));
            for instr in &mut func_instructions {
                match instr {
                    Instruction::MatchVariant(idx) => *idx += base as u32,
                    Instruction::GetStatic(idx) => *idx += base as u32,
                    Instruction::DeclStore { payload_idx, .. } => *payload_idx += base as u16,
                    _ => {}
                }
            }
        }

        // Merge opt-sym payloads
        if !func_opt_sym.is_empty() {
            let base = outer_bytecode.opt_sym_payloads.len() as u16;
            outer_bytecode
                .opt_sym_payloads
                .extend(func_opt_sym.drain(..));
            for instr in &mut func_instructions {
                match instr {
                    Instruction::Remember { store_idx, .. } => *store_idx += base,
                    Instruction::Recall { store_idx, .. } => *store_idx += base,
                    Instruction::Forget { store_idx, .. } => *store_idx += base,
                    _ => {}
                }
            }
        }

        // Merge C6 char-dispatch tables
        if !self.bytecode.char_dispatch_tables.is_empty() {
            let base = outer_bytecode.char_dispatch_tables.len() as u16;
            outer_bytecode
                .char_dispatch_tables
                .extend(self.bytecode.char_dispatch_tables.drain(..));
            for instr in &mut func_instructions {
                if let Instruction::CharDispatch { table_idx, .. } = instr {
                    *table_idx += base;
                }
            }
        }

        // ── Register nested chunks ─────────────────────────────────────
        let mut name_by_idx: Vec<String> = vec![String::new(); func_nested.len()];
        for (name, &idx) in func_names.iter() {
            if idx < name_by_idx.len() {
                name_by_idx[idx] = name.clone();
            }
        }
        for (idx, chunk) in func_nested.into_iter().enumerate() {
            let chunk_name = std::mem::take(&mut name_by_idx[idx]);
            outer_bytecode.add_function(chunk_name, chunk);
        }

        // ── Resolve captures ───────────────────────────────────────────
        let named_params: Vec<String> = params
            .iter()
            .map(|p| {
                p.strip_prefix("...")
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| p.clone())
            })
            .collect();
        let referenced = self
            .fn_ctx
            .as_ref()
            .map(|c| c.referenced.clone())
            .unwrap_or_default();

        // Extract local names BEFORE building captures (needed for filter)
        let func_local_names = std::mem::take(&mut self.local_slot_names);
        let func_local_count = func_local_names.len() as u32;

        // ADIM B: which locals are genuinely captured by nested closures?
        let nested_captured: HashSet<&str> = self.fn_ctx.as_ref()
            .map(|c| c.nested_captured.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();

        let mut captures = Vec::new();
        let mut seen = HashSet::new();
        // ADIM B: a local is only a capture if it IS captured by a nested closure.
        // Pure locals (not in nested_captured) are excluded from captures.
        let pure_locals: HashSet<&str> = func_local_names.iter()
            .map(|s| s.as_str())
            .filter(|n| !nested_captured.contains(n))
            .collect();
        for name in &referenced {
            if !named_params.contains(name)
                && !pure_locals.contains(name.as_str())
                && !self.top_level_names.contains(name)
                && !self.shared_top_level_names.contains(name)
                && !seen.contains(name)
            {
                seen.insert(name.clone());
                captures.push(name.clone());
            }
        }

        if let Some(outer_fn_ctx) = &mut saved_fn_ctx {
            for cap in &captures {
                outer_fn_ctx.referenced.push(cap.clone());
                // ADIM B: track which locals are genuinely captured by nested closures
                outer_fn_ctx.nested_captured.insert(cap.clone());
            }
        }

        // ── Capture fn_declared_names BEFORE restore (used by local_set guard) ─
        let fn_declared_names: Vec<String> = self.declared_fns.last()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();

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

        // ── Build param_slots ──────────────────────────────────────────
        let param_slots: Vec<u16> = named_params
            .iter()
            .map(|name| func_local_names.iter().position(|n| n == name).unwrap_or(0) as u16)
            .collect();

        while sp.len() < func_instructions.len() {
            sp.push(None);
        }
        sp.truncate(func_instructions.len());

        // P1: plain function = no captures, non-async, non-generator
        let is_plain = !is_async && captures.is_empty();

        // P4: pre-compute SymId for each capture name
        let capture_sym_ids: Vec<u32> = captures
            .iter()
            .map(|name| hudhudscript_bytecode::interner::intern(name).0)
            .collect();

        // G5-slotvec: slot = position in captures vector (0..N-1)
        let capture_slots: Vec<u8> = (0..captures.len() as u8).collect();

        // G5-slotvec: replace LoadGlobal/StoreGlobal with LoadClosureSlot/StoreClosureSlot
        if !captures.is_empty() {
            let name_to_slot: std::collections::HashMap<String, u8> = captures.iter()
                .enumerate()
                .map(|(i, name)| (name.clone(), i as u8))
                .collect();
            // ADIM B: skip locally-defined names (params, locals, declared fns)
            // — shared_top_level_names only covers TOP-LEVEL fn names
            let local_set: std::collections::HashSet<&str> = named_params.iter()
                .map(|s| s.as_str())
                .chain(func_local_names.iter().map(|s| s.as_str()))
                .chain(fn_declared_names.iter().map(|s| s.as_str()))
                .collect();
            for instr in &mut func_instructions {
                match instr {
                    Instruction::LoadGlobal { dst, sym } => {
                        let name = hudhudscript_bytecode::interner::resolve(
                            hudhudscript_bytecode::interner::SymbolId(*sym as u32))
                            .to_string();
                        if !local_set.contains(name.as_str()) {
                            if let Some(&slot) = name_to_slot.get(&name) {
                                *instr = Instruction::LoadClosureSlot { dst: *dst, slot };
                            }
                        }
                    }
                    Instruction::StoreGlobal { src, sym } => {
                        let name = hudhudscript_bytecode::interner::resolve(
                            hudhudscript_bytecode::interner::SymbolId(*sym as u32))
                            .to_string();
                        if !local_set.contains(name.as_str()) {
                            if let Some(&slot) = name_to_slot.get(&name) {
                                *instr = Instruction::StoreClosureSlot { src: *src, slot };
                            }
                        }
                    }
                    // LANG-2 handled in drain pass below
                    _ => {}
                }
            }
            // LANG-2: register-based reads of captured locals → cell reads
            let mut new_instrs: Vec<Instruction> = Vec::with_capacity(func_instructions.len() + captures.len());
            for instr in func_instructions.drain(..) {
                match instr {
                    Instruction::Return { src } if (src as usize) < func_local_names.len() => {
                        let name = &func_local_names[src as usize];
                        if let Some(&slot) = name_to_slot.get(name) {
                            // Insert cell load before return
                            new_instrs.push(Instruction::LoadClosureSlot { dst: 254, slot });
                            new_instrs.push(Instruction::Return { src: 254 });
                        } else {
                            new_instrs.push(Instruction::Return { src });
                        }
                    }
                    other => new_instrs.push(other),
                }
            }
            func_instructions = new_instrs;
        }

        Ok(FunctionChunk {
            params: params.clone(),
            instructions: func_instructions,
            constants: func_constants,
            captures,
            capture_sym_ids,
            capture_slots,
            is_async,
            is_generator: false,
            local_count: func_local_count,
            local_names: func_local_names,
            capture_cells: vec![],
            max_register,
            sym_to_slot: std::sync::OnceLock::new(),
            source_positions: sp,
            param_slots: param_slots.into_boxed_slice(),
            is_plain_function: is_plain,
        })
    }
}
