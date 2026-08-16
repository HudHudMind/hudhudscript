#![allow(unused_imports)]
use super::module_merge::merge_module_bytecode;
use super::*;
use crate::vm::module_load_context::{ModuleIdentity, ModuleLoadGuard};

impl VM {
    pub(crate) fn load_module_from_bytecode(
        &mut self,
        path: &str,
        module_bc: &Bytecode,
        bytecode: &Bytecode,
        export_names: Option<&[String]>,
        guard: ModuleLoadGuard,
    ) -> CompileResult<Value16> {
        let mut sub_vm = VM::new();
        sub_vm.module_load_context = Arc::clone(&self.module_load_context);
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
        merge_module_bytecode(module_bc, bytecode).map_err(|e| {
            compile_codes::runtime_error(format!("Module '{}' resolve error: {}", path, e))
        })?;
        drop(guard);

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

    pub(crate) fn load_module_from_source(
        &mut self,
        path: &str,
        source: &str,
        bytecode: &Bytecode,
        base_dir: Option<&std::path::Path>,
        guard: ModuleLoadGuard,
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
        let export_names = super::module_merge::collect_module_export_names(&ast);
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
            Ok(module_bc) => self.load_module_from_bytecode(
                path,
                &module_bc,
                bytecode,
                Some(&export_names),
                guard,
            ),
            Err(e) => {
                return Err(compile_codes::runtime_error(format!(
                    "Compile error in module '{}': {:?}",
                    path, e
                )));
            }
        }
    }

    /// Filesystem module identity: canonical path, so aliases and relative
    /// variants of the same file collapse to one identity.
    pub(crate) fn filesystem_module_guard(
        &self,
        candidate: &std::path::Path,
    ) -> CompileResult<ModuleLoadGuard> {
        let canonical = std::fs::canonicalize(candidate).map_err(|error| {
            compile_codes::runtime_error(format!(
                "Cannot canonicalize module '{}': {}",
                candidate.display(),
                error
            ))
        })?;
        let identity = ModuleIdentity(canonical.to_string_lossy().into_owned());
        ModuleLoadGuard::enter(Arc::clone(&self.module_load_context), identity)
    }

    /// Resolver-only module identity: no filesystem, so (base_dir, path)
    /// uniqueness is the strongest available identity.
    pub(crate) fn resolver_module_guard(
        &self,
        base_dir: Option<&std::path::Path>,
        requested_path: &str,
    ) -> CompileResult<ModuleLoadGuard> {
        let identity = ModuleIdentity(format!(
            "resolver:{}:{}",
            base_dir
                .unwrap_or_else(|| std::path::Path::new(""))
                .display(),
            requested_path
        ));
        ModuleLoadGuard::enter(Arc::clone(&self.module_load_context), identity)
    }
}
