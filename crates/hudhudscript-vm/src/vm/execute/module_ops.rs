#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_module_ops(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;
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

                // #921: Try module_resolver first if set
                if let Some(resolver) = &self.module_resolver {
                    match resolver.resolve(path, None) {
                        Ok(hudhudscript_errors::ModuleContent::Bytecode(bytes)) => {
                            match Bytecode::from_bytes(&bytes) {
                                Ok(module_bc) => {
                                    let mut sub_vm = VM::new();
                                    if let Err(e) = sub_vm.execute(&module_bc) {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Module '{}' error: {}",
                                            path, e
                                        )));
                                    }
                                    // Merge module functions into parent bytecode
                                    let names: Vec<String> =
                                        module_bc.function_keys();
                                    for name in names {
                                        if !bytecode.has_function(&name) {
                                            let chunk = module_bc
                                                .get_function(&name)
                                                .unwrap();
                                            bytecode.add_function(name, chunk);
                                        }
                                    }
                                    let module_val = Value16::object(
                                        sub_vm
                                            .globals
                                            .iter()
                                            .map(|(k, v)| {
                                                (hudhudscript_bytecode::interner::resolve(*k), *v)
                                            }),
                                    );
                                    self.set_var(&module_name, module_val)?;
                                    return Ok(StepAction::Jumped);
                                }
                                Err(e) => {
                                    return Err(compile_codes::runtime_error(format!(
                                        "Module '{}' bytecode corrupt: {}",
                                        path, e
                                    )));
                                }
                            }
                        }
                        Ok(hudhudscript_errors::ModuleContent::Source(_source)) => {
                            return Err(compile_codes::runtime_error(format!(
                            "Module '{}' resolved as source but VM requires compiled bytecode. \
                             Run `hudhud compile {}` first.",
                            path, path
                        )));
                        }
                        Ok(hudhudscript_errors::ModuleContent::Native { name }) => {
                            let mut obj = hudhudscript_bytecode::ObjMap::default();
                            obj.insert("__module".to_string(), Value16::string(name));
                            obj.insert("__loaded".to_string(), Value16::bool_(true));
                            let module_val = Value16::object(obj);
                            self.set_var(&module_name, module_val)?;
                            return Ok(StepAction::Jumped);
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
                    // Try pre-compiled bytecode first
                    let hudb_path = format!(
                        "{}.hudb",
                        path.trim_end_matches(".hudhud")
                            .trim_end_matches(".hud")
                            .trim_end_matches(".hudb")
                    );
                    if std::path::Path::new(&hudb_path).exists() {
                        // Load and execute pre-compiled bytecode
                        match std::fs::read(&hudb_path) {
                            Ok(bytes) => match Bytecode::from_bytes(&bytes) {
                                Ok(module_bc) => {
                                    let mut sub_vm = VM::new();
                                    if let Err(e) = sub_vm.execute(&module_bc) {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Module '{}' error: {}",
                                            path, e
                                        )));
                                    }
                                    // Merge module functions into parent bytecode
                                    let names: Vec<String> =
                                        module_bc.function_keys();
                                    for name in names {
                                        if !bytecode.has_function(&name) {
                                            let chunk = module_bc
                                                .get_function(&name)
                                                .unwrap();
                                            bytecode.add_function(name, chunk);
                                        }
                                    }
                                    Value16::object(
                                        sub_vm
                                            .globals
                                            .iter()
                                            .map(|(k, v)| {
                                                (hudhudscript_bytecode::interner::resolve(*k), *v)
                                            }),
                                    )
                                }
                                Err(e) => {
                                    return Err(compile_codes::runtime_error(format!(
                                        "Module '{}' bytecode corrupt: {}",
                                        path, e
                                    )));
                                }
                            },
                            Err(e) => {
                                return Err(compile_codes::runtime_error(format!(
                                    "Cannot read module '{}': {}",
                                    hudb_path, e
                                )));
                            }
                        }
                    } else if std::path::Path::new(path).exists() {
                        // Source file exists but no bytecode — need to compile first
                        return Err(compile_codes::runtime_error(format!(
                            "Module '{}' found but not compiled. Run `hudhud compile {}` first, \
                         or use the interpreter (`hudhud run`) for source-level imports.",
                            path, path
                        )));
                    } else {
                        // Builtin or non-existent — store as module marker
                        let mut obj = hudhudscript_bytecode::ObjMap::default();
                        obj.insert("__module".to_string(), Value16::string(path.clone()));
                        obj.insert("__loaded".to_string(), Value16::bool_(true));
                        Value16::object(obj)
                    }
                };

                self.set_var(&module_name, module_val)?;
            }

            Instruction::DefineFunction(payload_idx) => {
                // CROSS-2a: payload lives in `bytecode.define_function_payloads`.
                let payload = &bytecode.define_function_payloads[*payload_idx as usize];
                let name = bytecode.resolve_symbol(payload.name.0);
                let chunk_name = &payload.chunk_name;
                if let Some(chunk) = bytecode
                    .get_function(chunk_name.as_str())
                {
                    // Upvalue capture (Issue #1078 — interpreter parity):
                    // each captured name is promoted to a shared cell in
                    // the enclosing scope so mutations from the outer
                    // body and from every call of this closure go
                    // through the SAME slot.  Sibling closures created
                    // by different `counter()` calls get fresh cells
                    // (scope was pushed fresh per call).
                    let mut captured: std::collections::HashMap<
                        String,
                        Arc<parking_lot::RwLock<Value16>>,
                    > = HashMap::new();
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
