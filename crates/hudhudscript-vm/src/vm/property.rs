use crate::vm::VM;
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::SymId;
use hudhudscript_bytecode::Value16;
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
        let field = hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(field_sym.0));

        // SOP: subject instance state read — always reads from live instance
        if let Some(inner) = obj.as_object() {
            if inner.get("__type").and_then(|v| v.as_string()).as_deref() == Some("subject_instance") {
                if let Some(instance_id) = inner.get("__instance_id").and_then(|v| v.as_string()) {
                    if let Some(inst) = self.subject_instances.get(&instance_id) {
                        // SOP0008: if accessed via view ... as, check view state first
                        if let Some(view_name) = inner.get("__view_name").and_then(|v| v.as_string()) {
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

        // PERF: Fast path for .length on array/string — avoids interner
        // String alloc + HashMap lookup + instance/object dispatch chain.
        // Fast path for .length — cached sym ID, no interner lock needed.
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
        } else if let Some(mut map) = obj.as_object().cloned() {
            // Issue #438: mcp.ServerName => return MCP server proxy object.
            if map.get("__module").and_then(|v| v.as_string()).as_deref() == Some("mcp")
                && !map.contains_key("__server")
                && !field.starts_with("__")
                && field != "call"
            {
                let mut proxy = HashMap::new();
                proxy.insert("__module".to_string(), Value16::string("mcp"));
                proxy.insert("__server".to_string(), Value16::string(field.clone()));
                return Ok(Value16::object(proxy));
            }

            let is_env = map.get("__hudhud_env").and_then(|v| v.as_bool()) == Some(true);

            if let Some(val) = map.remove(&field) {
                return Ok(val);
            }

            if field.starts_with("__") {
                return Ok(Value16::null());
            }

            let mut current_parent = map.remove("__parent__");
            while let Some(parent_val) = current_parent {
                if let Some(pmap) = parent_val.as_object() {
                    let mut pmap = pmap.clone();
                    if let Some(v) = pmap.remove(&field) {
                        return Ok(v);
                    }
                    current_parent = pmap.remove("__parent__");
                } else {
                    break;
                }
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

    /// Push a new empty scope cell map for a function call.
    pub(crate) fn set_var(&mut self, name: &str, value: Value16) -> CompileResult<()> {
        // Check if the variable is immutable and already bound (reassignment attempt)
        if self.immutables.contains(name) && self.globals.contains_key(name) {
            return Err(compile_codes::runtime_error(format!(
                "Cannot reassign to constant variable '{}'",
                name
            )));
        }
        // Fast path: route through an existing upvalue cell when available.
        if let Some(cell) = self.find_cell(name) {
            *cell.write() = value;
            return Ok(());
        }
        // S2.2c: slot-first fast path. If the name has a slot allocated
        // by the compiler (params + `let` bindings inside a function
        // body), write to `registers[slot]` directly so subsequent
        // register reads see the update.
        // `WriteBackReceiver` after a method call flushes the mutated
        // receiver back into its slot (otherwise the slot keeps holding
        // PERF-B1: Mirror to globals so get_var() sees the value.
        // K7: sym_to_slot fast path removed — compiler emits slot ops now

        // Existing global? Update in place.
        if let Some(slot) = self.globals.get_mut(name) {
            *slot = value;
            return Ok(());
        }
        // New binding lands in globals.
        self.globals.insert(name.to_string(), value);
        Ok(())
    }
}
