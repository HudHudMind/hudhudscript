#![allow(unused_imports)]
use super::module_merge::merge_module_bytecode;
use super::*;

impl VM {
    #[inline(always)]
    pub fn step_module_ops(
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
                                    let guard = self.resolver_module_guard(base_dir, path)?;
                                    let module_val = self.load_module_from_bytecode(
                                        path, &module_bc, bytecode, None, guard,
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
                            let guard = self.resolver_module_guard(base_dir, path)?;
                            let module_val = self.load_module_from_source(
                                path, &source, bytecode, base_dir, guard,
                            )?;
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
                                    Ok(module_bc) => {
                                        let guard = self.filesystem_module_guard(&cand_path)?;
                                        self.load_module_from_bytecode(
                                            path, &module_bc, bytecode, None, guard,
                                        )?
                                    }
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
                                Ok(source) => {
                                    let guard = self.filesystem_module_guard(&cand_path)?;
                                    self.load_module_from_source(
                                        path,
                                        &source,
                                        bytecode,
                                        cand_path.parent(),
                                        guard,
                                    )?
                                }
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
