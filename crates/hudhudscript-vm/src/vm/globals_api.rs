use crate::vm::VM;
use hudhudscript_bytecode::interner::SymbolId;
use hudhudscript_bytecode::Value16;

impl VM {
    // ── REGISTER-BASED VM ─────────────────────────────────────────────
    // pop_stack() and push_value() wrappers have been removed. All code
    // now directly accesses registers (registers[255] for accumulator,
    // registers[first_arg..+n] for call arguments). See exec.rs for
    // the register-based call convention (v0.4.224).

    /// Execute the `GetProperty` bytecode op.
    ///
    /// P2-9: Missing-property error parity — when the field is absent and
    /// the name is not an internal marker (starts with `__`), raise a
    /// runtime error that matches the interpreter (see
    /// `crates/hudhudscript-builtins/src/operations/helpers.rs`
    /// `eval_member_access`). Internal markers (`__module`, `__server`,
    /// `__parent__`, `__static__*`, `__actor_id`, ...) are probed by the
    /// runtime and MUST keep returning null on miss. For plain Objects we
    /// also walk `__parent__` to match the interpreter's prototype chain.
    ///
    /// Extracted into a dedicated method so the main interpreter-loop
    /// stack frame stays small (the `execute_instructions` match has many
    /// arms and growing any one of them pushes the frame over the default
    /// thread stack limit for recursion-heavy scripts).
    #[inline(never)]
    pub(crate) fn set_global<V: Into<Value16>>(&mut self, name: &str, value: V) {
        let sym = hudhudscript_bytecode::interner::intern(name);
        self.globals.insert(sym, value.into());
    }

    /// now only globals remain.)
    pub(crate) fn remove_var(&mut self, name: &str) {
        if let Some(sym) = hudhudscript_bytecode::interner::try_resolve_id(name) {
            self.remove_var_by_sym(sym);
        }
    }

    /// Remove a global by SymId — zero allocation.
    pub(crate) fn remove_var_by_sym(&mut self, sym: u32) {
        // T5.2: remove this = reset to empty object (old prelude semantigi).
        if sym == self.this_sym {
            self.cur_this = Value16::object(hudhudscript_bytecode::ObjMap::default());
            return;
        }
        self.globals.remove(&SymbolId(sym));
    }

    /// Define a global variable by name (for external injection).
    pub fn define_global(&mut self, name: String, value: Value16) {
        let sym = hudhudscript_bytecode::interner::intern(&name);
        self.globals.insert(sym, value);
    }

    /// Read a global variable by name (for testing / introspection).
    pub fn get_variable(&self, name: &str) -> Option<&Value16> {
        self.get_var(name)
    }

    /// Clone the named global out — owned `Value` return, Interpreter-API
    /// parity. Use this from external callers (REPL `:vars`, test harness,
    /// Python FFI) so they don't have to re-clone themselves.
    pub fn get_variable_owned(&self, name: &str) -> Option<Value16> {
        self.get_variable(name).cloned()
    }

    /// Iterate over all globals. Used by REPL's `:vars` command after the
    /// AST interpreter was retired. Excludes nothing — callers can filter
    /// out native-function markers themselves.
    pub fn all_globals(&self) -> impl Iterator<Item = (&SymbolId, &Value16)> {
        self.globals.iter()
    }
}
