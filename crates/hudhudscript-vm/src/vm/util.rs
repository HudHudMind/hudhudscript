/// Cached hash set of builtin function names. Audit v3 Finding 10.1
/// (PERF-18): replaces the ~87-arm `matches!` pattern with an `FxHashSet`
/// so `is_builtin(name)` becomes O(1) regardless of table size.
///
/// The table is populated once behind a `OnceLock` and shared for the
/// lifetime of the process. Every entry is a `&'static str`, so the
/// set never copies strings.

pub(crate) fn builtin_name_set() -> &'static rustc_hash::FxHashSet<&'static str> {
    static SET: std::sync::OnceLock<rustc_hash::FxHashSet<&'static str>> =
        std::sync::OnceLock::new();
    SET.get_or_init(|| {
        let print_aliases: &[&'static str] = crate::vm::builtin_aliases::PRINT_ALIASES;
        let println_aliases: &[&'static str] = crate::vm::builtin_aliases::PRINTLN_ALIASES;
        let eprint_aliases: &[&'static str] = crate::vm::builtin_aliases::EPRINT_ALIASES;
        let eprintln_aliases: &[&str] = crate::vm::builtin_aliases::EPRINTLN_ALIASES;
        let input_aliases: &[&str] = crate::vm::builtin_aliases::INPUT_ALIASES;
        let put_aliases: &[&str] = crate::vm::builtin_aliases::PUT_ALIASES;
        let putf_aliases: &[&str] = crate::vm::builtin_aliases::PUTF_ALIASES;
        let other_builtins: &[&'static str] = &[
            "len",
            "uzunluk",
            "toml",
            "type",
            "toString",
            "metneÇevir",
            "toNumber",
            "sayıyaÇevir",
            "toBoolean",
            "mantıksalaÇevir",
            "booleÇevir",
            "push",
            "pop",
            "keys",
            "values",
            "sqrt",
            "abs",
            "floor",
            "ceil",
            "round",
            "idiv",
            "min",
            "max",
            "concat",
            "slice",
            "indexOf",
            "contains",
            "split",
            "join",
            "replace",
            "trim",
            "toUpperCase",
            "toLowerCase",
            "substring",
            "strrev",
            "length",
            "map",
            "filter",
            "reduce",
            "forEach",
            "find",
            "some",
            "every",
            "Some",
            "Ok",
            "Err",
            "unwrap",
            "unwrap_or",
            "is_some",
            "is_none",
            "is_ok",
            "is_err",
            "env",
            "exception",
            "istisna",
            "Promise.resolve",
            "Promise.reject",
            "Promise.all",
            "Promise.allSettled",
            "Promise.race",
            "dispatch_intent",
            "get_relation",
            "update_relation",
            "enforce_relation",
            "invoke_protocol_hook",
            "spawn_actor",
            "register_constitution",
            "activate_constitution",
            "deactivate_constitution",
            "check_constitution_compliance",
            "mcp_call",
            "tvar_new",
            "tvar_read",
            "tvar_write",
            "atomically",
            "instanceof",
            "sleep",
            "delay",
            "typeof",
            "is",
            "input",
            "input_hidden",
            "confirm",
            "style",
            "setTimeout",
            "setInterval",
            "clearTimeout",
            "clearInterval",
        ];
        let mut set: rustc_hash::FxHashSet<&'static str> =
            rustc_hash::FxHashSet::with_capacity_and_hasher(
                print_aliases.len()
                    + println_aliases.len()
                    + eprint_aliases.len()
                    + eprintln_aliases.len()
                    + input_aliases.len()
                    + put_aliases.len()
                    + putf_aliases.len()
                    + other_builtins.len(),
                Default::default(),
            );
        set.extend(print_aliases.iter().copied());
        set.extend(println_aliases.iter().copied());
        set.extend(eprint_aliases.iter().copied());
        set.extend(eprintln_aliases.iter().copied());
        set.extend(input_aliases.iter().copied());
        set.extend(put_aliases.iter().copied());
        set.extend(putf_aliases.iter().copied());
        set.extend(other_builtins.iter().copied());
        set
    })
}

/// Evaluate a conditional-breakpoint expression string and return whether
/// it is truthy (Issue #661 — VM mirror of
/// `Interpreter::evaluate_condition_static`).
///
/// The static-evaluation approach matches the interpreter exactly: the
/// closure passed to `Debugger::on_statement_with_eval` cannot borrow
/// `&mut self`, so we cannot access the VM's live state from here.
/// Only literal boolean conditions are evaluated precisely; everything
/// else fails open (returns `true`) so the breakpoint still triggers
/// and the user doesn't silently miss it.
///
/// TODO(#661): upgrade to compile the condition to a one-shot
/// bytecode fragment at `setBreakpoints` time and execute it against
/// the paused frame's scope — proper parity with a future interpreter
/// `set_condition_evaluator` hook. Tracked in this file for
/// discoverability; the hook-into-VM-scope work is bigger than a
/// sitting and is explicitly deferred by the task brief.
pub(crate) fn evaluate_condition_static(condition: &str) -> bool {
    let trimmed = condition.trim();
    if trimmed == "true" {
        return true;
    }
    if trimmed == "false" {
        return false;
    }
    // Fail open — breakpoint still triggers for any non-literal
    // condition. Matches the interpreter's best-effort behaviour.
    true
}
