use crate::{Bytecode, BYTECODE_VERSION};

impl Bytecode {
    pub fn new() -> Self {
        Self {
            version: BYTECODE_VERSION,
            constants: Vec::new(),
            instructions: Vec::new(),
            functions: std::cell::RefCell::new(std::collections::HashMap::new()),
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
            main_local_names: Vec::new(),
            main_local_count: 0,
            packed: std::cell::RefCell::new(None),
        }
    }
}
