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
        let field = hudhudscript_bytecode::interner::resolve(
            hudhudscript_bytecode::interner::SymbolId(field_sym.0),
        );

        // SOP: subject instance state read — always reads from live instance
        if let Some(inner) = obj.as_object() {
            if inner.get("__type").and_then(|v| v.as_string()).as_deref()
                == Some("subject_instance")
            {
                if let Some(instance_id) = inner.get("__instance_id").and_then(|v| v.as_string()) {
                    if let Some(inst) = self.subject_instances.get(&instance_id) {
                        // SOP0008: if accessed via view ... as, check view state first
                        if let Some(view_name) =
                            inner.get("__view_name").and_then(|v| v.as_string())
                        {
                            if let Some(view_state) = inst.views.get(&view_name) {
                                if let Some(val) = view_state.get(&field) {
                                    return Ok(*val);
                                }
                            }
                        }
                        if let Some(val) = inst.state.get(&field) {
                            return Ok(*val);
                        }
                        // SOP0006: view state lookup
                        for (_view_name, view_state) in &inst.views {
                            if let Some(val) = view_state.get(&field) {
                                return Ok(*val);
                            }
                        }
                    } else {
                        // SOP0005: despawned subject accessed
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
        if field_sym.0 == self.length_sym_id {
            if let Some(arr) = obj.as_array() {
                return Ok(Value16::int(arr.len() as i64));
            } else if let Some(s) = obj.as_string() {
                return Ok(Value16::int(s.len() as i64));
            }
        }

        if hudhudscript_bytecode::interner::resolve_with(
            hudhudscript_bytecode::interner::SymbolId(field_sym.0),
            |field| field == "length",
        ) {
            if let Some(arr) = obj.as_array() {
                return Ok(Value16::int(arr.len() as i64));
            } else if let Some(s) = obj.as_string() {
                return Ok(Value16::int(s.len() as i64));
            }
        }

        if let Some(inst) = obj.as_instance_data() {
            if let Some(val) = inst.fields.get(&field).cloned() {
                Ok(val)
            } else if field.starts_with("__") {
                Ok(Value16::null())
            } else {
                Err(compile_codes::runtime_error(format!(
                    "Property '{}' not found on {} instance",
                    field, inst.class_name
                )))
            }
        } else if let Some(map) = obj.as_object() {
            // Issue #438: mcp.ServerName => return MCP server proxy object.
            if map.get("__module").and_then(|v| v.as_string()).as_deref() == Some("mcp")
                && !map.contains_key("__server")
                && !field.starts_with("__")
                && field != "call"
            {
                let mut proxy = hudhudscript_bytecode::ObjMap::default();
                proxy.insert("__module".to_string(), Value16::string("mcp"));
                proxy.insert("__server".to_string(), Value16::string(field.clone()));
                return Ok(Value16::object(proxy));
            }

            let is_env = map.get("__hudhud_env").and_then(|v| v.as_bool()) == Some(true);

            // ISSUE-9c: avoid cloning the whole object map on every property read.
            // Walk __parent__ chain using borrowed references only.
            let mut current: Option<&hudhudscript_bytecode::ObjMap> = Some(map);
            while let Some(m) = current {
                if let Some(val) = m.get(&field) {
                    return Ok(*val);
                }
                if field.starts_with("__") {
                    return Ok(Value16::null());
                }
                current = m.get("__parent__").and_then(|p| p.as_object());
            }

            if is_env {
                if let Ok(live) = std::env::var(&field) {
                    return Ok(Value16::string(live));
                }
                return Ok(Value16::string(String::new()));
            }

            Err(compile_codes::runtime_error(format!(
                "Property '{}' not found on object",
                field
            )))
        } else {
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
        // K2-3: const reassignment guard (compile-time also rejects, but VM
        // keeps a runtime safety net for external set_var callers).
        let sym = hudhudscript_bytecode::interner::intern(name);
        if self.immutables.contains(&sym) && self.globals.contains_key(&sym) {
            return Err(compile_codes::runtime_error(format!(
                "Cannot reassign to constant variable '{}'",
                name
            )));
        }

        let sym_id = sym.0;
        let (slot, is_shared) = self
            .main_slot_encoded(sym_id)
            .map(crate::vm::VM::main_slot_decode)
            .unwrap_or((usize::MAX, false));
        let is_top_level = slot != usize::MAX;
        let is_main_only = is_top_level && !is_shared;

        // Fast path: route through an existing upvalue cell when available.
        // A top-level main-only function declaration can also be promoted to a
        // cell; in that rare case keep the cell in sync and mirror to the slot.
        // For shared top-level symbols, globals is the canonical home read by
        // LoadGlobal, so we must mirror the cell write to globals too.
        if let Some(cell) = self.find_cell(name) {
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
                // ISSUE-2e-E: shared top-level symbols live in shared_globals_vec;
                // mirror to HashMap for cold host/reflection consumers.
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
                // ISSUE-2e-D/E: main-only top-level symbols live only in their
                // absolute register slot.
                self.registers.set_absolute(slot, value);
            }
        } else {
            // S2.2c: non-top-level slot-first fast path. If the name has a slot
            // allocated by the compiler, write to the relative register slot
            // directly so subsequent register reads in the same frame see it.
            if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
                let local_syms = unsafe { &**local_syms_ptr };
                if !local_syms.is_empty() {
                    let cached = {
                        let cache = self.name_sym_cache.borrow();
                        cache.get(name).copied()
                    };
                    let resolved = cached.or_else(|| {
                        hudhudscript_bytecode::interner::try_resolve_id(name).map(|id| {
                            self.name_sym_cache
                                .borrow_mut()
                                .insert(name.to_string(), id);
                            id
                        })
                    });
                    if let Some(id) = resolved {
                        if let Ok(idx) = local_syms.binary_search_by_key(&id, |(s, _, _)| *s) {
                            let local_slot = local_syms[idx].1 as i32;
                            if local_slot >= 0 {
                                self.registers[local_slot as usize] = value;
                                return Ok(());
                            }
                        }
                    }
                }
            }
            // No local slot: new or updated global binding.
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
