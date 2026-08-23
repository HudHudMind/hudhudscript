use crate::vm::VM;
use hudhudscript_bytecode::Value16;

impl VM {
    pub(super) fn register_late_globals(&mut self) {
        register(self, "database", "database");
        register(self, "StringBuilder", "StringBuilder");
        register(self, "ocr", "ocr");
        register(self, "workflow", "workflow");
        register(self, "Web", "Web");
    }
}

fn register(vm: &mut VM, global: &str, module: &str) {
    let mut object = hudhudscript_bytecode::ObjMap::default();
    object.insert("__module".to_string(), Value16::string(module.to_string()));
    vm.set_global(global, Value16::object(object));
}
