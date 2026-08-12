use crate::vm::VM;
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{FunctionChunk, SymId, Value16};
use std::collections::HashMap;

impl VM {
    pub(crate) fn get_property_op(
        &mut self,
        field_sym: &hudhudscript_bytecode::SymId,
    ) -> CompileResult<Value16> {
        let obj = self.registers[255];
        self.resolve_property(obj, field_sym)
    }

    pub(crate) fn resolve_property(
        &mut self,
        obj: Value16,
        field_sym: &hudhudscript_bytecode::SymId,
    ) -> CompileResult<Value16> {
        #[cfg(feature = "telemetry")]
        { self.telemetry.property_lookup_count += 1; }
        let sym = hudhudscript_bytecode::SymId(field_sym.0);

        // SOP: subject instance state read — always reads from live instance
        if let Some(inner) = obj.as_object() {
            if inner.get(&hudhudscript_bytecode::well_known::wk().type_).and_then(|v| v.as_string()).as_deref()
                == Some("subject_instance")
            {
                let field = hudhudscript_bytecode::interner::resolve(
                    hudhudscript_bytecode::interner::SymbolId(sym.0));
                if let Some(instance_id) = inner.get(&hudhudscript_bytecode::well_known::wk().instance_id).and_then(|v| v.as_string()) {
                    if let Some(inst) = self.subject_instances.get(&instance_id) {
                        if let Some(view_name) =
                            inner.get(&hudhudscript_bytecode::well_known::wk().view_name).and_then(|v| v.as_string())
                        {
                            if let Some(view_state) = inst.views.get(&view_name) {
                                if let Some(val) = view_state.get(&field) {
                                    return Ok(*val);
                                }
                            }
                            if let Some(val) = inst.state.get(&field) {
                                return Ok(*val);
                            }
                        } else {
                            if let Some(val) = inst.state.get(&field) {
                                return Ok(*val);
                            }
                            let mut view_names: Vec<_> = inst.views.keys().collect();
                            view_names.sort();
                            for view_name in view_names {
                                if let Some(view_state) = inst.views.get(view_name) {
                                    if let Some(val) = view_state.get(&field) {
                                        return Ok(*val);
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(compile_codes::runtime_error(format!(
                            "Cannot access property '{}' on despawned subject '{}'",
                            field, instance_id
                        )));
                    }
                }
            }
        }

        // Fast path for .length on array/string — cached sym ID, no interner
        // lock + HashMap lookup + instance/object dispatch chain.
        if field_sym.0 == hudhudscript_bytecode::well_known::wk().length.0 {
            if let Some(arr) = obj.as_array() {
                return Ok(Value16::int(arr.len() as i64));
            } else if let Some(len) = obj.str_char_len() {
                return Ok(Value16::int(len as i64));
            }
        }

        if let Some(inst) = obj.as_instance_data() {
            if let Some(val) = inst.fields.get(&sym).cloned() {
                Ok(val)
            } else if hudhudscript_bytecode::interner::resolve_with(
                hudhudscript_bytecode::interner::SymbolId(sym.0),
                |field| field.starts_with("__"))
            {
                Ok(Value16::null())
            } else {
                Err(compile_codes::runtime_error(format!(
                    "Property '{}' not found on {} instance",
                    hudhudscript_bytecode::interner::resolve(
                        hudhudscript_bytecode::interner::SymbolId(sym.0)),
                    inst.class_name
                )))
            }
        } else if let Some(map) = obj.as_object() {
            // Issue #438: mcp.ServerName => return MCP server proxy object.
            if map.get(&hudhudscript_bytecode::well_known::wk().module).and_then(|v| v.as_string()).as_deref() == Some("mcp")
                && !map.get(&hudhudscript_bytecode::well_known::wk().server).is_some()
                && !hudhudscript_bytecode::interner::resolve_with(
                    hudhudscript_bytecode::interner::SymbolId(sym.0),
                    |f| f.starts_with("__"))
                && hudhudscript_bytecode::interner::resolve_with(
                    hudhudscript_bytecode::interner::SymbolId(sym.0),
                    |f| f != "call")
            {
                let field = hudhudscript_bytecode::interner::resolve(
                    hudhudscript_bytecode::interner::SymbolId(sym.0));
                let mut proxy = hudhudscript_bytecode::ObjMap::default();
                proxy.insert("__module".to_string(), Value16::string("mcp"));
                proxy.insert("__server".to_string(), Value16::string(field.clone()));
                return Ok(Value16::object(proxy));
            }

            let is_env = map.get(&hudhudscript_bytecode::well_known::wk().hudhud_env).and_then(|v| v.as_bool()) == Some(true);

            // ISSUE-9c: avoid cloning the whole object map on every property read.
            // Walk __parent__ chain using borrowed references only.
            // Use SymId directly — ObjMap is SymId-keyed.
            let mut current: Option<&hudhudscript_bytecode::ObjMap> = Some(map);
            while let Some(m) = current {
                if let Some(val) = m.get(&sym) {
                    return Ok(*val);
                }
                let field = hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(sym.0));
                if field.starts_with("__") {
                    return Ok(Value16::null());
                }
                current = m.get(&hudhudscript_bytecode::well_known::wk().parent).and_then(|p| p.as_object());
            }

            if is_env {
                let field = hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(sym.0));
                if let Ok(live) = std::env::var(&field) {
                    return Ok(Value16::string(live));
                }
                return Ok(Value16::string(String::new()));
            }

            Err(compile_codes::runtime_error(format!(
                "Property '{}' not found on object",
                hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(sym.0))
            )))
        } else {
            let field = hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(sym.0));
            match crate::vm::operations::helpers::eval_member_access(obj, &field) {
                Ok(result) => Ok(result),
                Err(e) => Err(compile_codes::runtime_error(e.to_string())),
            }
        }
    }

    /// Set a variable by name, routing to its compiler-determined home.
    /// ISSUE-2e-D: top-level main-only symbols live only in their absolute
    /// register slot; shared symbols live in globals (slot mirror kept until
    /// 2e.E).  Non-top-level names use function-local slots or globals.
    pub(crate) fn set_var(&mut self, name: &str, value: Value16) -> CompileResult<()> {
        let sym_id = hudhudscript_bytecode::interner::intern(name).0;
        self.set_var_by_sym(sym_id, name, value)
    }

    /// P3: `set_var` variant that accepts a pre-interned SymId + display name.
    /// Avoids the `interner::intern()` call on the hot method dispatch path.
    /// Contains the single canonical const-reassignment guard + writeback logic.
    pub(crate) fn set_var_by_sym(
        &mut self,
        sym_id: u32,
        name: &str,
        value: Value16,
    ) -> CompileResult<()> {
        // T5.2: Inline `this` write — avoids FxHashMap insert.
        if sym_id == self.this_sym {
            self.cur_this = value;
            return Ok(());
        }
        // K2-3: const reassignment guard (shared by both set_var paths)
        let sym = hudhudscript_bytecode::interner::SymbolId(sym_id);
        if self.immutables.contains(&sym) && self.globals.contains_key(&sym) {
            return Err(compile_codes::runtime_error(format!(
                "Cannot reassign to constant variable '{}'",
                name
            )));
        }
        let (slot, is_shared) = self
            .main_slot_encoded(sym_id)
            .map(crate::vm::VM::main_slot_decode)
            .unwrap_or((usize::MAX, false));
        let is_top_level = slot != usize::MAX;
        let is_main_only = is_top_level && !is_shared;

        if let Some(cell) = self.find_cell_by_sym(sym_id) {
            *cell.write() = value;
            if is_main_only {
                self.registers.set_absolute(slot, value);
            } else if is_shared {
                let encoded = self.main_slot_encoded(sym_id).unwrap_or(0);
                let idx = crate::vm::VM::main_slot_shared_index(encoded);
                if idx < self.shared_globals_vec.len() {
                    self.shared_globals_vec[idx] = value;
                }
                self.globals.insert(sym, value);
            }
            return Ok(());
        }

        if is_top_level {
            if is_shared {
                if let Some(encoded) = self.main_slot_encoded(sym_id) {
                    let idx = crate::vm::VM::main_slot_shared_index(encoded);
                    if idx < self.shared_globals_vec.len() {
                        self.shared_globals_vec[idx] = value;
                    }
                }
                if let Some(existing) = self.globals.get_mut(&sym) {
                    *existing = value;
                } else {
                    self.globals.insert(sym, value);
                }
            } else {
                self.registers.set_absolute(slot, value);
            }
        } else {
            if let Some(existing) = self.globals.get_mut(&sym) {
                *existing = value;
            } else {
                self.globals.insert(sym, value);
            }
        }
        Ok(())
    }

    /// Property access helper.
    ///
    /// P3: ObjMap reverted to `FxHashMap<SymId, Value16>`, so the shape-based
    /// inline cache is no longer applicable.  This wrapper routes directly to
    /// `resolve_property` so SOP subject instances, `__parent__` chains, and
    /// other special cases keep working.
    #[inline(always)]
    pub(crate) fn get_property_ic(
        &mut self,
        obj_val: Value16,
        prop_sym: &SymId,
        _ip: usize,
    ) -> CompileResult<Value16> {
        self.resolve_property(obj_val, prop_sym)
    }

    /// Property write helper.
    ///
    /// P3: with the HashMap-backed `ObjMap`, writes go directly through
    /// `insert`.  The caller is responsible for rebuilding the value variant
    /// (e.g. `Value16::object`) afterwards.
    #[inline(always)]
    pub(crate) fn set_property_ic(
        &mut self,
        obj_map: &mut hudhudscript_bytecode::ObjMap,
        prop_sym: &SymId,
        value: Value16,
        _ip: usize,
    ) {
        obj_map.insert(*prop_sym, value);
    }
}
