// O(1) lookup indices, lazily built on first access.  Audit v3 Finding
// 10.1 — the previous `iter().find()` was O(n) over 22+ modules and
// 50+ globals.  Knuth TAOCP v.3 §6.4 amortized O(1) hash lookup.

use std::sync::OnceLock;

use super::BuiltinModule;

static MODULE_INDEX: OnceLock<rustc_hash::FxHashMap<&'static str, &'static BuiltinModule>> =
    OnceLock::new();
static GLOBAL_INDEX: OnceLock<rustc_hash::FxHashMap<&'static str, &'static super::BuiltinMember>> =
    OnceLock::new();

fn module_index() -> &'static rustc_hash::FxHashMap<&'static str, &'static BuiltinModule> {
    MODULE_INDEX.get_or_init(|| {
        let mut m = rustc_hash::FxHashMap::default();
        for module in super::modules_core::BUILTIN_MODULES_CORE.iter() {
            m.insert(module.name, module);
        }
        for module in super::modules_io::BUILTIN_MODULES_IO.iter() {
            m.insert(module.name, module);
        }
        for module in super::modules_system::BUILTIN_MODULES_SYSTEM.iter() {
            m.insert(module.name, module);
        }
        m
    })
}

fn global_index() -> &'static rustc_hash::FxHashMap<&'static str, &'static super::BuiltinMember> {
    GLOBAL_INDEX.get_or_init(|| {
        let mut m = rustc_hash::FxHashMap::default();
        for g in super::globals::BUILTIN_GLOBALS.iter() {
            m.insert(g.name, g);
        }
        m
    })
}

/// Look up a builtin module by name.
pub fn get_module(name: &str) -> Option<&'static BuiltinModule> {
    module_index().get(name).copied()
}

/// Look up a global builtin function by name.
pub fn get_global(name: &str) -> Option<&'static super::BuiltinMember> {
    global_index().get(name).copied()
}
