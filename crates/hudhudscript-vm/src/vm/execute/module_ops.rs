#![allow(unused_imports)]
use super::*;

impl VM {
    fn load_module_from_bytecode(
        &mut self,
        path: &str,
        module_bc: &Bytecode,
        bytecode: &Bytecode,
        export_names: Option<&[String]>,
    ) -> CompileResult<Value16> {
        let mut sub_vm = VM::new();
        let initial_globals: Option<rustc_hash::FxHashSet<_>> = if export_names.is_none() {
            Some(sub_vm.globals.keys().copied().collect())
        } else {
            None
        };
        if let Err(e) = sub_vm.execute(module_bc) {
            return Err(compile_codes::runtime_error(format!(
                "Module '{}' error: {}",
                path, e
            )));
        }
        // Merge module functions into parent bytecode
        let names: Vec<String> = module_bc.function_keys();
        for name in names {
            if !bytecode.has_function(&name) {
                let chunk = module_bc.get_function(&name).unwrap();
                let remapped = remap_and_merge_chunk(module_bc, bytecode, chunk);
                bytecode.add_function(name, remapped);
            }
        }

        let module_actions: Vec<_> = module_bc
            .action_registry
            .borrow()
            .iter()
            .map(|(name, chunk)| (name.clone(), std::sync::Arc::clone(chunk)))
            .collect();

        for (name, chunk) in module_actions {
            let remapped = remap_and_merge_chunk(module_bc, bytecode, chunk);
            bytecode
                .action_registry
                .borrow_mut()
                .entry(name)
                .or_insert(remapped);
        }

        for (name, id) in sub_vm.agent_names.iter() {
            self.agent_names.insert(name.clone(), *id);
        }

        for (name, class_data) in sub_vm.classes.iter() {
            self.classes.insert(name.clone(), class_data.clone());
        }

        let mut exports = hudhudscript_bytecode::ObjMap::default();

        if let Some(names) = export_names {
            for name in names {
                if let Some(value) = sub_vm.get_var_cloned(name) {
                    exports.insert(name.clone(), value);
                }
            }
        } else {
            let initials = initial_globals.unwrap();
            for (sym, value) in sub_vm.globals.iter() {
                if initials.contains(sym) {
                    continue;
                }
                let name = hudhudscript_bytecode::interner::resolve(*sym);
                // Skip internal names
                if name == "this"
                    || name == "env"
                    || name == "__hudhud_env"
                    || name == "tcp"
                    || name == "http"
                    || name == "fs"
                    || name == "exec"
                    || name == "__module"
                    || name == "__loaded"
                {
                    continue;
                }
                exports.insert(name, *value);
            }
        }

        let module_val = Value16::object(exports);
        Ok(module_val)
    }

    fn load_module_from_source(
        &mut self,
        path: &str,
        source: &str,
        bytecode: &Bytecode,
        base_dir: Option<&std::path::Path>,
    ) -> CompileResult<Value16> {
        let ast = match hudhudscript_parser::parse(source) {
            Ok(ast) => ast,
            Err(e) => {
                return Err(compile_codes::runtime_error(format!(
                    "Parse error in module '{}': {}",
                    path, e
                )));
            }
        };
        let export_names = collect_module_export_names(&ast);
        let mut compiler = hudhudscript_compiler::Compiler::new();
        if let Some(dir) = base_dir {
            compiler.set_module_base_dir(dir.to_path_buf());
        } else {
            let p = std::path::Path::new(path);
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    compiler.set_module_base_dir(parent.to_path_buf());
                } else {
                    compiler.set_module_base_dir(std::path::Path::new(".").to_path_buf());
                }
            }
        }

        match compiler.compile(&ast) {
            Ok(module_bc) => {
                self.load_module_from_bytecode(path, &module_bc, bytecode, Some(&export_names))
            }
            Err(e) => {
                return Err(compile_codes::runtime_error(format!(
                    "Compile error in module '{}': {:?}",
                    path, e
                )));
            }
        }
    }

    #[inline(always)]
    pub(crate) fn step_module_ops(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let _instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let _ip = ctx.ip;
        let _ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::LoadModule(payload_idx) => {
                // CROSS-2a: payload lives in `bytecode.load_module_payloads`.
                let payload = &bytecode.load_module_payloads[*payload_idx as usize];
                let path = &payload.path;
                let alias_sym = payload.alias;
                let module_name = alias_sym
                    .as_ref()
                    .map(|s| bytecode.resolve_symbol(s.0))
                    .unwrap_or_else(|| {
                        path.rsplit('/')
                            .next()
                            .unwrap_or(path.as_str())
                            .trim_end_matches(".hudhud")
                            .trim_end_matches(".hud")
                            .trim_end_matches(".js")
                            .trim_end_matches(".ts")
                            .to_string()
                    });

                let base_dir = payload.base_dir.as_ref().map(|s| std::path::Path::new(s));

                // #921: Try module_resolver first if set
                if let Some(resolver) = &self.module_resolver {
                    match resolver.resolve(path, base_dir.and_then(|p| p.to_str())) {
                        Ok(hudhudscript_errors::ModuleContent::Bytecode(bytes)) => {
                            match Bytecode::from_bytes(&bytes) {
                                Ok(module_bc) => {
                                    let module_val = self.load_module_from_bytecode(
                                        path, &module_bc, bytecode, None,
                                    )?;
                                    self.set_var(&module_name, module_val)?;
                                    return Ok(StepAction::Advance);
                                }
                                Err(e) => {
                                    return Err(compile_codes::runtime_error(format!(
                                        "Module '{}' bytecode corrupt: {}",
                                        path, e
                                    )));
                                }
                            }
                        }
                        Ok(hudhudscript_errors::ModuleContent::Source(source)) => {
                            let module_val =
                                self.load_module_from_source(path, &source, bytecode, base_dir)?;
                            self.set_var(&module_name, module_val)?;
                            return Ok(StepAction::Advance);
                        }
                        Ok(hudhudscript_errors::ModuleContent::Native { name }) => {
                            let mut obj = hudhudscript_bytecode::ObjMap::default();
                            obj.insert("__module".to_string(), Value16::string(name));
                            obj.insert("__loaded".to_string(), Value16::bool_(true));
                            let module_val = Value16::object(obj);
                            self.set_var(&module_name, module_val)?;
                            return Ok(StepAction::Advance);
                        }
                        Err(_) => {
                            // Resolver failed, fall through to default behavior
                        }
                    }
                }

                // #706: Try to load pre-compiled bytecode (.hudb), then fall back
                let module_val = if let Some(existing) = self.get_var(&module_name).cloned() {
                    existing
                } else {
                    let base_path_obj = base_dir.unwrap_or_else(|| std::path::Path::new("."));
                    let is_explicit = path.ends_with(".hud")
                        || path.ends_with(".hudhud")
                        || path.ends_with(".hudb");

                    let mut candidates = Vec::new();
                    candidates.push(base_path_obj.join(path));

                    if !is_explicit {
                        candidates.push(base_path_obj.join(format!("{}.hudhud", path)));
                        candidates.push(base_path_obj.join(format!("{}.hud", path)));
                        candidates.push(base_path_obj.join(format!("{}.hudb", path)));
                    }

                    let mut found = None;
                    for cand in &candidates {
                        if cand.exists() {
                            found = Some(cand.clone());
                            break;
                        }
                    }

                    if let Some(cand_path) = found {
                        let is_hudb =
                            cand_path.extension().and_then(|s| s.to_str()) == Some("hudb");
                        if is_hudb {
                            match std::fs::read(&cand_path) {
                                Ok(bytes) => match Bytecode::from_bytes(&bytes) {
                                    Ok(module_bc) => self.load_module_from_bytecode(
                                        path, &module_bc, bytecode, None,
                                    )?,
                                    Err(e) => {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Module '{}' bytecode corrupt: {}",
                                            cand_path.display(),
                                            e
                                        )));
                                    }
                                },
                                Err(e) => {
                                    return Err(compile_codes::runtime_error(format!(
                                        "Cannot read module '{}': {}",
                                        cand_path.display(),
                                        e
                                    )));
                                }
                            }
                        } else {
                            match std::fs::read_to_string(&cand_path) {
                                Ok(source) => self.load_module_from_source(
                                    path,
                                    &source,
                                    bytecode,
                                    cand_path.parent(),
                                )?,
                                Err(e) => {
                                    return Err(compile_codes::runtime_error(format!(
                                        "Cannot read module '{}': {}",
                                        cand_path.display(),
                                        e
                                    )));
                                }
                            }
                        }
                    } else {
                        if is_explicit || path.contains('/') || path.contains('\\') {
                            return Err(compile_codes::runtime_error(format!(
                                "Cannot read module '{}': file not found",
                                path
                            )));
                        } else {
                            // Builtin or non-existent — store as module marker
                            let mut obj = hudhudscript_bytecode::ObjMap::default();
                            obj.insert("__module".to_string(), Value16::string(path.clone()));
                            obj.insert("__loaded".to_string(), Value16::bool_(true));
                            Value16::object(obj)
                        }
                    }
                };

                self.set_var(&module_name, module_val)?;
            }

            Instruction::DefineFunction(payload_idx) => {
                // CROSS-2a: payload lives in `bytecode.define_function_payloads`.
                let payload = &bytecode.define_function_payloads[*payload_idx as usize];
                let name = bytecode.resolve_symbol(payload.name.0);
                let chunk_name = &payload.chunk_name;
                if let Some(chunk) = bytecode.get_function(chunk_name.as_str()) {
                    // Upvalue capture (Issue #1078 — interpreter parity):
                    // each captured name is promoted to a shared cell in
                    // the enclosing scope so mutations from the outer
                    // body and from every call of this closure go
                    // through the SAME slot.  Sibling closures created
                    // by different `counter()` calls get fresh cells
                    // (scope was pushed fresh per call).
                    let mut captured: std::collections::HashMap<
                        String,
                        std::sync::Arc<parking_lot::RwLock<Value16>>,
                    > = std::collections::HashMap::new();
                    for cap_name in chunk.captures.iter() {
                        // Skip dead-end captures (see LoadConst for
                        // rationale — spurious names from popped block
                        // scopes must not install Null cells).
                        if let Some(cell) = self.upvalue_cell_for(cap_name) {
                            captured.insert(cap_name.clone(), cell);
                        }
                    }
                    let func_val = Value16::function(FunctionData {
                        name: name.clone(),
                        params: chunk.params.clone(),
                        chunk_name: chunk_name.clone(),
                chunk_sym: hudhudscript_bytecode::interner::intern(&chunk_name.clone()).0,
                        captures: captured,
                    });
                    self.set_var(&name, func_val)?;
                }
            }

            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}

pub(crate) fn collect_module_export_names(ast: &[hudhudscript_ast::Stmt]) -> Vec<String> {
    let mut names = Vec::new();
    for stmt in ast {
        match stmt {
            hudhudscript_ast::Stmt::Decl(decl) => match decl {
                hudhudscript_ast::Decl::Agent { name, .. } => names.push(name.clone()),
                hudhudscript_ast::Decl::Provider { name, .. } => names.push(name.clone()),
                hudhudscript_ast::Decl::Action { name, .. } => names.push(name.clone()),
                hudhudscript_ast::Decl::Tool { name, .. } => names.push(name.clone()),
                hudhudscript_ast::Decl::Resource { name, .. } => names.push(name.clone()),
                hudhudscript_ast::Decl::Subject { name, .. } => names.push(name.clone()),
                hudhudscript_ast::Decl::Role { name, .. } => names.push(name.clone()),
                hudhudscript_ast::Decl::Entity { name, .. } => names.push(name.clone()),
                _ => {}
            },
            hudhudscript_ast::Stmt::VarDecl(var_decl) => names.push(var_decl.name.clone()),
            hudhudscript_ast::Stmt::Let { name, .. } => names.push(name.clone()),
            hudhudscript_ast::Stmt::Const { name, .. } => names.push(name.clone()),
            hudhudscript_ast::Stmt::Function { name, .. } => names.push(name.clone()),
            hudhudscript_ast::Stmt::Class(class_decl) => names.push(class_decl.name.clone()),
            hudhudscript_ast::Stmt::Trait { name, .. } => names.push(name.clone()),
            hudhudscript_ast::Stmt::EnumDecl { name, .. } => names.push(name.clone()),
            hudhudscript_ast::Stmt::Export { item, .. } => {
                let mut nested = collect_module_export_names(std::slice::from_ref(item.as_ref()));
                names.append(&mut nested);
            }
            _ => {}
        }
    }
    names
}

fn remap_and_merge_chunk(
    source_bc: &hudhudscript_bytecode::Bytecode,
    target_bc: &hudhudscript_bytecode::Bytecode,
    chunk: std::sync::Arc<hudhudscript_bytecode::FunctionChunk>,
) -> std::sync::Arc<hudhudscript_bytecode::FunctionChunk> {
    #[allow(invalid_reference_casting)]
    let target_mut =
        unsafe { &mut *(target_bc as *const _ as *mut hudhudscript_bytecode::Bytecode) };

    let call_base = target_mut.call_payloads.len() as u16;
    let load_module_base = target_mut.load_module_payloads.len() as u32;
    let define_function_base = target_mut.define_function_payloads.len() as u32;
    let enum_decl_base = target_mut.enum_decl_payloads.len() as u32;
    let class_decl_base = target_mut.class_decl_payloads.len() as u32;
    let trait_check_base = target_mut.trait_check_payloads.len() as u32;
    let class_static_decl_base = target_mut.class_static_decl_payloads.len() as u32;
    let destruct_object_base = target_mut.destruct_object_payloads.len() as u32;
    let two_sym_base = target_mut.two_sym_payloads.len() as u32;
    let opt_sym_base = target_mut.opt_sym_payloads.len() as u32;
    let loop_base = target_mut.loop_payloads.len() as u32;
    let super_instr_base = target_mut.super_instr_payloads.len() as u32;
    let cmp_jump_base = target_mut.cmp_jump_payloads.len() as u32;

    target_mut
        .call_payloads
        .extend_from_slice(&source_bc.call_payloads);
    target_mut
        .load_module_payloads
        .extend_from_slice(&source_bc.load_module_payloads);
    target_mut
        .define_function_payloads
        .extend_from_slice(&source_bc.define_function_payloads);
    target_mut
        .enum_decl_payloads
        .extend_from_slice(&source_bc.enum_decl_payloads);
    target_mut
        .class_decl_payloads
        .extend_from_slice(&source_bc.class_decl_payloads);
    target_mut
        .trait_check_payloads
        .extend_from_slice(&source_bc.trait_check_payloads);
    target_mut
        .class_static_decl_payloads
        .extend_from_slice(&source_bc.class_static_decl_payloads);
    target_mut
        .destruct_object_payloads
        .extend_from_slice(&source_bc.destruct_object_payloads);
    target_mut
        .two_sym_payloads
        .extend_from_slice(&source_bc.two_sym_payloads);
    target_mut
        .opt_sym_payloads
        .extend_from_slice(&source_bc.opt_sym_payloads);
    target_mut
        .loop_payloads
        .extend_from_slice(&source_bc.loop_payloads);
    target_mut
        .cmp_jump_payloads
        .extend_from_slice(&source_bc.cmp_jump_payloads);

    let mut modified_super = source_bc.super_instr_payloads.clone();
    for sp in &mut modified_super {
        sp.call_idx += call_base as u32;
    }
    target_mut.super_instr_payloads.extend(modified_super);

    let num_base = target_mut.numeric_constants.len() as u16;
    target_mut
        .numeric_constants
        .extend_from_slice(&source_bc.numeric_constants);

    let int_base = target_mut.int_constants.len() as u16;
    target_mut
        .int_constants
        .extend_from_slice(&source_bc.int_constants);

    let mut new_chunk = (*chunk).clone();

    fn remap_instrs(
        instrs: &mut [Instruction],
        call_base: u16,
        load_module_base: u32,
        define_function_base: u32,
        enum_decl_base: u32,
        class_decl_base: u32,
        trait_check_base: u32,
        class_static_decl_base: u32,
        destruct_object_base: u32,
        two_sym_base: u32,
        opt_sym_base: u32,
        loop_base: u32,
        super_instr_base: u32,
        cmp_jump_base: u32,
        num_base: u16,
        int_base: u16,
    ) {
        for instr in instrs {
            match instr {
                Instruction::Call { payload_idx, .. }
                | Instruction::MethodCall { payload_idx, .. }
                | Instruction::SuperCall { payload_idx, .. }
                | Instruction::NewInstance { payload_idx, .. }
                | Instruction::MakeGenerator { payload_idx, .. } => {
                    *payload_idx += call_base;
                }
                Instruction::LoadModule(idx) => {
                    *idx += load_module_base;
                }
                Instruction::DefineFunction(idx) => {
                    *idx += define_function_base;
                }
                Instruction::EnumDecl(idx) => {
                    *idx += enum_decl_base;
                }
                Instruction::ClassDecl(idx) => {
                    *idx += class_decl_base;
                }
                Instruction::TraitCheck(idx) => {
                    *idx += trait_check_base;
                }
                Instruction::ClassStaticDecl(idx) => {
                    *idx += class_static_decl_base;
                }
                Instruction::DestructObject(idx) => {
                    *idx += destruct_object_base;
                }
                Instruction::MatchVariant(idx) | Instruction::GetStatic(idx) => {
                    *idx += two_sym_base;
                }
                Instruction::DeclStore { payload_idx, .. } => {
                    *payload_idx += two_sym_base as u16;
                }
                Instruction::Remember { store_idx, .. }
                | Instruction::Recall { store_idx, .. }
                | Instruction::Forget { store_idx, .. } => {
                    *store_idx += opt_sym_base as u16;
                }
                Instruction::LoopBegin(idx) => {
                    *idx += loop_base;
                }
                Instruction::IntSubCall1(idx)
                | Instruction::IntAddCall1(idx)
                | Instruction::IntLeJumpIfFalse(idx)
                | Instruction::IntLtJumpIfFalse(idx) => {
                    *idx += super_instr_base;
                }
                Instruction::LoadNumConst { const_idx, .. } => {
                    *const_idx += num_base;
                }
                Instruction::LoadIntConst { const_idx, .. }
                | Instruction::ArrayPushIntConst { const_idx, .. } => {
                    *const_idx += int_base;
                }
                Instruction::IntLtRRJumpPacked(idx) | Instruction::IntLeRRJumpPacked(idx) => {
                    *idx += cmp_jump_base;
                }
                _ => {}
            }
        }
    }

    remap_instrs(
        &mut new_chunk.instructions,
        call_base,
        load_module_base,
        define_function_base,
        enum_decl_base,
        class_decl_base,
        trait_check_base,
        class_static_decl_base,
        destruct_object_base,
        two_sym_base,
        opt_sym_base,
        loop_base,
        super_instr_base,
        cmp_jump_base,
        num_base,
        int_base,
    );

    std::sync::Arc::new(new_chunk)
}
