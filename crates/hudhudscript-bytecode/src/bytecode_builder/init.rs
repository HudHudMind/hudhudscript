use crate::error::{compile_codes, CompileResult};
use crate::{Bytecode, FunctionChunk, BYTECODE_VERSION};
use std::sync::Arc;

impl Bytecode {
    pub fn new() -> Self {
        Self {
            version: BYTECODE_VERSION,
            constants: Vec::new(),
            instructions: Vec::new(),
            functions: std::cell::RefCell::new(Vec::new()),
            function_names: std::cell::RefCell::new(rustc_hash::FxHashMap::default()),
            serialized_function_names: Vec::new(),
            action_registry: std::cell::RefCell::new(std::collections::HashMap::new()),
            source_positions: Vec::new(),
            numeric_constants: Vec::new(),
            int_constants: Vec::new(),
            symbols: Vec::new(),
            symbol_lists: Vec::new(),
            symbol_index: rustc_hash::FxHashMap::default(),
            numeric_index: rustc_hash::FxHashMap::default(),
            int_index: rustc_hash::FxHashMap::default(),
            symbol_list_index: rustc_hash::FxHashMap::default(),
            loop_payloads: Vec::new(),
            char_dispatch_tables: Vec::new(),
            enum_decl_payloads: Vec::new(),
            class_decl_payloads: Vec::new(),
            trait_check_payloads: Vec::new(),
            load_module_payloads: Vec::new(),
            define_function_payloads: Vec::new(),
            class_static_decl_payloads: Vec::new(),
            destruct_object_payloads: Vec::new(),
            call_payloads: Vec::new(),
            two_sym_payloads: Vec::new(),
            opt_sym_payloads: Vec::new(),
            super_instr_payloads: Vec::new(),
            cmp_jump_payloads: Vec::new(),
            main_local_names: Vec::new(),
            main_local_shared: Vec::new(),
            main_local_count: 0,
            packed: std::cell::RefCell::new(None),
            needs_async: true, // WI-1: safe default, compiler sets false for pure sync
        }
    }

    // ── Function registry helpers (Faz 1: O(1) array-based lookup) ────────

    /// O(1) function lookup by name.  Resolves name → index via
    /// `function_names`, then indexes into the `functions` Vec.
    #[inline]
    pub fn get_function(&self, name: &str) -> Option<Arc<FunctionChunk>> {
        let idx = *self.function_names.borrow().get(name)?;
        self.functions.borrow().get(idx).cloned()
    }

    /// O(1) lookup by function index (v4.3 hot path).
    #[inline]
    pub fn get_function_by_index(&self, idx: u32) -> Option<Arc<FunctionChunk>> {
        self.functions.borrow().get(idx as usize).cloned()
    }

    /// Get the function index for a name (v4.3: used once at ClassDecl).
    #[inline]
    pub fn get_function_idx(&self, name: &str) -> Option<u32> {
        self.function_names.borrow().get(name).map(|&i| i as u32)
    }

    /// O(1) membership check.
    #[inline]
    pub fn has_function(&self, name: &str) -> bool {
        self.function_names.borrow().contains_key(name)
    }

    /// Register a function chunk under `name`.  Returns the assigned index.
    /// If the name already exists, the chunk is replaced in-place.
    #[inline]
    pub fn add_function(&self, name: String, chunk: Arc<FunctionChunk>) -> usize {
        let mut names = self.function_names.borrow_mut();
        if let Some(&idx) = names.get(&name) {
            self.functions.borrow_mut()[idx] = chunk;
            return idx;
        }
        let idx = self.function_count();
        self.functions.borrow_mut().push(chunk);
        names.insert(name, idx);
        idx
    }

    /// Iterate over all function names.
    #[inline]
    pub fn function_keys(&self) -> Vec<String> {
        self.function_names.borrow().keys().cloned().collect()
    }

    /// Return function names and chunks in the canonical function-vector order.
    /// A malformed name table is rejected instead of silently changing indices.
    pub fn function_entries_by_index(&self) -> CompileResult<Vec<(String, Arc<FunctionChunk>)>> {
        let functions = self.functions.borrow();
        let names = self.function_names.borrow();
        if names.len() != functions.len() {
            return Err(function_table_error(format!(
                "function table length mismatch: {} names for {} chunks",
                names.len(),
                functions.len()
            )));
        }

        let mut names_by_index = vec![None; functions.len()];
        for (name, &index) in names.iter() {
            if index >= functions.len() {
                return Err(function_table_error(format!(
                    "function '{}' points to out-of-range index {} (chunk count {})",
                    name,
                    index,
                    functions.len()
                )));
            }
            if let Some(previous) = names_by_index[index].replace(name.clone()) {
                return Err(function_table_error(format!(
                    "function index {} is assigned to both '{}' and '{}'",
                    index, previous, name
                )));
            }
        }

        names_by_index
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let name = name.ok_or_else(|| {
                    function_table_error(format!("function index {} has no name", index))
                })?;
                Ok((name, Arc::clone(&functions[index])))
            })
            .collect()
    }

    /// Resolve a function index to its canonical name.
    pub fn function_name_at(&self, index: u32) -> CompileResult<String> {
        let index = usize::try_from(index).map_err(|_| {
            function_table_error(format!("function index {} cannot fit usize", index))
        })?;
        let entries = self.function_entries_by_index()?;
        entries
            .get(index)
            .map(|(name, _)| name.clone())
            .ok_or_else(|| {
                function_table_error(format!(
                    "function index {} is out of range (chunk count {})",
                    index,
                    entries.len()
                ))
            })
    }

    /// Number of registered functions.
    #[inline]
    pub fn function_count(&self) -> usize {
        self.functions.borrow().len()
    }

    /// Resolve every `CallPayload::sym` to a direct `function_idx`.
    /// Payloads whose name is not in `function_names` keep `u32::MAX`
    /// and fall back to the slow symbol path at runtime.
    pub fn resolve_call_payload_function_indices(&mut self) {
        let names = self.function_names.borrow();
        let mut payloads = self.call_payloads.clone();
        for payload in payloads.iter_mut() {
            if payload.function_idx != u32::MAX {
                continue;
            }
            let name = crate::interner::resolve(crate::interner::SymbolId(payload.sym.0));
            if let Some(&idx) = names.get(&name) {
                payload.function_idx = idx as u32;
            }
        }
        drop(names);
        self.call_payloads = payloads;
    }

    /// Rebuild `function_names` from `serialized_function_names` (post-deserialization).
    pub fn rebuild_function_names(&self) -> CompileResult<()> {
        let function_count = self.functions.borrow().len();
        if self.serialized_function_names.len() != function_count {
            return Err(function_table_error(format!(
                "serialized function table has {} names for {} chunks",
                self.serialized_function_names.len(),
                function_count
            )));
        }

        let mut names = self.function_names.borrow_mut();
        names.clear();
        for (idx, name) in self.serialized_function_names.iter().enumerate() {
            if name.is_empty() {
                return Err(function_table_error(format!(
                    "serialized function index {} has an empty name",
                    idx
                )));
            }
            if let Some(previous) = names.insert(name.clone(), idx) {
                return Err(function_table_error(format!(
                    "serialized function '{}' appears at indices {} and {}",
                    name, previous, idx
                )));
            }
        }
        Ok(())
    }
}

#[cold]
#[inline(never)]
fn function_table_error(message: String) -> hudhudscript_errors::Error {
    compile_codes::invalid_bytecode(message)
}
