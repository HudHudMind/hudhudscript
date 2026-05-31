use crate::vm::VM;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

impl VM {
    pub(crate) fn copy_parent_methods(
        &self,
        class_name: &str,
        instance: &mut HashMap<String, Value16>,
    ) {
        if let Some((Some(parent_name), _methods)) = self.classes.get(class_name) {
            // Recurse for grandparent first
            self.copy_parent_methods(parent_name, instance);
            // Copy parent's instance methods
            if let Some((_gp, parent_methods)) = self.classes.get(parent_name) {
                for method in parent_methods {
                    if method != "constructor" {
                        let chunk_name = format!("{}::{}", parent_name, method);
                        // Only insert if not already overridden
                        instance
                            .entry(method.clone())
                            .or_insert(Value16::string(chunk_name));
                    }
                }
            }
        }
    }

    /// Check if a class transitively extends Error (Issue #1016).
    pub(crate) fn class_extends_error(&self, class_name: &str) -> bool {
        if let Some((Some(parent_name), _)) = self.classes.get(class_name) {
            if parent_name == "Error" {
                return true;
            }
            return self.class_extends_error(parent_name);
        }
        false
    }

    /// After constructor runs, collect this.field assignments back into the instance.
    pub(crate) fn collect_this_fields(
        &self,
        mut instance: HashMap<String, Value16>,
    ) -> HashMap<String, Value16> {
        if let Some(this_val) = self.get_var_cloned("this") {
            if let Some(updated) = this_val.as_object() {
                instance = updated.iter().map(|(k, v)| (k.clone(), *v)).collect();
            } else if let Some(inst) = this_val.as_instance_data() {
                instance = inst.fields.iter().map(|(k, v)| (k.clone(), *v)).collect();
            }
        }
        // Also collect any "this.field" composite-key variables from globals
        for (k, v) in self.globals.iter() {
            if let Some(field) = k.strip_prefix("this.") {
                instance.insert(field.to_string(), v.clone());
            }
        }
        instance
    }

    // -----------------------------------------------------------------------
    // User-defined function execution
    // -----------------------------------------------------------------------
}
